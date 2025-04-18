use conjunto_core::{
    delegation_record::{CommitFrequency, DelegationRecord},
    delegation_record_parser::DelegationRecordParser,
    errors::{CoreError, CoreResult},
};

pub struct DelegationRecordParserImpl;

impl DelegationRecordParser for DelegationRecordParserImpl {
    fn try_parse(&self, data: &[u8]) -> CoreResult<DelegationRecord> {
        parse_delegation_record(data)
    }
}

fn parse_delegation_record(data: &[u8]) -> CoreResult<DelegationRecord> {
    // bytemuck fails with TargetAlignmentGreaterAndInputNotAligned when the data isn't
    // properly aligned. This happens sporadically depending on how the data was stored, i.e.
    // only in debug mode and is fine in release mode or if we add unrelated code before the call.
    // The below forces the data to be aligned since vecs are always aligned.
    // NOTE: I didn't find 100% confirmation that vecs are always correctly aligned, but
    // the issue I encountered was fixed by this change.
    // TODO(thlorenz): with this fix we copy data and should revisit this to avoid that
    let aligned_data = data[8..].to_vec();
    let state =
        bytemuck::try_from_bytes::<dlp::state::DelegationRecord>(&aligned_data)
            .map_err(|err| {
                CoreError::FailedToParseDelegationRecord(format!(
                    "Failed to deserialize DelegationRecord: {}",
                    err
                ))
            })?;
    Ok(DelegationRecord {
        authority: state.authority,
        owner: state.owner,
        delegation_slot: state.delegation_slot,
        commit_frequency: CommitFrequency::Millis(state.commit_frequency_ms),
        lamports: state.lamports,
    })
}
