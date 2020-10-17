use udp_netmsg::{NetMessenger, Datagram};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct UpdatePos {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub text: String
}

impl Datagram for UpdatePos {
    fn serial(&self)->Vec<u8> {
        return serde_json::to_vec(self).unwrap();
    }

    fn header()->u32 {return 834227670}
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateEntity {
    pub entity_type: String,
    pub location: (f32, f32, f32),
}

impl Datagram for CreateEntity {
    fn serial(&self)->Vec<u8> {
        return serde_json::to_vec(self).unwrap();
    }

    fn header()->u32 {return 505005}
}

fn main() {

    //source_ip and dest_ip are the same so we don't have to spin up a server and client
    let source_ip = String::from("0.0.0.0:12000");
    let dest_ip = String::from("127.0.0.1:12000");
    let recv_buffer_size_bytes = 100;
    let mut net_msg = NetMessenger::new(
        source_ip,
        dest_ip,
        recv_buffer_size_bytes);

    //register the structs so it knows how to read datagram!
    net_msg.register(UpdatePos::header());
    net_msg.register(CreateEntity::header());
    
    match net_msg.send(UpdatePos{x: 5.0f32, y:5.0f32, z:5.0f32, text: String::from("Hello How are you?")}) {
        Ok(_) => println!("datagram sent!"),
        Err(e) => println!("datagram failed to send because: {}", e)
    }

    match net_msg.send(CreateEntity{entity_type: String::from("Boss"), location: (50f32, 15f32, 17f32)}) {
        Ok(_) => println!("datagram sent!"),
        Err(e) => println!("datagram failed to send because: {}", e)
    }

    net_msg.recv(true);
    net_msg.recv(true);

    //notice that each message type is stored separately, so you can write handlers in your code that check for
    //specific message types.
    let create_entity_message: CreateEntity = net_msg.get().unwrap();
    let update_pos_message: UpdatePos = net_msg.get().unwrap();

    println!("{:?}", create_entity_message);
    println!("{:?}", update_pos_message);
}
