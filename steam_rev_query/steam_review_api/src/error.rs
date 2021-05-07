use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    result,
};

#[derive(Debug, Clone, Copy)]
pub enum RevApiError {
    InvalidFilterCursor,
    InvalidFilterDayRange,
}

#[allow(dead_code)]
/// Convenience Result<T> type.
pub type Result<T> = result::Result<T, RevApiError>;

impl Error for RevApiError {}

impl Display for RevApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use RevApiError::*;
        write!(f, "{}", match *self {
            InvalidFilterCursor => "Cursors (for pagination) are only valid for Filter::Recent or Filter::Updated",
            InvalidFilterDayRange => "Day ranges are only allowed for Filter::All. You may need to manually call ReviewApi::filter."
        })
    }
}
