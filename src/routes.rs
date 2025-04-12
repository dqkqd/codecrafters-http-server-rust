use crate::spec::request::RequestURI;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Route {
    Echo { command: Vec<u8> },
    UserAgent,
    Files { filename: Vec<u8> },
    Root,
    Unknown,
}

impl From<&RequestURI> for Route {
    fn from(value: &RequestURI) -> Self {
        let mut components = value.0.split(|v| v == &b'/').skip(1);
        match components.next() {
            Some(b"") => Route::Root,
            Some(b"echo") => Route::Echo {
                command: components.flatten().cloned().collect(),
            },
            Some(b"user-agent") => Route::UserAgent,
            Some(b"files") => Route::Files {
                filename: components.flatten().cloned().collect(),
            },
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
        assert_eq!(Route::from(&RequestURI(b"/".into())), Route::Root);
    }

    #[test]
    fn echo() {
        assert_eq!(
            Route::from(&RequestURI(b"/echo/something".into())),
            Route::Echo {
                command: b"something".into()
            }
        );
    }

    #[test]
    fn user_agent() {
        assert_eq!(
            Route::from(&RequestURI(b"/user-agent".into())),
            Route::UserAgent,
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
