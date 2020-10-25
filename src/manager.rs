use std::any::TypeId;
use std::collections::{hash_map, HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::net::{UdpSocket, ToSocketAddrs, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;

use serde::{de, ser};
use byteorder::{ByteOrder, BigEndian, WriteBytesExt};

use crate::util::ThreadSafe;
use crate::serdes::SerDesType;

/// Helper struct for configuring the UDP Manager.
pub struct Builder {
    buffer_len: usize,
    socket: String,
    non_blocking: bool,
    read_timeout: Option<std::time::Duration>,
}

impl Builder {  
    /// Initializer that sets default configuration values. These configurations may be changed via
    /// provided methods to meet the needs of the program.
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
    /// is also bad as you 1. waste space & 2. waste time allocating unecessary space.
    /// 
    /// # Default value
    /// 
    /// 100 bytes
    pub fn buffer_len(mut self, len: usize) -> Builder {
        self.buffer_len = len;
        return self;
    }

    /// Used to determine how long the system should wait before returning from the try_recv method.
    /// A longer timeout value results in less cpu resources used, but a slower response from the 
    /// method get method as they both need mutable access to the same resource.
    /// Setting this value to anything other then None also sets non_blocking to false as this value is only
    /// necessary when it is blocking.
    /// 
    /// # Default value
    /// 
    /// None
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
    /// 
    /// # Default Value
    /// 
    /// False
    pub fn non_blocking(mut self, non_blocking: bool) -> Builder {
        if non_blocking == true {
            self.read_timeout = None
        }
        self.non_blocking = non_blocking;
        return self
    }

    /// Sets the listening port to receive datagrams on.
    /// 
    /// # Default Value
    /// 
    /// 39507
    pub fn socket(mut self, socket: String)-> Builder {
        self.socket = socket;
        return self;
    }

    /// Creates and starts the UDP Manager
    /// 
    /// Uses the configurations set with the builder struct to initialize and start the UDP Manager.
    /// Specifies the SerDes format for data. Spins up the background thread that continiously checks 
    /// for datagrams
    /// 
    /// # Errors
    /// 
    /// Errors if configurations to the underlying UDP Socket fail or if it was unable to create the 
    /// new thread at the OS level.
    pub fn start<T: SerDesType>(self)->Result<UdpManager<T>, std::io::Error> {
        let len = self.buffer_len;
        let mut manager = UdpManager::<T>::init(self)?;
        
        manager.start(len)?;

        return Ok(manager);
    }
}

/// Sends and receives datagrams conveniently. Runs a background thread to continuously check for datagrams
/// without interrupting other functionality.
pub struct UdpManager<T: SerDesType> {

    udp: Arc<UdpSocket>,

    msg_map: Arc<MsgStorage>,
    
    resource_type: PhantomData<T>,

    stop: ThreadSafe<bool>,

    thread: Option<thread::JoinHandle<()>>,
}

/// Allows the background thread to safely shutdown when the struct loses scope or program performs a shutdown.
impl<T: SerDesType> Drop for UdpManager<T> {
    fn drop(&mut self) {
        self.stop();
    }
}

impl <T: SerDesType>UdpManager<T> {

    /// initializer for the class that is only callable by the builder. Uses configured values
    /// from the builder helper to set the manager. 
    /// 
    /// # Errors
    /// 
    /// Initialization will fail if it is unable to set the nonblocking or read timeout values to 
    /// the underlying udp socket.
    fn init<K: SerDesType>(builder: Builder)->Result<UdpManager<K>, std::io::Error> {

        let socket = builder.socket;
        let read_timeout = builder.read_timeout;
        let resource_type = PhantomData;
        let non_blocking = builder.non_blocking;

        let udp: UdpSocket = UdpSocket::bind(socket)?;
        let udp = Arc::from(udp);
        
        udp.set_nonblocking(non_blocking)?;
        udp.set_read_timeout(read_timeout)?;

        let msg_map = Arc::from(MsgStorage::new());

        Ok(UdpManager {
            udp,
            stop: ThreadSafe::from(false),
            thread: None,
            resource_type,
            msg_map
        })
    }

    /// Spawns the background thread for receiving datagrams. Only callable by builder.
    /// 
    /// # Errors
    ///  
    /// Fails if unable to create a new thread at the OS level.
    fn start(&mut self, buffer_len: usize)->Result<(), std::io::Error> {

        let udp = self.udp.clone();
        let msg_map = self.msg_map.clone();
        let stop = self.stop.clone();

        let thread = thread::Builder::new()
            .name(String::from("thread_udp_listener"))
            .spawn( move || {
                while *stop.lock().unwrap() == false {
                    Self::try_recv(udp.clone(), msg_map.clone(), buffer_len);
            }})?;

        self.thread = Some(thread);
        return Ok(())
    }

    /// Safely closes the background thread. Automatically called when struct is dropped.
    fn stop(&mut self){
        *self.stop.lock().unwrap() = true;
        self.thread.take().map(thread::JoinHandle::join);
    }

    /// Attempts to receive a datagram from the underyling socket. 
    /// 
    /// Attempts to receive a datagram from the underlying socket and remove it from the queue.
    /// If no datagram is available, it will either return, or sit and wait depending on if the 
    /// the value of non_blocking, set with the Builder struct.
    /// 
    /// # Errors
    /// 
    /// Errors when the there is an issue receiving data from the underyling socket. 
    /// Does not return an error, prints the error to the command line.
    /// 
    /// # Panics
    /// 
    /// This will panic if the lock becomes poisioned.
    fn try_recv(udp: Arc<UdpSocket>, msg_map: Arc<MsgStorage>, buffer_len: usize) {
        let mut buffer: Vec<u8> = vec![0; buffer_len];

        let (num_bytes, addr) =  match udp.recv_from(&mut buffer) {
            Ok(n) => n ,
            Err(e)=> {
                if e.kind() == ErrorKind::WouldBlock {} //Unix response when non_blocking is true
                else if e.kind() == ErrorKind::TimedOut {}//Windows Response when non_blocking is true
                else {println!("{}",e);} //Prints this to screen instead of crashing for one fail read

                return; } //Break out of function if we received no bytes
        };

        buffer.truncate(num_bytes);
        let id: Vec<_> = buffer.drain(..8).collect();
        let id = BigEndian::read_u64(&id);

        msg_map.add_msg(id, addr, buffer);
    }

    /// Provides the oldest datagram of the specified type, if one exists. 
    /// 
    /// Attempts to retrieve the serialized object from the underlying storage depending
    /// on the requested data type. If a serialized object is available, it is removed from
    /// the underyling storage and and attempt to deserialize the data is made. If successful,
    /// the deserialized object is returned to the user.
    /// 
    /// # Errors
    /// 
    /// Returns error when the underlying storage is empty or the data could not be deserialized.
    /// 
    /// # Panics
    /// 
    /// This will panic if the lock becomes poisioned.
    pub fn get<J>(&self)->Result<(SocketAddr, J), std::io::Error>
        where J: de::DeserializeOwned + 'static
    {
        return self.msg_map.get_obj::<T,J>();
    }

    /// Provides all datagrams of the specified type, if any exist.
    /// 
    /// Attempts to retrieve all serialized objects from the underlying storage depending
    /// on the requested data type. If objects are available, they are removed from
    /// the underyling storage and and attempt to deserialize the data is made. If successful,
    /// the deserialized objects are returned to the user in the form of a Vector.
    /// 
    /// # Errors
    /// 
    /// Returns an error when the underlying storage is empty or the data could not be deserialized
    /// 
    /// # Panics
    /// 
    /// This will panic if the lock becomes poisioned.
    pub fn get_all<J>(&self)->Result<Vec<(std::net::SocketAddr, J)>, std::io::Error>
        where J: de::DeserializeOwned + 'static
    {
        return self.msg_map.get_obj_all::<T,J>();
    }

    /// Provides the oldest datagram of the specified type, if one exists, without
    /// removing it from the underlying storage.
    /// 
    /// Attempts to retrieve the serialized object from the underlying storage depending
    /// on the requested data type. If a serialized object is available, a copy is taken and
    /// an attempt to deserialize the data is made. If successful, the deserialized object is 
    /// returned to the user.
    /// 
    /// # Errors
    /// 
    /// Returns error when the underlying storage is empty or the data could not be deserialized.
    /// 
    /// # Panics
    /// 
    /// This will panic if the lock becomes poisioned.
    pub fn peek<J>(&self)->Result<(SocketAddr, J), std::io::Error>
        where J: de::DeserializeOwned + 'static
    {
        return self.msg_map.peek::<T,J>();
    }

    /// Deserializes the datagram, appends the ID, and sends to requested location.
    /// 
    /// Consumes a datagram and a destination address for the datagram to be sent to.
    /// An attempt to serialize the data is made; The datagram ID is prepended to the
    /// message and a request to the underlying UDP socket is made to send the data.
    /// 
    /// # Errors
    /// 
    /// Returns an error when the data could not be serialized or when the underyling 
    /// UDP socket failed to send the message.
    /// 
    /// # Panics
    /// 
    /// This will panic if the lock becomes poisioned.
    pub fn send<J: ser::Serialize + 'static, A: ToSocketAddrs>(&mut self, datagram: J, dest_addr: A)->Result<(),std::io::Error> {

        let mut wtr: Vec<u8> = vec![];
        let mut payload = match T::serial(&datagram) {
            Ok(obj) => obj,
            Err(_) => return Err(std::io::Error::new(ErrorKind::InvalidData, "Could not serialize"))
        };

        let id = self.msg_map.get_id::<J>();

        wtr.write_u64::<BigEndian>(id)?;
        wtr.append(&mut payload);

        self.udp.send_to(&wtr, dest_addr)?;

        return Ok(());
    }

    /// Allows the header id of a particular struct to be specified rather than be automatically generated.
    /// 
    /// Generally, the struct ID is automatically created using a hash of the TypeID. This method allows
    /// the struct id to be set by the user. This should be called before any attempt to send or receive
    /// a datagram is made. This is commonly used if interacting with a socket that does not use this crate
    /// and is expecting a specific ID for the type of message you are sending.
    /// 
    /// # Panics
    /// 
    /// This will panic if the lock becomes poisioned.
    pub fn set_id<F: 'static>(&self, id: u64) {
        self.msg_map.set_id(std::any::TypeId::of::<F>(), id);
    }
}

#[doc(hidden)]
struct MsgStorage {
    msgs: Mutex<HashMap<u64, VecDeque<(SocketAddr, Vec<u8>)>>>,
    ids: Mutex<HashMap<TypeId, u64>>
}

#[doc(hidden)]
impl MsgStorage {
    
    fn get_obj<T, J>(&self)->Result<(SocketAddr, J), std::io::Error> 
        where T: SerDesType, J: de::DeserializeOwned + 'static
    {
        
        let id = self.get_id::<J>();
        let mut msgs = self.msgs.lock().unwrap();

        match msgs.get_mut(&id) {
            Some(vec) => {
                match vec.pop_front() {
                    Some((addr, vec)) => {
                        match T::deserial(&vec){
                            Ok(obj) => {
                                return Ok((addr, obj))
                            },
                            Err(_) => return Err(std::io::Error::new(ErrorKind::InvalidData, "Could not be deserialized"))
                        }
                    },
                    None => return Err(std::io::Error::new(ErrorKind::NotFound, "Empty Vector"))
                }
            },
            None => Err(std::io::Error::new(ErrorKind::NotFound, "Empty Vector"))
        }
    }

    fn peek<T, J>(&self)->Result<(SocketAddr, J), std::io::Error> 
        where T: SerDesType, J: de::DeserializeOwned + 'static
    {
        let id = self.get_id::<J>();
        let mut msgs = self.msgs.lock().unwrap();

        match msgs.get_mut(&id) {
            Some(vec) => {
                match vec.front() {
                    Some((addr, vec)) => {
                        match T::deserial(&vec){
                            Ok(obj) => {
                                return Ok((*addr, obj))
                            },
                            Err(_) => return Err(std::io::Error::new(ErrorKind::InvalidData, "Could not be deserialized"))
                        }
                    },
                    None => return Err(std::io::Error::new(ErrorKind::NotFound, "Empty Vector"))
                }
            },
            None => Err(std::io::Error::new(ErrorKind::NotFound, "Empty Vector"))
        }
    }

    fn get_obj_all<T, J>(&self) -> Result<Vec<(SocketAddr, J)>, std::io::Error>
        where T: SerDesType, J: de::DeserializeOwned + 'static
    {
        let id = self.get_id::<J>();
        let mut msgs = self.msgs.lock().unwrap();

        match msgs.get_mut(&id) {
            Some(vec) => {
                let x: Vec<(SocketAddr, J)> = vec
                    .drain(..)
                    .into_iter()
                    .filter_map(|(addr, vec)| 
                    {
                        match T::deserial(&vec) 
                        {
                            Ok(obj) => return Some((addr, obj)),
                            Err(_) => return None
                        }  
                    })
                    .collect();
                
                    if x.len() > 0 {return Ok(x)}
                    return Err(std::io::Error::new(ErrorKind::InvalidData, "Could not Deserialize the data"))
                
            }
            None => Err(std::io::Error::new(ErrorKind::NotFound, "Empty Vector"))
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

    fn get_id<T>(&self)->u64 
        where T: 'static
    {
        
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

    fn calculate_hash<T>()->u64 
        where T: 'static
    {
        let mut hasher = hash_map::DefaultHasher::new();
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
