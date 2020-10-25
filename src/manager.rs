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
use std::sync::{Arc, Mutex};
use std::any::TypeId;
use crate::util::ThreadSafe;
use crate::serdes::SerDesType;

/// Helper struct for setting up the UDP Manager, either automatic or manuel
pub struct Builder {
    buffer_len: usize,
    socket: String,
    non_blocking: bool,
    read_timeout: Option<std::time::Duration>,
}

impl Builder {
    
    /// Constructor that sets all default values for the Builder (and thus the UDP Manager). 
    /// Users change defaults to meet their needs in the Builder before applying to the UDP Manager.
    pub fn init()->Builder { 
        let buffer_len = 100;
        let socket = String::from("0.0.0.0:39507");
        let read_timeout = None;
        let non_blocking = true;

        return Builder {
            buffer_len,
            socket,
            read_timeout,
            non_blocking,
        }
    }

    /// Sets the buffer_len. The closer the this value is to the size of datagrams, 
    /// the faster the execution. This is because less time is spent reallocating 
    /// memory when the buffer size needs to be increased. To large of a buffer
    /// is also bad as you 1. waste space & 2. waste time allocating unecessary space
    /// Default value: 100 bytes
    pub fn buffer_len(mut self, len: usize) -> Builder {
        self.buffer_len = len;
        return self;
    }

    /// Used to determine how long the system should wait before returning from the try_recv method.
    /// A longer timeout value results in less cpu resources used, but a slower response from the 
    /// method get method as they both need mutable access to the same resource.
    /// Setting this value to anything other then None also sets non_blocking to false as this value is only
    /// necessary when it is blocking.
    /// default value: None
    pub fn read_timeout(mut self, read_timeout: Option<std::time::Duration>) -> Builder {
        
        if read_timeout != None {
            self.non_blocking = false;
        }
        self.read_timeout = read_timeout;
        return self;
    }

    /// Used to determine if the system will block the background thread until a message is received.
    /// Only set this to false if you are certain you will receive a message. Currently this shares mutable access
    /// needs of the same resource with the get method. If data is never received, the try_recv method will never relinquish control 
    /// of the resource over to the get method.
    /// default value: false
    pub fn non_blocking(mut self, non_blocking: bool) -> Builder {
        if non_blocking == true {
            self.read_timeout = None
        }
        self.non_blocking = non_blocking;
        return self
    }

    /// Sets the listening port to receive datagrams on
    /// default value: 39507
    pub fn socket(mut self, socket: String)-> Builder {
        self.socket = socket;
        return self;
    }

    /// Pushes all settings to the UDP Manager and starts the background thread. returns the UdpManager to the user
    pub fn start<T: SerDesType>(self)->UdpManager<T> {
        let len = self.buffer_len;
        let mut man = UdpManager::<T>::init(self).unwrap();
        man.start(len);

        return man;
    }
}

/// Sends and receives datagrams conveniently. Runs background thread to continuously check for datagrams
/// without interrupting other functionality
pub struct UdpManager<T: SerDesType> {

    udp: Arc<UdpSocket>,

    msg_map: Arc<MsgStorage>,
    
    resource_type: PhantomData<T>,

    stop: ThreadSafe<bool>,

    thread: Option<thread::JoinHandle<()>>,
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
        let udp = Arc::from(udp);
        udp.set_nonblocking(non_blocking).unwrap();
        udp.set_read_timeout(read_timeout).unwrap();

        let msg_map = Arc::from(MsgStorage::new());

        Ok(UdpManager {
            udp,
            stop: ThreadSafe::from(false),
            thread: None,
            resource_type,
            msg_map
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
    fn try_recv(udp: Arc<UdpSocket>, msg_map: Arc<MsgStorage>, buffer_len: usize) {
        let mut buffer: Vec<u8> = vec![0; buffer_len];

        let (num_bytes, addr) =  match udp.recv_from(&mut buffer){
            Ok(n) => { println!("received!: {:?}, ", n); n },
            Err(e)=> {
                if e.kind() == ErrorKind::WouldBlock {} //Unix response when non_blocking is true
                else if e.kind() == ErrorKind::TimedOut {}//Windows Response when non_blocking is true
                else {println!("{}",e);} //Prints this to screen instead of crashing for one fail read

                return; } //Break out of function if we received no bytes
        };
        println!("here");
        buffer.truncate(num_bytes);
        let id: Vec<_> = buffer.drain(..8).collect();
        let id = BigEndian::read_u64(&id);

        println!("ID Received: {}", id);
        msg_map.add_msg(id, addr, buffer);
    }

    /// Provides the user with the oldest datagram of the specified type, if one exists. Otherwise
    /// returns None. Provides the deserialized object and the return address to the user
    pub fn get<J:DeserializeOwned + 'static>(&self)->Option<(SocketAddr, J)> {
        return self.msg_map.get_obj::<T,J>();
    }

    /// Deserializes the datagram, appends the ID, and sends to requested location
    pub fn send<J: Serialize + 'static, A: ToSocketAddrs>(&mut self, datagram: J, dest_addr: A)->Result<(),String> {

        let mut wtr: Vec<u8> = vec![];
        let mut payload = match T::serial(&datagram) {
            Ok(obj) => obj,
            Err(_) => return Err(String::from("could not serialize data"))
        };

        let id = self.msg_map.get_id::<J>();
        println!("ID Sent: {}", id);

        wtr.write_u64::<BigEndian>(id).unwrap();

        wtr.append(&mut payload);

        match self.udp.send_to(&wtr, dest_addr) {
            Ok(_) => return Ok(()),
            Err(e)=> return Err(e.to_string()),
        }
    }

    pub fn set_id<F: 'static>(&self, id: u64) {
        self.msg_map.set_id(std::any::TypeId::of::<F>(), id);
    }
}

pub struct MsgStorage {
    msgs: Mutex<HashMap<u64, VecDeque<(SocketAddr, Vec<u8>)>>>,
    ids: Mutex<HashMap<TypeId, u64>>
}

impl MsgStorage {
    /// Change this to Result
    fn get_obj<T: SerDesType, J:DeserializeOwned + 'static>(&self)->Option<(SocketAddr, J)> {
        
        let id = self.get_id::<J>();
        let mut msgs = self.msgs.lock().unwrap();

        match msgs.get_mut(&id) {
            Some(vec) => {
                match vec.pop_front() {
                    Some((addr, vec)) => {
                        match T::deserial(&vec){
                            Ok(obj) => {
                                return Some((addr, obj))
                            },
                            Err(_) => return None
                        }
                    },
                    None => return None
                }
            },
            None => None
        }
    }

    fn add_msg(&self, id: u64, addr: SocketAddr, buffer: Vec<u8>) {
        
        let mut msgs = self.msgs.lock().unwrap();
        
        match msgs.get_mut(&id) {
            Some(vec) => {
                vec.push_back((addr, buffer));
            }
            None => {
                let mut vec = VecDeque::new();
                vec.push_back((addr, buffer));
                msgs.insert(id, vec);
            }
        }
    }

    fn get_id<T: 'static>(&self)->u64 {
        
        let id = std::any::TypeId::of::<T>();
        let mut ids = self.ids.lock().unwrap();

        match ids.get(&id) {        
            Some(val) => return *val,
            None => {
                let obj = MsgStorage::calculate_hash::<T>();
                ids.insert(id, obj);
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

    fn new()->MsgStorage {
        let ids = Mutex::from(HashMap::new());
        let msgs = Mutex::from(HashMap::new());

        return MsgStorage {
            ids,
            msgs
        }
    }

    pub fn set_id(&self, type_id: TypeId, id: u64) {
        let mut ids = self.ids.lock().unwrap();
        ids.insert(type_id, id);
    }
}
