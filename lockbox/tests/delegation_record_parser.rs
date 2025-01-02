use conjunto_core::{
    delegation_record::{CommitFrequency, DelegationRecord},
    delegation_record_parser::DelegationRecordParser,
};
use conjunto_lockbox::delegation_record_parser_impl::DelegationRecordParserImpl;
use solana_sdk::pubkey;

#[test]
fn test_delegation_record_parser() {
    // NOTE: from delegation-program/tests/fixtures/accounts.rs
    let authority = pubkey!("CLMS5guJDje8BA9tQdd1wXmGmPx5S32yhGztw4xytAYN");
    let owner = pubkey!("CLMS5guJDje8BA9tQdd1wXmGmPx5S32yhGztw4xytAYN");
    let mut data = [0u8; size_of::<DelegationRecord>() + 8];
    dlp::state::DelegationRecord {
        authority,
        owner,
        delegation_slot: 4,
        commit_frequency_ms: 30_000,
        lamports: 500,
    }
    .to_bytes_with_discriminator(&mut data)
    .unwrap();
    let parser = DelegationRecordParserImpl;
    let record = parser.try_parse(&data).unwrap();
    assert_eq!(
        record,
        DelegationRecord {
            authority,
            owner,
            delegation_slot: 4,
            commit_frequency: CommitFrequency::Millis(30_000),
            lamports: 500,
        }
    );
}
