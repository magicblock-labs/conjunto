use crate::{
    delegation_inconsistency::DelegationInconsistency,
    delegation_record::DelegationRecord,
};

pub enum DelegationAccount {
    Valid(DelegationRecord),
    Invalid(Vec<DelegationInconsistency>),
}
