use byteorder::{BigEndian,WriteBytesExt,ReadBytesExt};

use std::net::UdpSocket;
use std::collections::HashMap;
use std::string::ToString;
use std::io::{Cursor, ErrorKind};
//use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;

pub mod utilities;

/// This Trait must be implemented on any struct that you wish to use as a UDP datagram
pub trait Datagram {
    /// Called when sending a udp datagram
    /// This method should take the internal variables of the struct and convert them to bytecode (easy with the byteorder crate) and return the buffer
    fn serial(&self)->Vec<u8>;

    /// Static method used to define the header_id.
    fn header()->u32;

    /// method used to get the header from specific object
    fn get_header(&self)->u32 {
        return Self::header();
    }
}

/// This is the struct that the user will use for sending and receiving datagrams
///TODO: Change the way messages are stored so that user can chose to either get the next message in time received
/// or get it by message type.
pub struct NetMessenger {
    /// The udp socket (e.g. 0.0.0.0:12000) is the ip & socket that datagrams are received on
    udp: UdpSocket,
    /// The ip/socket that all datagrams will be sent to
    dest_ip: String,

    /// The amount of bytes the buffer for receiving data should be.
    /// If your datagrams are huge, this will become slow as it uses vectors as the buffers
    /// about a 30% performance loss when sending / receiving the biggest datagrams (65k bytes)
    
    msg_map: HashMap<u32, Vec<Vec<u8>>>,
    recv_buffer_size_bytes: u16,
}

impl NetMessenger {

    /// Simple Constructor for the class
    pub fn new(source_ip: String, dest_ip: String, recv_buffer_size_bytes: u16)->NetMessenger {

        let udp: UdpSocket = UdpSocket::bind(source_ip)
            .expect("Failed to bind to address for sending/receiving datagrams");

        udp.set_nonblocking(true).unwrap();
        let msg_map: HashMap<u32, Vec<Vec<u8>>> = HashMap::new();

        NetMessenger {
            udp,
            dest_ip,
            msg_map,
            recv_buffer_size_bytes,
        }
    }

    /// Checks to see if a datagram has been received. Returns None if it has not.
    /// If data has been received, it removes the header_id and gets the constructor for the
    /// matched datagram struct.
    /// If 'ignore_payload_len' = true, it won't pass payload len (u32) to the datagram constructor
    /// passes remaining buffer to specific datagram constructor and returns struct
    pub fn recv(&mut self, blocking: bool){

        if blocking {
            self.udp.set_nonblocking(false).unwrap();
        }

        let mut buffer: Vec<u8> = Vec::new();
        buffer.reserve(self.recv_buffer_size_bytes as usize);
        buffer = vec![0; self.recv_buffer_size_bytes as usize];

        let num_bytes =  match self.udp.recv(&mut buffer){
            Ok(n) => { n },
            Err(e)=> {
                if e.kind() == ErrorKind::WouldBlock {}
                else {println!("{}",e);}

                return; } //Break out of function if we received no bytes
        };

        buffer.resize(num_bytes,0);
        if blocking {self.udp.set_nonblocking(true).unwrap();}

        let mut rdr = Cursor::new(buffer);
        let id = rdr.read_u32::<BigEndian>().unwrap();
        let mut data = rdr.into_inner();
        data.remove(0);
        data.remove(0);
        data.remove(0);
        data.remove(0);

        let vec = self.msg_map.get_mut(&id).unwrap();
        vec.push(data);
    }

    pub fn get<T: Datagram + DeserializeOwned >(&mut self)->Option<T> {
        
        match self.msg_map.get_mut(&T::header()) {
            Some(data) => {
                return serde_json::from_slice(&data.remove(0)).unwrap()
            },
            None => return None
        };
    }

    /// Takes in a struct that implements the datagram trait. Will call the to_buffer method and
    /// If 'include_header' = true: attach the header to the front and send the datagram
    /// If 'include_payload_len' = true: insert payload len (4 bytes) between header & payload
    /// **Incorrectly setting include_payload_len will produce no error but bad data!**
    pub fn send<T: Datagram>(&self, datagram: T)->Result<(),String> {

        let mut wtr: Vec<u8> = vec![];
        let mut payload = datagram.serial();

        wtr.write_u32::<BigEndian>(datagram.get_header()).unwrap();

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
    pub fn register(&mut self, key: u32) {
        self.msg_map.insert(key,Vec::new());
    }
}


#[cfg(test)]
mod struct_creation {
    use crate::{NetMessenger, Datagram};
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    struct UpdatePos {
        pub x: f32,
        pub y: f32,
        pub z: f32
    }

    impl Datagram for UpdatePos {

        fn serial(&self)->Vec<u8> {
            
            let vec = serde_json::to_vec(self).unwrap();

            return vec
        }

        fn header()->u32 {return 834227670}
        fn get_header(&self)->u32 { return 834227670 }
    }

    #[test]
    fn correct_source_dest() {
        let net_msg = NetMessenger::new(
            String::from("0.0.0.0:12000"),
            String::from("127.0.0.1:12000"),
            50,
        );

        assert_eq!(net_msg.send(UpdatePos{x: 15f32, y: 15f32, z:15f32}).unwrap(), ());
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

        match net_msg.send(UpdatePos{x: 15f32, y: 15f32, z:15f32}) {
            Err(_) => assert!(true),
            _ => panic!(),
        }
    }

    #[test]
    #[should_panic]
    fn buffer_size_too_small(){
        let mut net_msg = NetMessenger::new(
            String::from("0.0.0.0:12002"),
            String::from("127.0.0.1:12002"),
            5,
        );

        net_msg.register(UpdatePos::header());
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos).unwrap();

        net_msg.recv(true);

        net_msg.get::<UpdatePos>().unwrap();
    }

    #[test]
    fn buffer_size_ok() {
        let mut net_msg = NetMessenger::new(
            String::from("0.0.0.0:12003"),
            String::from("127.0.0.1:12003"),
            50,
        );
        net_msg.register(UpdatePos::header());

        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};

        net_msg.send(pos).unwrap();

        net_msg.recv(true);

        net_msg.get::<UpdatePos>().unwrap();
    }
}

