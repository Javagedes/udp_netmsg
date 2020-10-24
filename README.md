### Usage
If you're looking to easily send and receive Udp Messages then this crate is perfect for you. 
This gives you the ability to define your own Net Messages by simply creating a struct
and implementing a trait. 

### Important to note:
- **This crate is actively being worked on and interfaces may change from update to update**
- Handles network communication by prepending a header to identify the message type. This header tells the receiving end how to interpret the data.
- Supports both automatic header id creation and manuel header id creation.
- Relies on Serde for Serialization and Deserialization.
- Can Serialize/Deserialize any struct that implements Serde's Serialization & Deserialization traits.
- Any Format that is implemented for Serde works for this crate. The only format currently implemented within the crate (for user convenience) is JSON. More to come! (It's simple).
- Runs a listener on a second thread that listens for incoming Udp Messages.
- This crate is not for you if you are receiving or sending extremely large udp datagrams
as it uses a vector as a buffer. In my light testing, sending/receiving max size udp datagrams
(65k bytes), it is about 30% slower than if you were to use an array.

If you have any suggestions for this crate, let me know! If something is not clear or confusing, let me know!

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
        let mut net_msg = Builder::init().start_automatic::<JSON>();

        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:39507")).unwrap();

        thread::sleep(time::Duration::from_millis(100));

        net_msg.get::<UpdatePos>().unwrap();
    }
```

### To do 
- [ ] Split up the sending and receiving part of the socket so I don't have to lock the whole thing and makes responses extremely slow.
- [ ] Maybe? add functionality to just get the most recently received net message for when message order (of different structs) matter. 
- [ ] Allow user to decide what Endianness that data is sent as.
- [ ] Change the way messages are stored so that user can chose to either get the next message in time received or get it by message type. Issue with providing the data to the user. We can store it as just a long vec of (u32,SocketAddr, Vec<u8>). Then we can get them in order or filter them With maybe a map that only returns the correct ones based off the u32 identifier.