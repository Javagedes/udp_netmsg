use downcast_rs::{Downcast,impl_downcast};
use byteorder::{BigEndian,WriteBytesExt,ReadBytesExt};

use std::net::UdpSocket;
use std::collections::HashMap;
use std::io::Cursor;
use std::string::ToString;

pub trait Message: Downcast {
    fn from_buffer(buffer: Cursor<Vec<u8>>)->Box<Message> where Self: Sized; // return Self not Message
    fn to_buffer(&self)->Vec<u8>;
    fn id()->u32 where Self: Sized;
    fn get_id(&self)->u32 where Self: Sized {
        Self::id()
    }
}
impl_downcast!(Message);

pub struct NetMessenger {
    udp: UdpSocket,
    target_ip: String,
    con_map: HashMap<u32, fn(Cursor<Vec<u8>>)->Box<Message>>,
    recv_buffer_size_bytes: u16,
}

impl NetMessenger {

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

    pub fn send<T: Message>(&self, msg: Box<T>)->Result<(),String> {

        let mut wtr: Vec<u8> = vec![];

        wtr.write_u32::<BigEndian>(msg.get_id()).unwrap();
        wtr.append(&mut msg.to_buffer());
        match self.udp.send_to(&wtr,&self.target_ip) {
            Ok(_) => return Ok(()),
            Err(e)=> return Err(e.to_string()),
        }
    }

    pub fn register(&mut self, key: u32, con: fn(Cursor<Vec<u8>>)->Box<Message> ) {
        self.con_map.insert(key,con);
    }
}