use crate::{
    request::HandleRequest,
    spec::{request::Method, response::Status},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Root;

impl HandleRequest for Root {
    fn handle(
        &self,
        request: &crate::request::Request,
    ) -> (
        Option<crate::spec::response::Status>,
        crate::request::AdditionalHeader,
        crate::request::AdditionalBody,
    ) {
        match request.method() {
            Method::Get => (Some(Status::OK), vec![], vec![]),
            _ => (None, vec![], vec![]),
        }
    }
}
