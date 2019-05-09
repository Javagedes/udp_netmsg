###Usage
If you're looking to easily send Udp Messages then this crate is perfect for you. 
It gives you the ability to define your own Net Messages by simply creating a struct
and implementing a trait.

####Important to note:
- This crate will automatically add a message id to the head of the udp datagram. the header id is 
defined in the fn id() function(e.g. it sends [HEADER_ID,EVERYTHING_ELSE]. 
- This crate is not for you if you are receiving or sending extremely large udp datagrams
as it uses vectors as a buffer. In my light testing, sending/receiving max size udp datagrams
(65k bytes), it is about 30% slower than if you were to use an array.
- This crate is designed to send datagrams in BigEndian. You decide how the buffer of data is built,
but the 4bytes of header is always as u32 BigEndian
- The recv method returns a trait object *Message*, but you are able to downcast the message 
back to the original struct (see example below).

If you have any suggestions for this crate, let me know! If something is not clear or confusing, let me know!

###Example
```rust
use udp_netmsg::{NetMessenger, Message};
use std::io::Cursor;
use byteorder::{BigEndian,ReadBytesExt, WriteBytesExt};

#[derive(Debug)]
struct UpdatePos {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Message for UpdatePos {
    fn from_buffer(mut buffer: Cursor<Vec<u8>>)->Box<Message> {
        let id = buffer.read_u32::<BigEndian>().unwrap();
        let x = buffer.read_f32::<BigEndian>().unwrap();
        let y = buffer.read_f32::<BigEndian>().unwrap();
        let z = buffer.read_f32::<BigEndian>().unwrap();
        return Box::new(UpdatePos{id,x,y,z})
    }

    fn to_buffer(&self)->Vec<u8> {
        let mut wtr: Vec<u8> = Vec::new();
        //this buffer is 16 + 4 more for the header
        wtr.write_u32::<BigEndian>(self.id).unwrap();
        wtr.write_f32::<BigEndian>(self.x).unwrap();
        wtr.write_f32::<BigEndian>(self.y).unwrap();
        wtr.write_f32::<BigEndian>(self.z).unwrap();
        return wtr
    }

    //header
    fn id()->u32 {return 834227670}
}
fn main() {
    //port: port for receiving
    //target_ip: port for sending
    //recv_buffer_size_bytes: size of biggest datagram to be received
    let mut net_msg = NetMessenger::new(
        String::from("12000"),
        String::from("127.0.0.1:12000"),
        20);

    //register the struct so it knows how to read message!
    net_msg.register(UpdatePos::id(), UpdatePos::from_buffer);

    //Sends message. Returns Ok or Err
    match net_msg.send(Box::from(UpdatePos{id: 16, x: 5.0f32, y:5.0f32, z:5.0f32})) {
        Ok(_) => println!("message sent!"),
        Err(e) => println!("message failed to send because: {}", e)
    }

    //Set blocking or not
    //returns Some(Message) or None
    match net_msg.recv(false) {
        //Some(Msg: Box<Message>)
        Some(msg) => {
            //now downcast to particular struct here

            if let Some(t) = msg.downcast_ref::<UpdatePos>() {
                println!("id: {} at ({},{},{})", t.id, t.x, t.y, t.z);}
            //else if let Some(t) = msg.downcast_ref::<Another msg type>() {}
        }
        None => {println!("no message received!")}
    }
}
```
