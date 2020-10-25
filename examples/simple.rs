use udp_netmsg::prelude::*;
use serde::{Serialize, Deserialize};
use std::{thread, time};

#[derive(Serialize, Deserialize, Debug)]
struct UpdatePos {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub text: String
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateEntity {
    pub entity_type: String,
    pub location: (f32, f32, f32),
}
fn main() {
    //source_ip and dest_ip are the same so we don't have to spin up a server and client
    let source_ip = String::from("0.0.0.0:12000");
    let net_msg = Builder::init().socket(source_ip).start::<JSON>().unwrap();
    
    let dest_ip = String::from("127.0.0.1:12000");

    let new_entity = CreateEntity {entity_type: String::from("Boss"), location: (50f32, 15f32, 17f32)};

    match net_msg.send(new_entity, &dest_ip) {
        Ok(_) => println!("datagram sent!"),
        Err(e) => println!("datagram failed to send because: {}", e)
    }

    thread::sleep(time::Duration::from_millis(100));

    let (from_addr, create_entity_message) = net_msg.get::<CreateEntity>().unwrap();

    println!("Message Received!: {:?}", create_entity_message);

    let move_entity = UpdatePos {x: 16f32, y: 17f32, z: 20f32, text: String::from("Hello! I Moved")};

    match net_msg.send(move_entity, from_addr) {
        Ok(_) => println!("datagram sent!"),
        Err(e) => println!("datagram failed to send because: {}", e)
    }

    thread::sleep(time::Duration::from_millis(100));

    let (_, update_pos_message) = net_msg.get::<UpdatePos>().unwrap();

    println!("Message Received!: {:?}", update_pos_message);
}
