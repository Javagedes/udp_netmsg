#[cfg(test)]
mod struct_creation {
    use crate::serdes::JSON;
    use crate::managers::Builder;
    use crate::managers::manual::Datagram;
    use serde::{Serialize, Deserialize};
    use std::{thread, time};

    #[derive(Serialize, Deserialize)]
    struct UpdatePos {
        pub x: f32,
        pub y: f32,
        pub z: f32
    }

    impl Datagram for UpdatePos {
        fn header()->u32 {return 834227670}
    }

    #[test]
    fn test_manuel() {
        let mut net_msg = Builder::init().start_manuel::<JSON>();

        net_msg.register(UpdatePos::header());
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:39507")).unwrap();

        thread::sleep(time::Duration::from_millis(100));

        net_msg.get::<UpdatePos>().unwrap();
    }

    #[test]
    fn test_automatic() {
        let mut net_msg = Builder::init()
            .socket(String::from("0.0.0.0:50000"))
            .start_automatic::<JSON>();  
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:50000")).unwrap();
    
        thread::sleep(time::Duration::from_millis(100));

        net_msg.get::<UpdatePos>().unwrap();
    }
}

