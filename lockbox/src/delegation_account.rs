use solana_sdk::account::Account;

use crate::{
    accounts::predicates::is_owned_by_delegation_program,
    delegation_inconsistency::DelegationInconsistency,
    delegation_record::DelegationRecord,
    delegation_record_parser::DelegationRecordParser, errors::LockboxResult,
};

pub enum DelegationAccount {
    Valid(DelegationRecord),
    Invalid(Vec<DelegationInconsistency>),
}

impl DelegationAccount {
    pub fn try_from_fetched_account<U: DelegationRecordParser>(
        delegation_fetched_account: Option<Account>,
        delegation_record_parser: &U,
    ) -> LockboxResult<DelegationAccount> {
        let delegation_account = match delegation_fetched_account {
            None => {
                return Ok(DelegationAccount::Invalid(vec![
                    DelegationInconsistency::AccountNotFound,
                ]))
            }
            Some(acc) => acc,
        };
        let mut inconsistencies = vec![];
        if !is_owned_by_delegation_program(&delegation_account) {
            inconsistencies.push(DelegationInconsistency::AccountInvalidOwner);
        }
        match delegation_record_parser.try_parse(&delegation_account.data) {
            Ok(delegation_record) => {
                if inconsistencies.is_empty() {
                    Ok(DelegationAccount::Valid(delegation_record))
                } else {
                    Ok(DelegationAccount::Invalid(inconsistencies))
                }
            }
            Err(err) => {
                inconsistencies.push(
                    DelegationInconsistency::RecordAccountDataInvalid(
                        err.to_string(),
                    ),
                );
                Ok(DelegationAccount::Invalid(inconsistencies))
            }
        }
    }
}
