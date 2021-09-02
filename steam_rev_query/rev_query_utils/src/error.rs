use attohttpc::Error as RequestError;
use csv::{Error as CsvError, ErrorKind as CsvErrorKind};
use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    io::Error as IoError,
    result,
};
use steam_review_api::RevApiError;
use url::ParseError as UrlParseError;

pub type Result<T> = result::Result<T, Error>;

/// Error is a mess that holds every possible error that everything can return.
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    ReviewApi(RevApiError),
    MultipleAppids,
    NoDataAfterFiltering,
    Io(IoError),
    Csv(CsvError),
    UrlParse(UrlParseError),
    Request(RequestError),
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Error::ReviewApi(ref e) => Some(e),
            Error::MultipleAppids => None,
            Error::NoDataAfterFiltering => None,
            Error::Io(ref e) => Some(e),
            Error::Csv(ref e) => Some(e),
            Error::UrlParse(ref e) => Some(e),
            Error::Request(ref e) => Some(e),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Error::*;

        match self {
            ReviewApi(e) => e.fmt(f),
            MultipleAppids => write!(f, "Scraping multiple appids is unsupported."),
            NoDataAfterFiltering => write!(
                f,
                "No data were available to write after filtering for duplicates."
            ),
            Io(e) => e.fmt(f),
            Csv(e) => e.fmt(f),
            UrlParse(e) => e.fmt(f),
            Request(e) => e.fmt(f),
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
    #[inline]
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

impl From<UrlParseError> for Error {
    #[inline]
    fn from(error: UrlParseError) -> Self {
        Error::UrlParse(error)
    }
}

impl From<RequestError> for Error {
    #[inline]
    fn from(error: RequestError) -> Self {
        Error::Request(error)
    }
}
