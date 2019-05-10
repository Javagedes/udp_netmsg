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

