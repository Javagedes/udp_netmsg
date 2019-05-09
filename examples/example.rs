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

