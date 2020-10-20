use byteorder::{ByteOrder, BigEndian, WriteBytesExt};
use std::net::UdpSocket;
use std::collections::{HashMap, VecDeque};
use std::io::ErrorKind;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::net::{ToSocketAddrs, SocketAddr};
use std::{thread, time};
use std::marker::PhantomData;

use super::threadsafe::ThreadSafe;
use super::datagram::Datagram;
use super::serdes::SerDesType;

/// This is the struct that the user will use for sending and receiving datagrams
///TODO: Change the way messages are stored so that user can chose to either get the next message in time received
/// or get it by message type.
//We can store it as just a long vec of (u32, SocketAddr, Vec<u8>). Then we can get them in order or filter them
//With maybe a map that only returns the correct ones based off the u32 identifier.
pub struct UdpManager<T: SerDesType> {

    udp: ThreadSafe<UdpSocket>,

    msg_map: ThreadSafe<HashMap<u32, VecDeque<(SocketAddr, Vec<u8>)>>>,
    
    resource_type: PhantomData<T>,

    recv_buffer_size_bytes: usize,

    stop: ThreadSafe<bool>,

    thread: Option<thread::JoinHandle<()>>
}

impl <T: SerDesType>UdpManager<T> {

    /// Simple Constructor for the class
    fn new(builder: Builder<T>)->Result<UdpManager<T>, &'static str> {

        let source_ip = builder.source_ip;
        let recv_buffer_size_bytes = builder.buffer_len;
        let read_timeout = builder.read_timeout;
        let resource_type = builder.resource_type;

        let udp: UdpSocket = match UdpSocket::bind(source_ip) {
            Ok(socket) => socket,
            Err(_) => return Err("Failed to bind to socket")
        };
            
        udp.set_nonblocking(false).unwrap();
        udp.set_read_timeout(read_timeout).unwrap();

        let udp = ThreadSafe::from(udp);
        let msg_map: ThreadSafe<HashMap<u32, VecDeque<(SocketAddr, Vec<u8>)>>> = ThreadSafe::from(HashMap::new());

        Ok(UdpManager {
            udp,
            msg_map,
            recv_buffer_size_bytes,
            stop: ThreadSafe::from(false),
            thread: None,
            resource_type
        })
    }

    fn start(&mut self){

        let udp = self.udp.clone();
        //Due to how this is set up, the buffer size cant be changed after started
        let buffer_len = self.recv_buffer_size_bytes as usize;
        let msg_map = self.msg_map.clone();
        let stop = self.stop.clone();

        let thread = thread::Builder::new()
            .name(String::from("thread_udp_listener"))
            .spawn( move || {
                while *stop.lock().unwrap() == false {
                    Self::try_recv(udp.clone(), msg_map.clone(), buffer_len);
                }
        }).unwrap();

        self.thread = Some(thread);
    }

    pub fn stop(&mut self){
        *self.stop.lock().unwrap() = true;
        self.thread.take().map(thread::JoinHandle::join);
    }

    //Tries to receive a Datagram from the socket. If no datagram is available, it simply returns.
    //fn try_recv(udp: ThreadSafe<UdpSocket>, msg_map: ThreadSafe<HashMap<u32, VecDeque<(SocketAddr, Vec<u8>)>>>, buffer_len: usize){
    fn try_recv(udp: ThreadSafe<UdpSocket>, msg_map: ThreadSafe<HashMap<u32, VecDeque<(SocketAddr, Vec<u8>)>>>, buffer_len: usize) {
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
    pub fn get<J: Datagram + DeserializeOwned >(&mut self)->Option<(SocketAddr, J)> {
        
        match self.msg_map.lock().unwrap().get_mut(&J::header()) {
            Some(data) => {
                match data.pop_front() {
                    None => return None,
                    Some((addr, vec)) => {
                        match T::deserial(&vec) {
                            Ok(obj) => return Some((addr,obj)),
                            Err(_) => return None
                        }
                    }
                }
            },
            None => return None
        };
    }

    /// Sends datagram (with header) to dest_addr
    pub fn send<J: Serialize + Datagram, A: ToSocketAddrs>(&self, datagram: J, dest_addr: A)->Result<(),String> {

        let mut wtr: Vec<u8> = vec![];
        let mut payload = match T::serial(&datagram) {
            Ok(obj) => obj,
            Err(_) => return Err(String::from("could not serialize data"))
        };

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

pub struct Builder<T: SerDesType> {
    buffer_len: usize,
    source_ip: String,
    read_timeout: Option<std::time::Duration>,
    resource_type: PhantomData<T>
}

impl <T: SerDesType>Builder<T> {
    
    pub fn new()->Builder<T> {
        let buffer_len = 100;
        let source_ip = String::from("0.0.0.0:39507");
        let read_timeout = Some(time::Duration::from_millis(250));

        return Builder {
            buffer_len,
            source_ip,
            read_timeout,
            resource_type: PhantomData
        }
    }

    pub fn buffer_len(mut self, len: usize) -> Builder<T> {
        self.buffer_len = len;
        return self;
    }

    pub fn read_timeout(mut self, read_timeout: Option<std::time::Duration>) -> Builder<T> {
        self.read_timeout = read_timeout;
        return self;
    }

    pub fn source_ip(mut self, source_ip: String)-> Builder<T> {
        self.source_ip = source_ip;
        return self;
    }

    pub fn start(self)->UdpManager<T> {
        let mut man = UdpManager::new(self).unwrap();

        man.start();

        return man;
    }
}
