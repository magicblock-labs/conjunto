use conjunto_core::{
    delegation_record_parser::DelegationRecordParser, AccountProvider,
    AccountsHolder,
};
use conjunto_lockbox::{
    account_chain_snapshot::AccountChainSnapshotProvider,
    account_chain_snapshot_shared::AccountChainSnapshotShared,
};
use futures_util::future::{try_join, try_join_all};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::{
    errors::TranswiseResult,
    transaction_accounts_holder::TransactionAccountsHolder,
};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TransactionAccountsSnapshot {
    pub readonly: Vec<AccountChainSnapshotShared>,
    pub writable: Vec<AccountChainSnapshotShared>,
    pub payer: Pubkey,
}

impl TransactionAccountsSnapshot {
    pub async fn from_accounts_holder<
        T: AccountProvider,
        V: DelegationRecordParser,
    >(
        holder: &TransactionAccountsHolder,
        account_chain_snapshot_provider: &AccountChainSnapshotProvider<T, V>,
    ) -> TranswiseResult<Self> {
        // Fully parallelize snapshot fetching using join(s)
        let (readonly, writable) = try_join(
            try_join_all(holder.get_readonly().iter().map(|pubkey| {
                account_chain_snapshot_provider
                    .try_fetch_chain_snapshot_of_pubkey(pubkey)
            })),
            try_join_all(holder.get_writable().iter().map(|pubkey| {
                account_chain_snapshot_provider
                    .try_fetch_chain_snapshot_of_pubkey(pubkey)
            })),
        )
        .await?;
        Ok(Self {
            readonly: readonly
                .into_iter()
                .map(AccountChainSnapshotShared::from)
                .collect(),
            writable: writable
                .into_iter()
                .map(AccountChainSnapshotShared::from)
                .collect(),
            payer: *holder.get_payer(),
        })
    }

    pub fn writable_inconsistent_pubkeys(&self) -> Vec<Pubkey> {
        self.writable
            .iter()
            .filter(|chain_snapshot| {
                chain_snapshot.chain_state.is_inconsistent()
            })
            .map(|chain_snapshot| chain_snapshot.pubkey)
            .collect()
    }

    pub fn writable_delegated_pubkeys(&self) -> Vec<Pubkey> {
        self.writable
            .iter()
            .filter(|chain_snapshot| chain_snapshot.chain_state.is_delegated())
            .map(|chain_snapshot| chain_snapshot.pubkey)
            .collect()
    }

    pub fn writable_undelegated_non_payer_pubkeys(&self) -> Vec<Pubkey> {
        self.writable
            .iter()
            .filter(|chain_snapshot| {
                !chain_snapshot.chain_state.is_delegated()
                    && (chain_snapshot.pubkey != self.payer)
            })
            .map(|chain_snapshot| chain_snapshot.pubkey)
            .collect()
    }

    pub fn writable_new_pubkeys(&self) -> Vec<Pubkey> {
        self.writable
            .iter()
            .filter(|chain_snapshot| chain_snapshot.chain_state.is_new())
            .map(|chain_snapshot| chain_snapshot.pubkey)
            .collect()
    }

    pub fn writable_payer_pubkeys(&self) -> Vec<Pubkey> {
        self.writable
            .iter()
            .filter(|chain_snapshot| chain_snapshot.pubkey == self.payer)
            .map(|chain_snapshot| chain_snapshot.pubkey)
            .collect()
    }
}
