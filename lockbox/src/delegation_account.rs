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

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey::Pubkey;

    use crate::{CommitFrequency, DelegationRecordParserImpl};

    use super::*;

    #[test]
    fn test_delegation_record_parser() {
        // NOTE: from delegation-program/tests/fixtures/accounts.rs
        let delegation_record_account_data: [u8; 80] = [
            100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43, 85, 175,
            207, 195, 148, 154, 129, 218, 62, 110, 177, 81, 112, 72, 172, 141,
            157, 3, 211, 24, 26, 191, 79, 101, 191, 48, 19, 105, 181, 70, 132,
            48, 117, 0, 0, 0, 0, 0, 0,
        ];
        let parser = DelegationRecordParserImpl;
        let record = parser.try_parse(&delegation_record_account_data).unwrap();
        assert_eq!(
            record,
            DelegationRecord {
                owner: <Pubkey as std::str::FromStr>::from_str(
                    "3vAK9JQiDsKoQNwmcfeEng4Cnv22pYuj1ASfso7U4ukF"
                )
                .unwrap(),
                commit_frequency: CommitFrequency::Millis(30_000),
            }
        );
    }
}
