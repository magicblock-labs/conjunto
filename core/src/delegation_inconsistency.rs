use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DelegationInconsistency {
    AccountNotFound,
    AccountInvalidOwner,
    DelegationRecordNotFound,
    DelegationRecordInvalidOwner,
    DelegationRecordAccountDataInvalid(String),
}
