#[cfg(test)]
mod struct_creation {
    use crate::prelude::*;
    use crate::serdes::{Bincode, YAML};
    use serde::{Serialize, Deserialize};
    use std::{thread, time};

    #[derive(Serialize, Deserialize)]
    struct UpdatePos {
        pub x: f32,
        pub y: f32,
        pub z: f32
    }

    #[derive(Serialize, Deserialize)]
    struct RenameObj {
        pub name: String
    }

    #[test]
    fn test_manual() {
        let mut net_msg = Builder::init().start::<JSON>().unwrap();

        net_msg.set_id::<UpdatePos>(505550550);
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        let name = RenameObj{name: String::from("Billy")};
        net_msg.send(name, String::from("127.0.0.1:39507")).unwrap();
        net_msg.send(pos, String::from("127.0.0.1:39507")).unwrap();
        
        thread::sleep(time::Duration::from_millis(100));

        net_msg.get::<UpdatePos>().unwrap();
    }

    #[test]
    fn automatic() {
        let mut net_msg = Builder::init()
            .socket(String::from("0.0.0.0:50000"))
            .start::<JSON>()
            .unwrap(); 

        let name = RenameObj{name: String::from("Billy")};
        net_msg.send(name, String::from("127.0.0.1:50000")).unwrap();
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50000")).unwrap();
    
        thread::sleep(time::Duration::from_millis(100));

        net_msg.get::<UpdatePos>().unwrap();
    }

    #[test]
    fn bincode_serdes() {
        let mut net_msg = Builder::init()
            .socket(String::from("0.0.0.0:50001"))
            .start::<Bincode>()
            .unwrap(); 

        let name = RenameObj{name: String::from("Billy")};
        net_msg.send(name, String::from("127.0.0.1:50001")).unwrap();
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50001")).unwrap();
    
        thread::sleep(time::Duration::from_millis(100));

        net_msg.get::<UpdatePos>().unwrap();
    }

    #[test]
    fn yaml_serdes() {
        
        let mut net_msg = Builder::init()
            .socket(String::from("0.0.0.0:50002"))
            .start::<YAML>()
            .unwrap(); 

        let name = RenameObj{name: String::from("Billy")};
        net_msg.send(name, String::from("127.0.0.1:50002")).unwrap();
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50002")).unwrap();

        thread::sleep(time::Duration::from_millis(100));

        net_msg.get::<UpdatePos>().unwrap();
    }

    #[test]
    fn get_multiple_at_once() {
        let mut net_msg = Builder::init()
            .socket(String::from("0.0.0.0:50003"))
            .start::<YAML>()
            .unwrap(); 

        let name = RenameObj{name: String::from("Billy")};
        net_msg.send(name, String::from("127.0.0.1:50003")).unwrap();
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50003")).unwrap();
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50003")).unwrap();
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50003")).unwrap();

        thread::sleep(time::Duration::from_millis(100));

        assert_eq!(net_msg.get_all::<UpdatePos>().unwrap().len(), 3);
    }

    #[test]
    fn peek() {
        let mut net_msg = Builder::init()
            .socket(String::from("0.0.0.0:50004"))
            .start::<YAML>()
            .unwrap(); 

        let name = RenameObj{name: String::from("Billy")};
        net_msg.send(name, String::from("127.0.0.1:50004")).unwrap();
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50004")).unwrap();
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50004")).unwrap();
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50004")).unwrap();

        thread::sleep(time::Duration::from_millis(100));

        net_msg.peek::<UpdatePos>().unwrap();

        assert_eq!(net_msg.get_all::<UpdatePos>().unwrap().len(), 3);
    }

    #[test]
    fn no_ids_succeed() {
        let mut net_msg = Builder::init()
            .socket(String::from("0.0.0.0:50005"))
            .use_ids(false)
            .start::<JSON>()
            .unwrap(); 

        let name = RenameObj{name: String::from("Billy")};
        net_msg.send(name, String::from("127.0.0.1:50005")).unwrap();

        thread::sleep(time::Duration::from_millis(100));

        net_msg.get::<RenameObj>().unwrap();
    }

    #[test]
    #[should_panic]
    fn no_ids_panic() {
        let mut net_msg = Builder::init()
            .socket(String::from("0.0.0.0:50006"))
            .use_ids(false)
            .start::<JSON>()
            .unwrap(); 

        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50006")).unwrap();
        let name = RenameObj{name: String::from("Billy")};
        net_msg.send(name, String::from("127.0.0.1:50006")).unwrap();

        thread::sleep(time::Duration::from_millis(100));

        net_msg.get::<RenameObj>().unwrap();
    }

    #[test]
    fn fail_des_keep_item() {
        let mut net_msg = Builder::init()
            .socket(String::from("0.0.0.0:50007"))
            .use_ids(false)
            .start::<JSON>()
            .unwrap(); 

        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50007")).unwrap();

        thread::sleep(time::Duration::from_millis(100));

        match net_msg.peek::<RenameObj>() {
            Ok(_) => panic!("Should not have serialized correctly"),
            Err(_) => {}
        }

        net_msg.peek::<UpdatePos>().unwrap();
    }

    #[test]
    fn remove_front() {
        let mut net_msg = Builder::init()
            .socket(String::from("0.0.0.0:50008"))
            .use_ids(false)
            .start::<JSON>()
            .unwrap(); 

        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50008")).unwrap();
        let name = RenameObj{name: String::from("Billy")};
        net_msg.send(name, String::from("127.0.0.1:50008")).unwrap();

        thread::sleep(time::Duration::from_millis(100));

        match net_msg.peek::<RenameObj>() {
            Ok(_) => panic!("Should not have serialized correctly"),
            Err(_) => {}
        }
        net_msg.remove_front::<RenameObj>().unwrap();
        net_msg.peek::<RenameObj>().unwrap();
    }

    #[test]
    fn remove_all() {
        let mut net_msg = Builder::init()
            .socket(String::from("0.0.0.0:50009"))
            .start::<YAML>()
            .unwrap(); 

        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50009")).unwrap();
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50009")).unwrap();
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50009")).unwrap();

        thread::sleep(time::Duration::from_millis(100));

        net_msg.remove_all::<UpdatePos>().unwrap();

        assert_eq!(net_msg.get_all::<UpdatePos>().unwrap().len(), 0);
    }
}

