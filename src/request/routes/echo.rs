use crate::{
    request::HandleRequest,
    spec::{request::Method, response::Status},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Echo {
    pub command: Vec<u8>,
}

impl HandleRequest for Echo {
    fn handle(
        &self,
        request: &crate::request::Request,
    ) -> (
        Option<crate::spec::response::Status>,
        crate::request::AdditionalHeader,
        crate::request::AdditionalBody,
    ) {
        match request.method() {
            Method::Get => (
                Some(Status::OK),
                vec![("Content-Type".into(), "text/plain".into())],
                self.command.to_vec(),
            ),
            _ => (None, vec![], vec![]),
        }
    }
}
