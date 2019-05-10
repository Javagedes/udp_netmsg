### Usage
If you're looking to easily send and receive Udp Messages then this crate is perfect for you. 
It gives you the ability to define your own Net Messages by simply creating a struct
and implementing a trait.

### Important to note:
- **This crate is actively being worked on and interfaces may change from update to update**
- This crate will automatically add the user defined header to the front of the datagram. the header id is 
defined in the fn id() function
- It can optionally include payload length (u32), can be important for target application implementation
- This crate is not for you if you are receiving or sending extremely large udp datagrams
as it uses vectors as a buffer. In my light testing, sending/receiving max size udp datagrams
(65k bytes), it is about 30% slower than if you were to use an array.
- This crate is designed to send datagrams in BigEndian. You decide how the buffer of data is built,
but the 4bytes of header is always as u32 BigEndian. I plan on changing this!
- The recv method returns a trait object *Message*, but you are able to downcast the message 
back to the original struct (see example below). I hope to simplify this!
- Currently can only send datagrams to a single ip/port
- If the buffer does not contain enough data for the datagram, **It will crash!** I plan on fixing this soon


If you have any suggestions for this crate, let me know! If something is not clear or confusing, let me know!

### Example
```rust
use udp_netmsg::{NetMessenger, Datagram};
use std::io::Cursor;
use byteorder::{BigEndian,ReadBytesExt, WriteBytesExt};

#[derive(Debug)]
struct UpdatePos {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Datagram for UpdatePos {
    fn from_buffer(mut buffer: Cursor<Vec<u8>>)->Box<Datagram> {
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

    fn header()->u32 {return 834227670}
}
fn main() {

    let source_ip = String::from("0.0.0.0:12000");
    let dest_ip = String::from("127.0.0.1:12000");
    let recv_buffer_size_bytes = 30;
    let mut net_msg = NetMessenger::new(
        source_ip,
        dest_ip,
        recv_buffer_size_bytes);

    //register the struct so it knows how to read datagram!
    net_msg.register(UpdatePos::header(), UpdatePos::from_buffer);

    match net_msg.send(Box::from(UpdatePos{id: 16, x: 5.0f32, y:5.0f32, z:5.0f32}), true) {
        Ok(_) => println!("datagram sent!"),
        Err(e) => println!("datagram failed to send because: {}", e)
    }

    //msg.recv(...) returns Some(datagram) or None
    match net_msg.recv(false, true) {
        //Some(Msg: Box<Datagram>)
        Some(msg) => {

            //now downcast to particular struct here
            if let Some(t) = msg.downcast_ref::<UpdatePos>() {
                println!("Received: [id {} at ({},{},{})]", t.id, t.x, t.y, t.z);}
            //else if let Some(t) = msg.downcast_ref::<Another msg type>() {}
        }
        None => {println!("no Datagram received!")}
    }
}
```

### To do 
- [ ] Check if buffer is large enough for datagram. Have it return None instead of crashing
- [ ] Allow users to choose bigEndian or littleEndian for header / payload length
- [ ] Simplify downcasting