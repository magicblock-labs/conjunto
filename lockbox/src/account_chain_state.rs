use conjunto_core::{
    delegation_inconsistency::DelegationInconsistency,
    delegation_record::DelegationRecord,
};
use serde::{Deserialize, Serialize};
use solana_sdk::account::Account;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum AccountChainState {
    // For on-curve accounts
    Wallet {
        account: Account,
        // TODO - this would contain escrow information probably
    },
    /// The account is not delegated and therefore should
    /// not be used as writable on the ephemeral validator
    Undelegated {
        account: Account,
        delegation_inconsistency: DelegationInconsistency,
    },
    /// The account was found on chain in a proper delegated state which means we
    /// also found the related accounts like the buffer and delegation
    /// NOTE: commit records and state diff are not checked since an account
    /// is delegated and then used before the validator commits a state change.
    Delegated {
        account: Account,
        delegation_record: DelegationRecord,
    },
}

impl AccountChainState {
    pub fn is_wallet(&self) -> bool {
        matches!(self, AccountChainState::Wallet { .. })
    }
    pub fn is_undelegated(&self) -> bool {
        matches!(self, AccountChainState::Undelegated { .. })
    }
    pub fn is_delegated(&self) -> bool {
        matches!(self, AccountChainState::Delegated { .. })
    }
    pub fn account(&self) -> &Account {
        match self {
            AccountChainState::Wallet { account } => account,
            AccountChainState::Undelegated { account, .. } => account,
            AccountChainState::Delegated { account, .. } => account,
        }
    }
}
