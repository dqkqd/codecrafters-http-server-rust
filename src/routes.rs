#[derive(Debug)]
pub enum Route {
    Echo { command: String },
    Root,
    Unknown,
}

impl From<&str> for Route {
    fn from(value: &str) -> Route {
        let components = value.split('/').skip(1).collect::<Vec<_>>();
        match components.first() {
            Some(&"") => Route::Root,
            Some(&"echo") => Route::Echo {
                command: components[1..].join("/").to_string(),
            },
            _ => Route::Unknown,
        }
    }
}
