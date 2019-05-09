use downcast_rs::{Downcast,impl_downcast};
use byteorder::{BigEndian,WriteBytesExt,ReadBytesExt};

use std::net::UdpSocket;
use std::collections::HashMap;
use std::io::Cursor;
use std::string::ToString;

/// This Trait must be implemented on any struct that you wish to use as a UDP datagram
pub trait Datagram: Downcast {
    /// Called when a datagram with a matching header_id is received
    /// This method should use parse the bytes (easy with the byteorder crate) and return a copy of itself
    fn from_buffer(buffer: Cursor<Vec<u8>>)->Box<Datagram> where Self: Sized;
    /// Called when sending a udp datagram
    /// This method should take the internal variables of the struct and convert them to bytecode (easy with the byteorder crate) and return the buffer
    fn to_buffer(&self)->Vec<u8>;

    /// Static method used to define the header_id.
    fn header()->u32 where Self: Sized;

    /// An additional necessary method to make this crate work. Will not be touched by the user
    fn get_header(&self)->u32 where Self: Sized {
        Self::header()
    }
}
impl_downcast!(Datagram);

/// This is the struct that the user will use for sending and receiving datagrams
pub struct NetMessenger {
    /// The udp socket (e.g. 0.0.0.0:12000) is the ip & socket that datagrams are received on
    udp: UdpSocket,
    /// The ip/socket that all datagrams will be sent to
    dest_ip: String,
    /// This is used when receiving a datagram. The recv function pulls out the header from the
    /// buffer and uses it as a key to receive the constructor of the appropriate datagram
    con_map: HashMap<u32, fn(Cursor<Vec<u8>>)->Box<Datagram>>,
    /// The amount of bytes the buffer for receiving data should be.
    /// If your datagrams are huge, this will become slow as it uses vectors as the buffers
    /// about a 30% performance loss when sending / receiving the biggest datagrams (65k bytes)
    recv_buffer_size_bytes: u16,
}

impl NetMessenger {

    /// Simple Constructor for the class
    pub fn new(source_ip: String, dest_ip: String, recv_buffer_size_bytes: u16)->NetMessenger {

        let udp: UdpSocket = UdpSocket::bind(source_ip)
            .expect("Failed to bind to address for sending/receiving datagrams");

        udp.set_nonblocking(true).unwrap();
        let con_map: HashMap<u32, fn(Cursor<Vec<u8>>)->Box<Datagram>> = HashMap::new();

        NetMessenger {
            udp,
            dest_ip,
            con_map,
            recv_buffer_size_bytes,
        }
    }

    /// Checks to see if a datagram has been received. Returns None if it has not.
    /// If data has been received, it removes the header_id and gets the constructor for the
    /// matched datagram struct.
    /// If 'ignore_payload_len' = true, it won't pass payload len (u32) to the datagram constructor
    /// passes remaining buffer to specific datagram constructor and returns struct
    pub fn recv(&mut self, blocking: bool, ignore_payload_len: bool)->Option<Box<Datagram>> {

        if blocking {
            self.udp.set_nonblocking(false).unwrap();
        }

        let mut buffer: Vec<u8> = Vec::new();
        buffer.reserve(self.recv_buffer_size_bytes as usize);
        buffer = vec![0; self.recv_buffer_size_bytes as usize];

        let num_bytes =  match self.udp.recv(&mut buffer){
            Ok(n) => { n },
            Err(e)=> {
                println!("{}",e.to_string());
                return None; } //Break out of function if we received no bytes
        };

        buffer.resize(num_bytes,0);
        if blocking {self.udp.set_nonblocking(true).unwrap();}

        let mut rdr = Cursor::new(buffer);
        let id = rdr.read_u32::<BigEndian>().unwrap();
        if ignore_payload_len {
            rdr.read_u32::<BigEndian>().unwrap();
        }

        match self.con_map.get(&id) {
            Some(func) => {return Some(func(rdr));}
            None => {println!("unknown msg id: {}", id); return None}
        }
    }

    /// Takes in a struct that implements the datagram trait. Will call the to_buffer method and
    /// If 'include_header' = true: attach the header to the front and send the datagram
    /// If 'include_payload_len' = true: insert payload len (4 bytes) between header & payload
    /// **Incorrectly setting include_payload_len will produce no error but bad data!**
    pub fn send<T: Datagram>(&self, datagram: Box<T>, include_payload_len: bool)->Result<(),String> {

        let mut wtr: Vec<u8> = vec![];
        let mut payload = datagram.to_buffer();

        wtr.write_u32::<BigEndian>(datagram.get_header()).unwrap();

        if include_payload_len {
            wtr.write_u32::<BigEndian>(payload.len() as u32).unwrap();
        }

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
    pub fn register(&mut self, key: u32, con: fn(Cursor<Vec<u8>>)->Box<Datagram> ) {
        self.con_map.insert(key,con);
    }
}