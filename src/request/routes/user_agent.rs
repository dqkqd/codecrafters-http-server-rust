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
            Method::Get => match request.inner.first_value_content(b"User-Agent") {
                Some(user_agent) => (
                    Some(Status::OK),
                    vec![
                        ("Content-Type".into(), "text/plain".into()),
                        ("Content-Length".into(), user_agent.0.len().to_string()),
                    ],
                    user_agent.0,
                ),
                None => (None, vec![], vec![]),
            },
            _ => (None, vec![], vec![]),
        }
    }
}
