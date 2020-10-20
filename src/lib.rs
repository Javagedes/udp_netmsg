use byteorder::{ByteOrder, BigEndian, WriteBytesExt};
use std::net::UdpSocket;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::io::ErrorKind;
use serde::de::DeserializeOwned;
use std::net::{ToSocketAddrs, SocketAddr};
use std::{thread, time};

pub struct ThreadSafe<T> {
    obj: Arc<Mutex<T>>
}

impl <T>ThreadSafe<T> {
    fn lock(&self)->Result<std::sync::MutexGuard<'_, T>,
    std::sync::PoisonError<std::sync::MutexGuard<'_, T>>>
    {
        return self.obj.lock()
    }

    fn clone(&self)->ThreadSafe<T> {
        return ThreadSafe{obj: self.obj.clone()}
    }

    fn from(obj: T)->ThreadSafe<T> {
        return ThreadSafe{obj: Arc::from(Mutex::from(obj))}
    }
}

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

//We can store it as just a long vec of (u32, SocketAddr, Vec<u8>). Then we can get them in order or filter them
//With maybe a map that only returns the correct ones based off the u32 identifier.
pub struct NetMessenger {
    /// The socket that datagrams are received on
    udp: ThreadSafe<UdpSocket>,

    /// Storage mechanism for datagrams
    msg_map: ThreadSafe<HashMap<u32, VecDeque<(SocketAddr, Vec<u8>)>>>,

    /// The reserved size of buffer. Since this is a vec, it will expand if needed.
    recv_buffer_size_bytes: u16,

    stop: ThreadSafe<bool>,

    thread: Option<thread::JoinHandle<()>>
}

impl NetMessenger {

    /// Simple Constructor for the class
    pub fn new(source_ip: String)->Result<NetMessenger, &'static str> {

        let udp: UdpSocket = match UdpSocket::bind(source_ip) {
            Ok(socket) => socket,
            Err(_) => return Err("Failed to bind to socket")
        };
            
        udp.set_nonblocking(false).unwrap();
        udp.set_read_timeout(Some(time::Duration::from_millis(250))).unwrap();

        let udp = ThreadSafe::from(udp);
        let msg_map: ThreadSafe<HashMap<u32, VecDeque<(SocketAddr, Vec<u8>)>>> = ThreadSafe::from(HashMap::new());

        Ok(NetMessenger {
            udp,
            msg_map,
            recv_buffer_size_bytes: 100,
            stop: ThreadSafe::from(false),
            thread: None
        })
    }

    pub fn set_buffer_size(&mut self, recv_buffer_size_bytes: u16) {
        self.recv_buffer_size_bytes = recv_buffer_size_bytes;
    }

    pub fn start(&mut self){

        let udp = self.udp.clone();
        //Due to how this is set up, the buffer size cant be changed after started
        let buffer_len = self.recv_buffer_size_bytes as usize;
        let msg_map = self.msg_map.clone();
        let stop = self.stop.clone();

        let thread = thread::Builder::new()
            .name(String::from("thread_udp_listener"))
            .spawn( move || {
                while *stop.lock().unwrap() == false {
                    NetMessenger::try_recv(udp.clone(), msg_map.clone(), buffer_len);
                }
        }).unwrap();

        self.thread = Some(thread);
    }

    pub fn stop(&mut self){
        *self.stop.lock().unwrap() = true;
        self.thread.take().map(thread::JoinHandle::join);
    }

    //Tries to receive a Datagram from the socket. If no datagram is available, it simply returns.
    fn try_recv(udp: ThreadSafe<UdpSocket>, msg_map: ThreadSafe<HashMap<u32, VecDeque<(SocketAddr, Vec<u8>)>>>, buffer_len: usize){

        let udp = udp.lock().unwrap();
        let mut msg_map = msg_map.lock().unwrap();

        let mut buffer: Vec<u8> = vec![0; buffer_len];

        let (num_bytes, addr) =  match udp.recv_from(&mut buffer){
            Ok(n) => { n },
            Err(e)=> {
                if e.kind() == ErrorKind::WouldBlock {} //Unix response
                else if e.kind() == ErrorKind::TimedOut {}//Windows Response
                else {println!("{}",e);} //Prints this to screen instead of crashing for one fail read

                return; } //Break out of function if we received no bytes
        };

        buffer.truncate(num_bytes);
        let id: Vec<_> = buffer.drain(..4).collect();
        let id = BigEndian::read_u32(&id);
        let vec = msg_map.get_mut(&id).unwrap();
        vec.push_back((addr, buffer));
    }

    //TODO: Need to handle the .remove(0) error when vec is empty
    pub fn get<T: Datagram + DeserializeOwned >(&mut self)->Option<(SocketAddr, T)> {
        
        match self.msg_map.lock().unwrap().get_mut(&T::header()) {
            Some(data) => {
                match data.pop_front() {
                    None => return None,
                    Some((addr, vec)) => return Some((addr, serde_json::from_slice(&vec).unwrap()))
                }
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
        self.msg_map.lock().unwrap().insert(key,VecDeque::new());
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
        thread::sleep(time::Duration::from_millis(100));
        net_msg.stop();

        net_msg.get::<UpdatePos>().unwrap();
    }
}

