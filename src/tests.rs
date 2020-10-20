#[cfg(test)]
mod struct_creation {
    use crate::{NetMessenger, Datagram};
    use serde::{Serialize, Deserialize};
    use std::{thread, time};

    #[derive(Serialize, Deserialize)]
    struct UpdatePos {
        pub x: f32,
        pub y: f32,
        pub z: f32
    }

    impl Datagram for UpdatePos {

        fn serial(&self)->Vec<u8> {
            
            let vec = serde_json::to_vec(self).unwrap();

            return vec
        }

        fn header()->u32 {return 834227670}
        fn get_header(&self)->u32 { return 834227670 }
    }

    #[test]
    fn correct_source_dest() {
        let net_msg = NetMessenger::new(
            String::from("0.0.0.0:12000")
        ).unwrap();

        assert_eq!(net_msg.send(UpdatePos{x: 15f32, y: 15f32, z:15f32}, String::from("127.0.0.1:12000")).unwrap(), ());
    }

    #[test]
    #[should_panic]
    fn incorrect_source() {
        NetMessenger::new(
            String::from("12000")
        ).unwrap();
    }

    #[test]
    fn send_receive_data() {
        let mut net_msg = NetMessenger::new(
            String::from("0.0.0.0:12004")
        ).unwrap();

        net_msg.register(UpdatePos::header());
        let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
        net_msg.send(pos, String::from("127.0.0.1:12004")).unwrap();

        net_msg.start();
        thread::sleep(time::Duration::from_millis(100));
        net_msg.stop();

        net_msg.get::<UpdatePos>().unwrap();
    }
}

