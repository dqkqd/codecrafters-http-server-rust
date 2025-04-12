#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct HttpVersion {
    pub major: u32,
    pub minor: u32,
}
