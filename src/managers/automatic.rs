use byteorder::{ByteOrder, BigEndian, WriteBytesExt};
use std::net::UdpSocket;
use std::collections::{HashMap, VecDeque};
use std::io::ErrorKind;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::net::{ToSocketAddrs, SocketAddr};
use std::thread;
use std::marker::PhantomData;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::threadsafe::ThreadSafe;
use crate::serdes::SerDesType;
use crate::managers::Builder;

/// Sends and receives datagrams conveniently. Runs background thread to continuously check for datagrams
/// without interrupting other functionality
pub struct UdpManager<T: SerDesType> {

    udp: ThreadSafe<UdpSocket>,

    msg_map: ThreadSafe<HashMap<u64, VecDeque<(SocketAddr, Vec<u8>)>>>,
    
    resource_type: PhantomData<T>,

    stop: ThreadSafe<bool>,

    thread: Option<thread::JoinHandle<()>>,

    id_storage: ThreadSafe<IdStorage>
}

/// Allows the background thread to safely shutdown when the struct loses scope or program shutsdown
impl<T: SerDesType> Drop for UdpManager<T> {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Methods for the struct
impl <T: SerDesType>UdpManager<T> {

    /// Constructor for the class. Can only be called by the Builder helper.
    pub fn init<K: SerDesType>(builder: Builder)->Result<UdpManager<K>, &'static str> {

        let socket = builder.socket;
        let read_timeout = builder.read_timeout;
        let resource_type = PhantomData;
        let non_blocking = builder.non_blocking;

        let udp: UdpSocket = match UdpSocket::bind(socket) {
            Ok(socket) => socket,
            Err(_) => return Err("Failed to bind to socket")
        };
        
        udp.set_nonblocking(non_blocking).unwrap();
        udp.set_read_timeout(read_timeout).unwrap();

        let udp = ThreadSafe::from(udp);
        let msg_map: ThreadSafe<HashMap<u64, VecDeque<(SocketAddr, Vec<u8>)>>> = ThreadSafe::from(HashMap::new());
        let id_storage = ThreadSafe::from(IdStorage::new());

        Ok(UdpManager {
            udp,
            msg_map,
            stop: ThreadSafe::from(false),
            thread: None,
            id_storage,
            resource_type
        })
    }

    /// Spawns the background thread for receiving datagrams. Only called by builder method
    pub fn start(&mut self, buffer_len: usize){

        let udp = self.udp.clone();
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

    /// Safely closes the background thread. Automatically called when struct is dropped
    fn stop(&mut self){
        *self.stop.lock().unwrap() = true;
        self.thread.take().map(thread::JoinHandle::join);
    }

    /// Tries to receive a Datagram from the socket. If no datagram is available, will either return, or sit and wait
    /// depending on if the underlying UDPSocket was set to non_blocking or not
    fn try_recv(udp: ThreadSafe<UdpSocket>, msg_map: ThreadSafe<HashMap<u64, VecDeque<(SocketAddr, Vec<u8>)>>>, buffer_len: usize) {
        let udp = udp.lock().unwrap();
        let mut msg_map = msg_map.lock().unwrap();
        //let id_storage = id_storage.lock().unwrap();

        let mut buffer: Vec<u8> = vec![0; buffer_len];

        let (num_bytes, addr) =  match udp.recv_from(&mut buffer){
            Ok(n) => { println!("received!: {:?}, ", n); n },
            Err(e)=> {
                if e.kind() == ErrorKind::WouldBlock {} //Unix response when non_blocking is true
                else if e.kind() == ErrorKind::TimedOut {}//Windows Response when non_blocking is true
                else {println!("{}",e);} //Prints this to screen instead of crashing for one fail read

                return; } //Break out of function if we received no bytes
        };

        buffer.truncate(num_bytes);
        let id: Vec<_> = buffer.drain(..8).collect();
        let id = BigEndian::read_u64(&id);

        match msg_map.get_mut(&id) {
            Some(vec) => {
                vec.push_back((addr, buffer));
            }
            None => {
                let mut vec = VecDeque::new();
                vec.push_back((addr, buffer));
                msg_map.insert(id, vec);
            }
        }

    }

    /// Provides the user with the oldest datagram of the specified type, if one exists. Otherwise
    /// returns None. Provides the deserialized object and the return address to the user
    pub fn get<J:DeserializeOwned + 'static>(&self)->Option<(SocketAddr, J)> {
        let mut id_storage = self.id_storage.lock().unwrap();
        let id = id_storage.get_id::<J>();

        match self.msg_map.lock().unwrap().get_mut(&id) {
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

    /// Deserializes the datagram, appends the ID, and sends to requested location
    pub fn send<J: Serialize + 'static, A: ToSocketAddrs>(&mut self, datagram: J, dest_addr: A)->Result<(),String> {

        let mut wtr: Vec<u8> = vec![];
        let mut payload = match T::serial(&datagram) {
            Ok(obj) => obj,
            Err(_) => return Err(String::from("could not serialize data"))
        };

        let mut id_storage = self.id_storage.lock().unwrap();
        let id = id_storage.get_id::<J>();

        wtr.write_u64::<BigEndian>(id).unwrap();

        wtr.append(&mut payload);

        match self.udp.lock().unwrap().send_to(&wtr, dest_addr) {
            Ok(_) => return Ok(()),
            Err(e)=> return Err(e.to_string()),
        }
    }
}

use std::any::TypeId;
pub struct IdStorage {
    id: HashMap<TypeId, u64>
}

impl IdStorage {

    pub fn get_id<T: 'static>(&mut self)->u64 {
        
        let id = std::any::TypeId::of::<T>();

        match self.id.get(&id) {
            Some(val) => return *val,
            None => {
                let obj = IdStorage::calculate_hash::<T>();
                self.id.insert(id, obj);
                return obj;
            }
        }
    }

    fn calculate_hash<T: 'static>()->u64 {
        let mut hasher = DefaultHasher::new();
        let x = std::any::TypeId::of::<T>();
        x.hash(&mut hasher);
        return hasher.finish();
    }

    pub fn new()->IdStorage {
        let id = HashMap::new();

        return IdStorage {
            id
        }
    }
}