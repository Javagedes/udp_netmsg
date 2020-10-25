use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use bincode;
use serde_yaml;



/// Implemented on empty struct when creating a new SerDes format (JSON, etc.)
pub trait SerDesType {
    type Error;

    fn serial<T: ?Sized + Serialize>(obj: &T) -> Result<Vec<u8>, Self::Error>;

    fn deserial<T: DeserializeOwned>(v: &'_ [u8]) -> Result<T, Self::Error>;
}

/// Convenience struct for SerDes operations using the JSON format
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

/// Convenience struct for SerDes Operations using the Bincode format
pub struct Bincode;
impl SerDesType for Bincode {
    type Error = bincode::Error;

    fn serial<T: ?Sized + Serialize>(obj: &T) -> Result<Vec<u8>, Self::Error> {
        return bincode::serialize(obj);
    }

    fn deserial<T: DeserializeOwned>(v: &'_ [u8])-> Result<T, Self::Error> {
        return bincode::deserialize(v);
    }
}

/// Convenience struct for SerDes Operations using the YAML format
pub struct YAML;
impl SerDesType for YAML {
    type Error = serde_yaml::Error;

    fn serial<T: ?Sized + Serialize>(obj: &T) -> Result<Vec<u8>, Self::Error> {
        return serde_yaml::to_vec(obj);
    }

    fn deserial<T: DeserializeOwned>(v: &'_ [u8])-> Result<T, Self::Error> {
        return serde_yaml::from_slice(v);
    }
}