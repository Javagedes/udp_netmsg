#[cfg(test)]
mod struct_creation {
    use crate::prelude::*;
    use serde::{Serialize, Deserialize};
    use std::{thread, time};

    #[derive(Serialize, Deserialize)]
    struct UpdatePos {
        pub x: f32,
        pub y: f32,
        pub z: f32
    }

    #[test]
    fn test_manual() {
        let mut net_msg = Builder::init().start::<JSON>().unwrap();

        net_msg.set_id::<UpdatePos>(505550550);
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:39507")).unwrap();

        thread::sleep(time::Duration::from_millis(100));

        net_msg.get::<UpdatePos>().unwrap();
    }

    #[test]
    fn test_automatic() {
        let mut net_msg = Builder::init()
            .socket(String::from("0.0.0.0:50000"))
            .start::<JSON>()
            .unwrap(); 
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50000")).unwrap();
    
        thread::sleep(time::Duration::from_millis(100));

        net_msg.get::<UpdatePos>().unwrap();
    }
}

