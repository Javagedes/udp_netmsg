use byteorder::{BigEndian,WriteBytesExt,ReadBytesExt};
use std::net::UdpSocket;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::{Cursor, ErrorKind};
use serde::de::DeserializeOwned;
use std::net::{ToSocketAddrs, SocketAddr};
use std::thread;



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
    /// The socket that datagrams are received on
    udp: Arc<Mutex<UdpSocket>>,

    /// Storage mechanism for datagrams
    msg_map: Arc<Mutex<HashMap<u32, Vec<(SocketAddr, Vec<u8>)>>>>,

    /// The reserved size of buffer. Since this is a vec, it will expand if needed.
    recv_buffer_size_bytes: u16,

    stop: Arc<Mutex<bool>>
}

impl NetMessenger {

    /// Simple Constructor for the class
    pub fn new(source_ip: String)->Result<NetMessenger, &'static str> {

        let udp: UdpSocket = match UdpSocket::bind(source_ip) {
            Ok(socket) => socket,
            Err(_) => return Err("Failed to bind to socket")
        };
            
        udp.set_nonblocking(true).unwrap();

        let udp = Arc::from(Mutex::from(udp));
        let msg_map: Arc<Mutex<HashMap<u32, Vec<(SocketAddr, Vec<u8>)>>>> = Arc::from(Mutex::from(HashMap::new()));

        Ok(NetMessenger {
            udp,
            msg_map,
            recv_buffer_size_bytes: 100,
            stop: Arc::from(Mutex::from(false))
        })
    }

    pub fn set_buffer_size(&mut self, recv_buffer_size_bytes: u16) {
        self.recv_buffer_size_bytes = recv_buffer_size_bytes;
    }

    pub fn start(&self){

        let udp = self.udp.clone();
        let buffer_len = self.recv_buffer_size_bytes as usize;
        let msg_map = self.msg_map.clone();
        let stop = self.stop.clone();

        thread::spawn( move || {
            println!("Detection Started");
            while *stop.lock().unwrap() == false {
                NetMessenger::recv(udp.clone(), msg_map.clone(), buffer_len,false);
            }

            println!("Detection Stopped");
        });
    }

    pub fn stop(&self){
        *self.stop.lock().unwrap() = false;
    }

    /// Checks to see if a datagram has been received. Returns None if it has not.
    /// If data has been received, it removes the header_id and gets the constructor for the
    /// matched datagram struct.
    /// If 'ignore_payload_len' = true, it won't pass payload len (u32) to the datagram constructor
    /// passes remaining buffer to specific datagram constructor and returns struct
    pub fn recv(udp: Arc<Mutex<UdpSocket>>, msg_map: Arc<Mutex<HashMap<u32, Vec<(SocketAddr, Vec<u8>)>>>>, buffer_len: usize, blocking: bool ){

        let udp = udp.lock().unwrap();
        let mut msg_map = msg_map.lock().unwrap();

        if blocking {
            udp.set_nonblocking(false).unwrap();
        }

        let mut buffer: Vec<u8> = Vec::new();
        buffer.reserve(buffer_len);
        buffer = vec![0; buffer_len];

        let (num_bytes, addr) =  match udp.recv_from(&mut buffer){
            Ok(n) => { println!("Data Recieved"); n },
            Err(e)=> {
                if e.kind() == ErrorKind::WouldBlock {}
                else {println!("{}",e);}

                return; } //Break out of function if we received no bytes
        };

        buffer.resize(num_bytes,0);
        if blocking {udp.set_nonblocking(true).unwrap();}

        let mut rdr = Cursor::new(buffer);
        let id = rdr.read_u32::<BigEndian>().unwrap();
        let mut data = rdr.into_inner();
        data.remove(0);
        data.remove(0);
        data.remove(0);
        data.remove(0);

        let vec = msg_map.get_mut(&id).unwrap();
        println!("Adding to the vec...");
        vec.push((addr, data));
    }

    //TODO: Need to handle the .remote(0) error when vec is empty
    pub fn get<T: Datagram + DeserializeOwned >(&mut self)->Option<(SocketAddr, T)> {
        
        match self.msg_map.lock().unwrap().get_mut(&T::header()) {
            Some(data) => {
                let (addr, vec) = &data.remove(0);
                return Some((*addr, serde_json::from_slice(vec).unwrap()))
            },
            None => return None
        };
    }

    /// Sends datagram (with header) to dest_addr
    pub fn send<T: Datagram, A: ToSocketAddrs>(&self, datagram: T, dest_addr: A)->Result<(),String> {

        let mut wtr: Vec<u8> = vec![];
        let mut payload = datagram.serial();

        wtr.write_u32::<BigEndian>(datagram.get_header()).unwrap();

        wtr.append(&mut payload);

        match self.udp.lock().unwrap().send_to(&wtr, dest_addr) {
            Ok(_) => return Ok(()),
            Err(e)=> return Err(e.to_string()),
        }
    }

    /// Must register the header_id with the appropriate constructor.
    /// To easily get the header id, use STRUCT_NAME::id().
    /// To easily get the constructor, use STRUCT_NAME::from_buffer
    /// Don't include the parentheses! We are passing in the function, not calling it!
    pub fn register(&mut self, key: u32) {
        self.msg_map.lock().unwrap().insert(key,Vec::new());
    }
}


#[cfg(test)]
mod struct_creation {
    use crate::{NetMessenger, Datagram};
    use serde::{Serialize, Deserialize};
    use std::{thread, time};

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
            String::from("0.0.0.0:12000")
        ).unwrap();

        assert_eq!(net_msg.send(UpdatePos{x: 15f32, y: 15f32, z:15f32}, String::from("127.0.0.1:12000")).unwrap(), ());
    }

    #[test]
    #[should_panic]
    fn incorrect_source() {
        NetMessenger::new(
            String::from("12000")
        ).unwrap();
    }

    #[test]
    fn send_receive_data() {
        let mut net_msg = NetMessenger::new(
            String::from("0.0.0.0:12004")
        ).unwrap();

        net_msg.register(UpdatePos::header());
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:12004")).unwrap();

        net_msg.start();
        thread::sleep(time::Duration::from_millis(1000));
        net_msg.stop();

        net_msg.get::<UpdatePos>().unwrap();
    }
}

