use serde::de::DeserializeOwned;
use serde::ser::Serialize;

/// Implemented on any struct that will be sent or received. 
/// Allows the system to identify struct type when serializing and deserializing
pub trait Datagram {

    fn header()->u32;

    fn get_header(&self)->u32 {
        return Self::header();
    }
}

/// Implemented on empty struct when creating a new SerDes format (JSON, etc.)
pub trait SerDesType {
    type Error;

    fn serial<T: ?Sized + Serialize>(obj: &T) -> Result<Vec<u8>, Self::Error>;

    fn deserial<T: DeserializeOwned>(v: &'_ [u8]) -> Result<T, Self::Error>;
}

/// Convenience struct for SerDes Operations using the JSON format
pub struct JSON;
impl SerDesType for JSON {
    type Error = serde_json::Error;

    fn serial<T: ?Sized + Serialize>(obj: &T) -> Result<Vec<u8>, Self::Error> {
        return serde_json::to_vec(obj);
    }

    fn deserial<T: DeserializeOwned>(v: &'_ [u8])-> Result<T, Self::Error> {
        return serde_json::from_slice(v);
    }
}