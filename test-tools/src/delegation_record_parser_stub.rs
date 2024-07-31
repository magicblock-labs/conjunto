#[derive(Default)]
pub struct DelegationRecordParserStub {
    next_record: Option<DelegationRecord>,
}

impl DelegationRecordParser for DelegationRecordParserStub {
    fn try_parse(&self, _data: &[u8]) -> LockboxResult<DelegationRecord> {
        match self.next_record {
            Some(ref record) => Ok(record.clone()),
            None => Err(LockboxError::FailedToParseDelegationRecord(
                "Test error".to_string(),
            )),
        }
    }
}

impl DelegationRecordParserStub {
    pub fn new(record: Option<DelegationRecord>) -> Self {
        Self {
            next_record: record,
        }
    }
    pub fn set_next_record(&mut self, record: DelegationRecord) {
        self.next_record = Some(record);
    }
}
