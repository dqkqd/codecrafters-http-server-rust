mod echo;
mod files;
mod root;
mod user_agent;

use crate::spec::{request::RequestURI, response::Status};
use echo::Echo;
use files::Files;
use root::Root;
use user_agent::UserAgent;

use super::HandleRequest;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Route {
    Echo(Echo),
    UserAgent(UserAgent),
    Files(Files),
    Root(Root),
    Unknown,
}

impl HandleRequest for Route {
    fn handle(
        &self,
        request: &crate::request::Request,
    ) -> (
        Option<crate::spec::response::Status>,
        super::AdditionalHeader,
        super::AdditionalBody,
    ) {
        match self {
            Route::Echo(echo) => echo.handle(request),
            Route::UserAgent(user_agent) => user_agent.handle(request),
            Route::Files(files) => files.handle(request),
            Route::Root(root) => root.handle(request),
            Route::Unknown => (Some(Status::NotFound), vec![], vec![]),
        }
    }
}

impl From<&RequestURI> for Route {
    fn from(value: &RequestURI) -> Self {
        let mut components = value.0.split(|v| v == &b'/').skip(1);
        match components.next() {
            Some(b"") => Route::Root(Root),
            Some(b"echo") => Route::Echo(Echo {
                command: components.flatten().cloned().collect(),
            }),
            Some(b"user-agent") => Route::UserAgent(UserAgent),
            Some(b"files") => Route::Files(Files {
                filename: components.flatten().cloned().collect(),
            }),
            _ => Route::Unknown,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::spec::request::RequestURI;

    #[test]
    fn root() {
        assert_eq!(Route::from(&RequestURI(b"/".into())), Route::Root(Root));
    }

    #[test]
    fn echo() {
        assert_eq!(
            Route::from(&RequestURI(b"/echo/something".into())),
            Route::Echo(Echo {
                command: b"something".into()
            })
        );
    }

    #[test]
    fn user_agent() {
        assert_eq!(
            Route::from(&RequestURI(b"/user-agent".into())),
            Route::UserAgent(UserAgent),
        );
    }

    #[test]
    fn files() {
        assert_eq!(
            Route::from(&RequestURI(b"/files/foo".into())),
            Route::Files(Files {
                filename: b"foo".into()
            }),
        );
    }

    #[test]
    fn unknown() {
        assert_eq!(
            Route::from(&RequestURI(b"/something".into())),
            Route::Unknown
        );
    }
}
