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
    let recv_buffer_size_bytes = 100;
    let mut net_msg = NetMessenger::new(source_ip,recv_buffer_size_bytes).unwrap();

    //register the structs so it knows how to read datagram!
    net_msg.register(UpdatePos::header());
    net_msg.register(CreateEntity::header());
    
    let dest_ip = String::from("127.0.0.1:12000");

    let new_entity = CreateEntity {entity_type: String::from("Boss"), location: (50f32, 15f32, 17f32)};

    match net_msg.send(new_entity, &dest_ip) {
        Ok(_) => println!("datagram sent!"),
        Err(e) => println!("datagram failed to send because: {}", e)
    }

    net_msg.recv(true);

    //notice that each message type is stored separately, so you can write handlers in your code that check for
    //specific message types.
    let (from_addr, create_entity_message) = net_msg.get::<CreateEntity>().unwrap();

    println!("Message Received!: {:?}", create_entity_message);

    let move_entity = UpdatePos {x: 16f32, y: 17f32, z: 20f32, text: String::from("Hello! I Moved")};

    match net_msg.send(move_entity, from_addr) {
        Ok(_) => println!("datagram sent!"),
        Err(e) => println!("datagram failed to send because: {}", e)
    }

    net_msg.recv(true);

    let (_, update_pos_message) = net_msg.get::<UpdatePos>().unwrap();

    println!("Message Received!: {:?}", update_pos_message);
}
