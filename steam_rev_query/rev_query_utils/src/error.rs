use csv::{Error as CsvError, ErrorKind as CsvErrorKind};
use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    io::Error as IoError,
    result,
};
use steam_review_api::RevApiError;

pub type Result<T> = result::Result<T, Error>;

/// Error is a mess that holds every possible error that everything can return.
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    ReviewApi(RevApiError),
    MultipleAppids,
    Io(IoError),
    Csv(CsvError),
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Error::*;

        match self {
            ReviewApi(e) => e.fmt(f),
            MultipleAppids => write!(f, "Scraping multiple appids is unsupported."),
            Io(e) => e.fmt(f),
            Csv(e) => e.fmt(f),
        }
    }
}

// From impls
impl From<IoError> for Error {
    #[inline]
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}

impl From<RevApiError> for Error {
    fn from(error: RevApiError) -> Self {
        Error::ReviewApi(error)
    }
}

impl From<CsvError> for Error {
    fn from(error: CsvError) -> Self {
        // Convert Csv::Error to Error::Io (i.e. this type) if it's an IO Error.
        if error.is_io_error() {
            if let CsvErrorKind::Io(e) = error.into_kind() {
                Error::Io(e)
            } else {
                unreachable!()
            }
        } else {
            Error::Csv(error)
        }
    }
}
