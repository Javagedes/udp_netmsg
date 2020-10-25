//! # User Datagram Protocol Network Messenger
//! 
//! If you're looking to easily send and receive Udp Messages then this crate is perfect for you. 
//! This gives you the ability to define your own Net Messages by simply creating a struct
//! and implementing a trait.
//! 
//! ## Formats
//!    
//! This crate supports any data format that is also supported by Serde. Not all
//! formats are implemented by this crate, but they can be implemented by the user
//! via the SerDesType trait.
//! 
//! Convenience Implementations:
//! - JSON
//! 


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

