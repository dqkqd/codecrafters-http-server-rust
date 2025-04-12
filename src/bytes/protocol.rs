use crate::spec::protocol::HttpVersion;

use super::ToBytes;

impl ToBytes for HttpVersion {
    fn into_bytes(self) -> Vec<u8> {
        format!("HTTP/{}.{}", self.major, self.minor).into_bytes()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn http_status() {
        assert_eq!(HttpVersion { major: 2, minor: 0 }.into_bytes(), b"HTTP/2.0");
    }
}
