use casper_types::ApiError;

/// An error enum which can be converted to a `u16` so it can be returned as an `ApiError::User`.
#[repr(u16)]
pub enum Error {
    Unauthorized = 1,
    Unregistered = 2,
    InsufficientBalance = 3,
    InsufficientAmount = 4,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}
