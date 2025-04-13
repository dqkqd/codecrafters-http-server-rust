use crate::{
    request::HandleRequest,
    spec::{request::Method, response::Status},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct UserAgent;

impl HandleRequest for UserAgent {
    fn handle(
        &self,
        request: &crate::request::Request,
    ) -> (
        Option<crate::spec::response::Status>,
        crate::request::AdditionalHeader,
        crate::request::AdditionalBody,
    ) {
        match request.method() {
            Method::Get => match request.inner.find_value(b"User-Agent") {
                Some(user_agent) => (
                    Some(Status::OK),
                    vec![("Content-Type".into(), "text/plain".into())],
                    user_agent,
                ),
                None => (None, vec![], vec![]),
            },
            _ => (None, vec![], vec![]),
        }
    }
}
