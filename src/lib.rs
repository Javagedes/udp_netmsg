use downcast_rs::{Downcast,impl_downcast};
use byteorder::{BigEndian,WriteBytesExt,ReadBytesExt};

use std::net::UdpSocket;
use std::collections::HashMap;
use std::io::Cursor;
use std::string::ToString;

/// This Trait must be implemented on any struct that you wish to use as a UDP Message
pub trait Message: Downcast {
    /// Called when a udp with a matching header_id message is received
    /// This method should use parse the bytes (easy with the byteorder crate) and return a copy of itself
    fn from_buffer(buffer: Cursor<Vec<u8>>)->Box<Message> where Self: Sized; // return Self not Message
    /// Called when sending a udp message
    /// This method should take the internal variables of the struct and convert them to bytecode (easy with the byteorder crate) and return the buffer
    fn to_buffer(&self)->Vec<u8>;

    /// Static method used to define the header_id.
    fn id()->u32 where Self: Sized;

    /// An additional necessary method to make this crate work. Will not be touched by the user
    fn get_id(&self)->u32 where Self: Sized {
        Self::id()
    }
}
impl_downcast!(Message);

/// This is the struct that the user will use for sending and receiving datagrams
pub struct NetMessenger {
    /// The udp socket (e.g. 0.0.0.0:12000) is the ip & socket that messages are received on
    udp: UdpSocket,
    /// The ip/socket that all datagrams will be sent to
    target_ip: String,
    /// This is used when receiving a message. The recv function pulls out the header from the
    /// buffer and uses it as a key to receive the constructor of the appropriate message
    con_map: HashMap<u32, fn(Cursor<Vec<u8>>)->Box<Message>>,
    /// The amount of bytes the buffer for receiving data should be.
    /// If your udp messages are huge, this will become slow as it uses vectors as the buffers
    /// about a 30% performance loss when sending / receiving the biggest datagrams (65k bytes)
    recv_buffer_size_bytes: u16,
}

impl NetMessenger {

    /// Simple Constructor for the class
    pub fn new(port: String, target_ip: String, recv_buffer_size_bytes: u16)->NetMessenger {
        let mut ip = "0.0.0.0:".to_string();
        ip.push_str(&port);

        let udp: UdpSocket = UdpSocket::bind(ip)
            .expect("Failed to bind to address for sending/receiving messages");

        udp.set_nonblocking(true).unwrap();
        let con_map: HashMap<u32, fn(Cursor<Vec<u8>>)->Box<Message>> = HashMap::new();

        NetMessenger {
            udp,
            target_ip,
            con_map,
            recv_buffer_size_bytes,
        }
    }

    /// Checks to see if a datagram has been received. Returns None if it has not
    /// If data has been received, it pulls out the header_id to get the constructor for the
    /// necessary message type. Then builds the message using that constructor and returns it
    pub fn recv(&mut self, blocking: bool)->Option<Box<Message>> {

        if blocking {self.udp.set_nonblocking(false).unwrap();}
        let mut buffer: Vec<u8> = Vec::new();
        buffer.reserve(self.recv_buffer_size_bytes as usize);
        buffer = vec![0; self.recv_buffer_size_bytes as usize];
//        let mut buffer: [u8;65000] = [0;65000]; //Necessary if you are doing very large udp messages
//        Reads received data, if any... fills buffer // 65k (max) array takes 24k ns compared to 31k ns for vector
        let num_bytes =  match self.udp.recv(&mut buffer){
            Ok(n) => { n },
            _=> { return None; } //Break out of function if we received no bytes
        };

//        let buffer = buffer[..num_bytes].to_vec();
        buffer.resize(num_bytes,0);
        if blocking {self.udp.set_nonblocking(true).unwrap();}

        let mut rdr = Cursor::new(buffer);
        let id = rdr.read_u32::<BigEndian>().unwrap();
        match self.con_map.get(&id) {
            Some(func) => {return Some(func(rdr));}
            None => {println!("unknown msg id: {}", id); return None}
        }
    }

    /// Takes in a struct that implements the Message trait. Will call the to_buffer method and
    /// attach the header to the front and send the datagram
    pub fn send<T: Message>(&self, msg: Box<T>)->Result<(),String> {

        let mut wtr: Vec<u8> = vec![];

        wtr.write_u32::<BigEndian>(msg.get_id()).unwrap();
        wtr.append(&mut msg.to_buffer());
        match self.udp.send_to(&wtr,&self.target_ip) {
            Ok(_) => return Ok(()),
            Err(e)=> return Err(e.to_string()),
        }
    }

    /// Must register the header_id with the appropriate constructor.
    /// To easily get the header id, use STRUCT_NAME::id().
    /// To easily get the constructor, use STRUCT_NAME::from_buffer
    /// Don't include the parentheses! We are passing in the function, not calling it!
    pub fn register(&mut self, key: u32, con: fn(Cursor<Vec<u8>>)->Box<Message> ) {
        self.con_map.insert(key,con);
    }
}