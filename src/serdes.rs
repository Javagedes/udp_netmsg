use serde::de::DeserializeOwned;
use serde::ser::Serialize;

pub trait SerDesType {
    type Error;

    fn serial<T: ?Sized + Serialize>(obj: &T) -> Result<Vec<u8>, Self::Error>;

    fn deserial<T: DeserializeOwned>(v: &'_ [u8]) -> Result<T, Self::Error>;
}

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