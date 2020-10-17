use byteorder::{BigEndian,WriteBytesExt,ReadBytesExt};
use std::net::UdpSocket;
use std::collections::HashMap;
use std::string::ToString;
use std::io::{Cursor, ErrorKind};
use serde::de::DeserializeOwned;
use std::net::{ToSocketAddrs, SocketAddr};

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

    msg_map: HashMap<u32, Vec<(SocketAddr, Vec<u8>)>>,

    /// The amount of bytes the buffer for receiving data should be.
    /// If your datagrams are huge, this will become slow as it uses vectors as the buffers
    /// about a 30% performance loss when sending / receiving the biggest datagrams (65k bytes)

    recv_buffer_size_bytes: u16,
}

impl NetMessenger {

    /// Simple Constructor for the class
    pub fn new(source_ip: String, recv_buffer_size_bytes: u16)->Result<NetMessenger, &'static str> {

        let udp: UdpSocket = match UdpSocket::bind(source_ip) {
            Ok(socket) => socket,
            Err(_) => return Err("Failed to bind to socket")
        };
            

        udp.set_nonblocking(true).unwrap();
        let msg_map: HashMap<u32, Vec<(SocketAddr, Vec<u8>)>> = HashMap::new();

        Ok(NetMessenger {
            udp,
            msg_map,
            recv_buffer_size_bytes,
        })
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

        let (num_bytes, addr) =  match self.udp.recv_from(&mut buffer){
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
        vec.push((addr, data));
    }

    pub fn get<T: Datagram + DeserializeOwned >(&mut self)->Option<(SocketAddr, T)> {
        
        match self.msg_map.get_mut(&T::header()) {
            Some(data) => {
                let (addr, vec) = &data.remove(0);
                return Some((*addr, serde_json::from_slice(vec).unwrap()))
            },
            None => return None
        };
    }

    /// Takes in a struct that implements the datagram trait. Will call the to_buffer method and
    /// If 'include_header' = true: attach the header to the front and send the datagram
    /// If 'include_payload_len' = true: insert payload len (4 bytes) between header & payload
    /// **Incorrectly setting include_payload_len will produce no error but bad data!**

    pub fn send<T: Datagram, A: ToSocketAddrs>(&self, datagram: T, dest_addr: A)->Result<(),String> {

        let mut wtr: Vec<u8> = vec![];
        let mut payload = datagram.serial();

        wtr.write_u32::<BigEndian>(datagram.get_header()).unwrap();

        wtr.append(&mut payload);

        match self.udp.send_to(&wtr, dest_addr) {
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
            50,
        ).unwrap();

        assert_eq!(net_msg.send(UpdatePos{x: 15f32, y: 15f32, z:15f32}, String::from("127.0.0.1:12000")).unwrap(), ());
    }

    #[test]
    #[should_panic]
    fn incorrect_source() {
        NetMessenger::new(
            String::from("12000"),
            50,
        ).unwrap();
    }

    #[test]
    #[should_panic]
    fn buffer_size_too_small(){
        let mut net_msg = NetMessenger::new(
            String::from("0.0.0.0:12002"),
            5,
        ).unwrap();

        net_msg.register(UpdatePos::header());
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:12002")).unwrap();

        net_msg.recv(true);

        net_msg.get::<UpdatePos>().unwrap();
    }

    #[test]
    fn buffer_size_ok() {
        let mut net_msg = NetMessenger::new(
            String::from("0.0.0.0:12003"),
            50,
        ).unwrap();

        net_msg.register(UpdatePos::header());

        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};

        net_msg.send(pos, String::from("127.0.0.1:12003")).unwrap();

        net_msg.recv(true);

        net_msg.get::<UpdatePos>().unwrap();
    }
}

