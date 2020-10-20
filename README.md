### Usage
If you're looking to easily send and receive Udp Messages then this crate is perfect for you. 
This gives you the ability to define your own Net Messages by simply creating a struct
and implementing a trait. 

### Important to note:
- **This crate is actively being worked on and interfaces may change from update to update**
- Currently this crate only only deserializes in JSON format. Future updates will allow for the user to specify the serialization format from serde
- The get method allows you to decide what message you want to receive. Any message of the same type is stored in received order, however there is no way to tell the order in which different type messages are received. This is designed such that you can give shared access to the NetMessenger throughout your codebase and write different handlers for different messages in separate locations of the code base
- This crate runs a listener on a second thread that listens for Udp Messages
- This crate will automatically add the user defined header to the front of the datagram. the header id is 
defined in the fn id() function
- This crate is not for you if you are receiving or sending extremely large udp datagrams
as it uses a vector as a buffer. In my light testing, sending/receiving max size udp datagrams
(65k bytes), it is about 30% slower than if you were to use an array.

If you have any suggestions for this crate, let me know! If something is not clear or confusing, let me know!

### Example
```rust
use udp_netmsg::{NetMessenger, Datagram};
use serde::{Serialize, Deserialize};
use std::{thread, time};

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
    let mut net_msg = NetMessenger::new(source_ip).unwrap();

    //register the structs so it knows how to read datagram!
    net_msg.register(UpdatePos::header());
    net_msg.register(CreateEntity::header());
    
    let dest_ip = String::from("127.0.0.1:12000");

    let new_entity = CreateEntity {entity_type: String::from("Boss"), location: (50f32, 15f32, 17f32)};

    match net_msg.send(new_entity, &dest_ip) {
        Ok(_) => println!("datagram sent!"),
        Err(e) => println!("datagram failed to send because: {}", e)
    }

    net_msg.start();
    thread::sleep(time::Duration::from_millis(100));

    //notice that each message type is stored separately, so you can write handlers in your code that check for
    //specific message types.
    let (from_addr, create_entity_message) = net_msg.get::<CreateEntity>().unwrap();

    println!("Message Received!: {:?}", create_entity_message);

    let move_entity = UpdatePos {x: 16f32, y: 17f32, z: 20f32, text: String::from("Hello! I Moved")};

    match net_msg.send(move_entity, from_addr) {
        Ok(_) => println!("datagram sent!"),
        Err(e) => println!("datagram failed to send because: {}", e)
    }

    thread::sleep(time::Duration::from_millis(100));
    net_msg.stop();

    let (_, update_pos_message) = net_msg.get::<UpdatePos>().unwrap();

    println!("Message Received!: {:?}", update_pos_message);
}
```

### To do 
- [ ] Allow users to choose serialization / deserialization methods. Do this by taking a Trait? maybe with Fn()?
- [ ] Maybe? add functionality to just get the most recently received net message for when message order (of different structs) matter. 
- [ ] Create a builder method for the NetMessenger so that we can eventually have the netmessenger start immediately when built
- [ ] Set up the netmessenger such that it starts up during the ::Build() method and automatically stops itsself when it loses scope
- [ ] Allow user to decide what Endianness that data is sent as.
- [ ] Allow user to set the timeout time (lower timeout time = faster response to close, but more resources used)