use std::fmt::{Debug, Display, Formatter};
#[derive(Debug)]
pub struct Error(anyhow::Error);

impl Error {
    pub fn new(msg: &'static str) -> Self {
        Self(anyhow::anyhow!(msg))
    }

    pub fn from_error(err: anyhow::Error) -> Self {
        Self(err)
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err = &self.0;
        write!(f, "{}", err)
    }
}

impl From<Error> for actix_web::Error {
    fn from(err: Error) -> Self {
        actix_web::error::ErrorInternalServerError(err)
    }
}

pub type Result<T> = actix_web::Result<T, Error>;
