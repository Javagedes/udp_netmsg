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
#[derive(Serialize, Deserialize)]
    struct UpdatePos {
        pub x: f32,
        pub y: f32,
        pub z: f32
    }

    impl Datagram for UpdatePos {
        fn header()->u32 {return 834227670}
    }

    fn main() {
        let mut net_msg = Builder::<JSON>::new().start();

        net_msg.register(UpdatePos::header());
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:39507")).unwrap();

        net_msg.start();
        thread::sleep(time::Duration::from_millis(100));
        net_msg.stop();

        net_msg.get::<UpdatePos>().unwrap();
    }
```

### To do 
- [ ] Split up the sending and receiving part of the socket so I don't have to lock the whole thing and makes responses extremely slow.
- [ ] Maybe? add functionality to just get the most recently received net message for when message order (of different structs) matter. 
- [ ] Set up the netmessenger such that it starts up during the ::start() method of the builder helper and automatically stops itsself when it loses scope
- [ ] Allow user to decide what Endianness that data is sent as.
- [ ] Change the way messages are stored so that user can chose to either get the next message in time received or get it by message type. Issue with providing the data to the user. We can store it as just a long vec of (u32,SocketAddr, Vec<u8>). Then we can get them in order or filter them With maybe a map that only returns the correct ones based off the u32 identifier.