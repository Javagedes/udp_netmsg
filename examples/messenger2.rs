use udp_netmsg::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct Message {
    user: String,
    message: String,
}

fn main() 
{
    let manager = Builder::init()
        .socket(String::from("0.0.0.0:40062"))
        //.use_ids(false)
        .start::<JSON>().unwrap();

    let manager = Arc::from(manager);
    let man = manager.clone();

    std::thread::spawn(move || {
        let man = man.clone();

        loop {
            match man.get::<Message>() {
                Ok((_,obj)) => println!("{} says: {}", obj.user, obj.message),
                Err(_) => {}
            }
        }
    });

    loop {
        let mut message = String::new();
        std::io::stdin().read_line(&mut message).expect("Did not enter a correct string");

        let m = Message{user: String::from("User1"), message};

        manager.send(m, String::from("127.0.0.1:40061")).unwrap();
    }
}