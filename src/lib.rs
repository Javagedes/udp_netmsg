use downcast_rs::{Downcast,impl_downcast};
use byteorder::{BigEndian,WriteBytesExt,ReadBytesExt};

use std::net::UdpSocket;
use std::collections::HashMap;
use std::string::ToString;
use std::io::{Cursor, ErrorKind};

pub mod utilities;

/// This Trait must be implemented on any struct that you wish to use as a UDP datagram
pub trait Datagram: Downcast {
    /// Called when a datagram with a matching header_id is received
    /// This method should use parse the bytes (easy with the byteorder crate) and return a copy of itself
    fn from_buffer(buffer: Cursor<Vec<u8>>)->Box<Datagram> where Self: Sized;
    /// Called when sending a udp datagram
    /// This method should take the internal variables of the struct and convert them to bytecode (easy with the byteorder crate) and return the buffer
    fn to_buffer(&self)->Vec<u8>;

    /// Static method used to define the header_id.
    fn header()->u32 where Self: Sized;

    /// method used to get the header from specific object
    fn get_header(&self)->u32;
}
impl_downcast!(Datagram);

/// This is the struct that the user will use for sending and receiving datagrams
pub struct NetMessenger {
    /// The udp socket (e.g. 0.0.0.0:12000) is the ip & socket that datagrams are received on
    udp: UdpSocket,
    /// The ip/socket that all datagrams will be sent to
    dest_ip: String,
    /// This is used when receiving a datagram. The recv function pulls out the header from the
    /// buffer and uses it as a key to receive the constructor of the appropriate datagram
    con_map: HashMap<u32, fn(Cursor<Vec<u8>>)->Box<Datagram>>,
    /// The amount of bytes the buffer for receiving data should be.
    /// If your datagrams are huge, this will become slow as it uses vectors as the buffers
    /// about a 30% performance loss when sending / receiving the biggest datagrams (65k bytes)
    recv_buffer_size_bytes: u16,
}

impl NetMessenger {

    /// Simple Constructor for the class
    pub fn new(source_ip: String, dest_ip: String, recv_buffer_size_bytes: u16)->NetMessenger {

        let udp: UdpSocket = UdpSocket::bind(source_ip)
            .expect("Failed to bind to address for sending/receiving datagrams");

        udp.set_nonblocking(true).unwrap();
        let con_map: HashMap<u32, fn(Cursor<Vec<u8>>)->Box<Datagram>> = HashMap::new();

        NetMessenger {
            udp,
            dest_ip,
            con_map,
            recv_buffer_size_bytes,
        }
    }

    /// Checks to see if a datagram has been received. Returns None if it has not.
    /// If data has been received, it removes the header_id and gets the constructor for the
    /// matched datagram struct.
    /// If 'ignore_payload_len' = true, it won't pass payload len (u32) to the datagram constructor
    /// passes remaining buffer to specific datagram constructor and returns struct
    pub fn recv(&mut self, blocking: bool, ignore_payload_len: bool)->Option<Box<Datagram>> {

        if blocking {
            self.udp.set_nonblocking(false).unwrap();
        }

        let mut buffer: Vec<u8> = Vec::new();
        buffer.reserve(self.recv_buffer_size_bytes as usize);
        buffer = vec![0; self.recv_buffer_size_bytes as usize];

        let num_bytes =  match self.udp.recv(&mut buffer){
            Ok(n) => { n },
            Err(e)=> {
                if e.kind() == ErrorKind::WouldBlock {return None}
                else {println!("{}",e);}

                return None; } //Break out of function if we received no bytes
        };

        buffer.resize(num_bytes,0);
        if blocking {self.udp.set_nonblocking(true).unwrap();}

        let mut rdr = Cursor::new(buffer);
        let id = rdr.read_u32::<BigEndian>().unwrap();
        if ignore_payload_len {
            rdr.read_u32::<BigEndian>().unwrap();
        }

        match self.con_map.get(&id) {
            Some(func) => {return Some(func(rdr));}
            None => {println!("unknown msg id: {}", id); return None}
        }
    }

    /// Takes in a struct that implements the datagram trait. Will call the to_buffer method and
    /// If 'include_header' = true: attach the header to the front and send the datagram
    /// If 'include_payload_len' = true: insert payload len (4 bytes) between header & payload
    /// **Incorrectly setting include_payload_len will produce no error but bad data!**
    pub fn send<T: Datagram>(&self, datagram: Box<T>, include_payload_len: bool)->Result<(),String> {

        let mut wtr: Vec<u8> = vec![];
        let mut payload = datagram.to_buffer();

        wtr.write_u32::<BigEndian>(datagram.get_header()).unwrap();

        if include_payload_len {
            wtr.write_u32::<BigEndian>(payload.len() as u32).unwrap();
        }

        wtr.append(&mut payload);

        match self.udp.send_to(&wtr,&self.dest_ip) {
            Ok(_) => return Ok(()),
            Err(e)=> return Err(e.to_string()),
        }
    }

    /// Must register the header_id with the appropriate constructor.
    /// To easily get the header id, use STRUCT_NAME::id().
    /// To easily get the constructor, use STRUCT_NAME::from_buffer
    /// Don't include the parentheses! We are passing in the function, not calling it!
    pub fn register(&mut self, key: u32, con: fn(Cursor<Vec<u8>>)->Box<Datagram> ) {
        self.con_map.insert(key,con);
    }
}

/*
#[cfg(test)]
mod struct_creation {
    use crate::{NetMessenger, Datagram};
    use std::io::Cursor;
    use byteorder::{BigEndian,ReadBytesExt, WriteBytesExt};

    #[derive(Debug)]
    struct UpdatePos {
        pub id: u32,
        pub x: f32,
        pub y: f32,
        pub z: f32
    }

    impl Datagram for UpdatePos {
        fn from_buffer(mut buffer: Cursor<Vec<u8>>)->Box<Datagram> {
            let id = buffer.read_u32::<BigEndian>().unwrap();
            let x = buffer.read_f32::<BigEndian>().unwrap();
            let y = buffer.read_f32::<BigEndian>().unwrap();
            let z = buffer.read_f32::<BigEndian>().unwrap();
            return Box::new(UpdatePos{id,x,y,z})
        }

        fn to_buffer(&self)->Vec<u8> {
            let mut wtr: Vec<u8> = Vec::new();
            //this buffer is 16 + 4 more for the header
            wtr.write_u32::<BigEndian>(self.id).unwrap();
            wtr.write_f32::<BigEndian>(self.x).unwrap();
            wtr.write_f32::<BigEndian>(self.y).unwrap();
            wtr.write_f32::<BigEndian>(self.z).unwrap();
            return wtr
        }

        fn header()->u32 {return 834227670}
    }

    #[test]
    fn correct_source_dest() {
        let net_msg = NetMessenger::new(
            String::from("0.0.0.0:12000"),
            String::from("127.0.0.1:12000"),
            50,
        );

        assert_eq!(net_msg.send(
            Box::from(UpdatePos{id:15, x: 15f32, y: 15f32, z:15f32}),
            true).unwrap(), ());
    }

    #[test]
    #[should_panic]
    fn incorrect_source() {
        NetMessenger::new(
            String::from("12000"),
            String::from("127.0.0.1:12000"),
            50,
        );
    }

    #[test]
    fn incorrect_dest() {
        let net_msg = NetMessenger::new(
            String::from("0.0.0.0:12001"),
            String::from("12001"),
            50,
        );

        match net_msg.send(
            Box::from(UpdatePos{id:15, x: 15f32, y: 15f32, z:15f32}),
            true) {
            Err(_) => assert!(true),
            _ => panic!(),
        }
    }

    #[test]
    fn buffer_size_too_small(){
        let mut net_msg = NetMessenger::new(
            String::from("0.0.0.0:12002"),
            String::from("127.0.0.1:12002"),
            5,
        );

        net_msg.send(
            Box::from(UpdatePos{id:15, x: 15f32, y: 15f32, z:15f32}),
            true).unwrap();

        match net_msg.recv(true, true) {
            None => assert!(true),
            _ => panic!(),
        }
    }

    #[test]
    fn buffer_size_ok() {
        let mut net_msg = NetMessenger::new(
            String::from("0.0.0.0:12003"),
            String::from("127.0.0.1:12003"),
            50,
        );
        net_msg.register(UpdatePos::header(), UpdatePos::from_buffer);

        net_msg.send(
            Box::from(UpdatePos{id:15, x: 15f32, y: 15f32, z:15f32}),
            true).unwrap();

        match net_msg.recv(true, true) {
            None => panic!(),
            _ => assert!(true),
        }
    }

    #[test]
    //to be written
    fn recv_wrong_size_buffer() {
        assert!(true);
    }

}
*/
