/// This Trait must be implemented on any struct that you wish to use as a UDP datagram
pub trait Datagram {
    /// Called when sending a udp datagram
    /// This method should take the internal variables of the struct and convert them to bytecode (easy with the byteorder crate) and return the buffer
    fn serial(&self)->Vec<u8>;

    /// Static method used to define the header_id.
    fn header()->u32;

    /// method used to get the header from specific object
    fn get_header(&self)->u32 {
        return Self::header();
    }
}