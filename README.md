### Usage
If you're looking to easily send and receive Udp Messages then this crate is perfect for you. 
This gives you the ability to define your own Net Messages by simply creating a struct
and implementing a trait. 

### Important to note:
- **This crate is actively being worked on and interfaces may change from update to update**
- By default, prepends an id to the front of the datagram to identify message type. Simple to disable if needed.
- Supports both automatic id creation (for convenience) and manual header id creation (suggested).
- All methods are &self making it easy to use in a multi-threaded situation. Handles interior mutability with locks.
- Relies on Serde for Serialization and Deserialization.
- Can Serialize/Deserialize any struct that implements Serde's Serialization & Deserialization traits.
- Any format that is implemented for Serde works for this crate. For convenience, JSON, Bincode, and YAML are implemented within the crate (It's simple to implement others, see examples [here]).
- Runs a listener on a second thread that listens for incoming Udp Messages.
- This crate is not for you if you are receiving or sending extremely large udp datagrams
as it uses a vector as a buffer. In my light testing, sending/receiving max size udp datagrams
(65k bytes), it is about 30% slower than if you were to use an array.
[here]: https://github.com/Javagedes/udp_netmsg/blob/master/src/serdes.rs
If you have suggestions or questions for this crate, raise an [issue]!
[issue]: https://github.com/Javagedes/udp_netmsg/issues

### Example
```rust
    use udp_netmsg::prelude::*;
    use serde::{Serialize, Deserialize};
    use std::{thread, time};

    #[derive(Serialize, Deserialize)]
    struct UpdatePos {
        pub x: f32,
        pub y: f32,
        pub z: f32
    }

    fn main() {
        let net_msg = Builder::init().start::<JSON>().unwrap(); 
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:39507")).unwrap();
    
        thread::sleep(time::Duration::from_millis(100));

        net_msg.get::<UpdatePos>().unwrap();
    }
```

### To do 
- [ ] Split up the sending and receiving part of the socket so I don't have to lock the whole thing and makes responses slower
- [ ] Maybe? add functionality to just get the most recently received net message for when message order (of different structs) matter. 
- [ ] Change the way messages are stored so that user can chose to either get the next message in time received or get it by message type. Issue with providing the data to the user. We can store it as just a long vec of (u32,SocketAddr, Vec<u8>). Then we can get them in order or filter them With maybe a map that only returns the correct ones based off the u32 identifier.