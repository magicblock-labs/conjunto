use conjunto_lockbox::{
    delegation_record::{CommitFrequency, DelegationRecord},
    delegation_record_parser::{
        DelegationRecordParser, DelegationRecordParserImpl,
    },
};
use solana_sdk::pubkey::Pubkey;

#[test]
fn test_delegation_record_parser() {
    // NOTE: from delegation-program/tests/fixtures/accounts.rs
    let delegation_record_account_data: [u8; 80] = [
        100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43, 85, 175, 207,
        195, 148, 154, 129, 218, 62, 110, 177, 81, 112, 72, 172, 141, 157, 3,
        211, 24, 26, 191, 79, 101, 191, 48, 19, 105, 181, 70, 132, 48, 117, 0,
        0, 0, 0, 0, 0,
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
