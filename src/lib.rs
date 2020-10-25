//! # User Datagram Protocol Network Messenger
//! 
//! If you're looking to easily send and receive Udp Messages then this crate is perfect for you. 
//! This gives you the ability to define your own Net Messages by simply creating a struct
//! and implementing a trait.
//! 
//! ## Formats
//!    
//! This crate supports any data format that is also supported by Serde. Not all
//! formats are implemented by this crate to reduce the size however they can be implemented by the user
//! via the SerDesType trait. See examples of the implementations [here]
//! 
//! [here]: https://github.com/Javagedes/udp_netmsg/blob/master/src/serdes.rs
//! 
//! Convenience Implementations:
//! - JSON
//! - Bincode
//! - YAML
//! 
//! ## Example
//! 
//! ```rust
//! use udp_netmsg::prelude::*;
//! use serde::{Serialize, Deserialize};
//! use std::{thread, time};
//! 
//! #[derive(Serialize, Deserialize)]
//! struct UpdatePos {
//!     pub x: f32,
//!     pub y: f32,
//!     pub z: f32
//! }
//! 
//! fn main() {
//!     let mut net_msg = Builder::init().start::<JSON>().unwrap(); 
//!     let pos = UpdatePos{x: 15f32, y: 15f32, z: 15f32};
//!     net_msg.send(pos, String::from("127.0.0.1:39507")).unwrap();
//! 
//!     thread::sleep(time::Duration::from_millis(100));
//! 
//!     net_msg.get::<UpdatePos>().unwrap();
//! }
//! ```

///Traits used for implementing SerDes formats and operations
pub mod serdes;

///UDP manager and associated methods
pub mod manager;

#[doc(hidden)]
pub mod prelude;

#[doc(hidden)]
mod util;
#[doc(hidden)]
mod tests;

