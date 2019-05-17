use std::io::{Cursor, Result, Write, BufRead, Read};

///Utility trait for writing a string to a buffer
///Strings in Rust are not null terminated but having null terminated strings can
///  be useful, if not necessary for sending strings over UDP
pub trait WriteString {

    /// Adds a \0 to the end of the string and writes it to a buffer
    #[inline]
    fn write_string(&mut self, s: String)-> Result<()>;
}

impl WriteString for Vec<u8> {

    /// Adds a \0 to the end of the string and writes it to a buffer
    #[inline]
    fn write_string(&mut self, mut s: String)-> Result<()>{
        s.insert(s.len(), '\0');
        self.write_all(s.as_bytes())
    }
}

///Utility trait for reading a string from a buffer
///Strings in Rust are not null terminated but having null terminated strings can
///  be useful, if not necessary for receiving strings over UDP
pub trait ReadString {

    ///Reads a string from starting point in cursor until a \0
    ///Includes the \0
    #[inline]
    fn read_string(&mut self)->Result<String>;
}

impl ReadString for Cursor<Vec<u8>> {

    ///Reads a string from starting point in cursor until a \0
    ///Includes the \0
    #[inline]
    fn read_string(&mut self)->Result<String> {

        //Create a vector to store the read bytes
        let mut mstr: Vec<u8> = Vec::new();
        self.read_until(0, &mut mstr).unwrap();

        //make it a cursor so we can use read_to_string
        let mut mstr : Cursor<Vec<u8> >= Cursor::new(mstr);
        let mut ip = String::new();
        mstr.read_to_string(&mut ip).unwrap();

        return Ok(ip);
    }
}