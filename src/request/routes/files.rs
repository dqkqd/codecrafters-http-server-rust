use std::{
    fs,
    io::{Read, Write},
};

use crate::{
    request::HandleRequest,
    spec::{request::Method, response::Status},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Files {
    pub filename: Vec<u8>,
}

impl HandleRequest for Files {
    fn handle(
        &self,
        request: &crate::request::Request,
    ) -> (
        Option<crate::spec::response::Status>,
        crate::request::AdditionalHeader,
        crate::request::AdditionalBody,
    ) {
        match request.method() {
            Method::Get => {
                let directory = request
                    .cli_directory
                    .as_ref()
                    .expect("directory must be passed");

                if let Ok(Ok(mut file)) = String::from_utf8(self.filename.clone())
                    .map(|filename| directory.join(filename))
                    .map(fs::File::open)
                {
                    let mut content = vec![];
                    file.read_to_end(&mut content).expect("cannot read file");

                    (
                        Some(Status::OK),
                        vec![("Content-Type".into(), "application/octet-stream".into())],
                        content,
                    )
                } else {
                    (Some(Status::NotFound), vec![], vec![])
                }
            }
            Method::Post => {
                let directory = request
                    .cli_directory
                    .as_ref()
                    .expect("directory must be passed");
                if let Ok(filename) = String::from_utf8(self.filename.clone()) {
                    let file = directory.join(filename);
                    let mut file = fs::File::options()
                        .create_new(true)
                        .truncate(true)
                        .write(true)
                        .open(file)
                        .expect("cannot open / create file");
                    match &request.inner.body {
                        Some(body) => file.write_all(&body.0).expect("cannot write file"),
                        None => file.write_all(b"").expect("cannot write file"),
                    }
                    (Some(Status::Created), vec![], vec![])
                } else {
                    (Some(Status::NotFound), vec![], vec![])
                }
            }
            _ => (None, vec![], vec![]),
        }
    }
}
