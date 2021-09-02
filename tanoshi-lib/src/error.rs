use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("version has invalid format, shouldbe major.minor.patch")]
    InvalidVersion,
}
