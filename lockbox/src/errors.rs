use thiserror::Error;

pub type LockboxResult<T> = std::result::Result<T, LockboxError>;

#[derive(Error, Debug)]
pub enum LockboxError {
    FailedToGetAccountFromCluster,
}
