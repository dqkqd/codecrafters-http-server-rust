pub mod message;
pub mod protocol;
pub mod request;
pub mod response;

pub(crate) trait ToBytes {
    fn into_bytes(self) -> Vec<u8>;
}
