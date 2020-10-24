/// Contains Udp Manager that allows user to choose the header value for datagrams
pub mod manual;
/// Contains Udp Manager that automatically chooses the header value for datagrams
pub mod automatic;

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
    pub fn start_manuel<T: SerDesType>(self)->manual::UdpManager<T> {
        let len = self.buffer_len;
        let mut man = manual::UdpManager::<T>::init(self).unwrap();
        man.start(len);

        return man;
    }

    pub fn start_automatic<T: SerDesType>(self)->automatic::UdpManager<T>{
        let len = self.buffer_len;
        let mut auto = automatic::UdpManager::<T>::init(self).unwrap();
        auto.start(len);

        return auto;
    }
}
