use std::ops::Deref;

use conjunto_core::{AccountProvider, AccountsHolder, DelegationRecordParser};
use conjunto_lockbox::{AccountLockState, AccountLockStateProvider};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::{self, Pubkey},
    transaction::{SanitizedTransaction, VersionedTransaction},
};

use crate::{
    errors::TranswiseResult,
    transaction_accounts_holder::TransactionAccountsHolder,
    validated_accounts::{
        ValidatedDelegatedAccount, ValidatedUndelegatedAccount,
    },
};

// TODO(vbrunet) - this abbreviation is a bit confusing, TransactionAccountMeta
// -----------------
// TransAccountMeta
// -----------------
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum TransAccountMeta {
    Readonly {
        pubkey: Pubkey,
        lockstate: AccountLockState,
    },
    Writable {
        pubkey: Pubkey,
        lockstate: AccountLockState,
        is_payer: bool,
    },
}

impl TransAccountMeta {
    pub async fn try_readonly<T: AccountProvider, U: DelegationRecordParser>(
        pubkey: Pubkey,
        lockbox: &AccountLockStateProvider<T, U>,
    ) -> TranswiseResult<Self> {
        let lockstate = lockbox.try_lockstate_of_pubkey(&pubkey).await?;
        Ok(TransAccountMeta::Readonly { pubkey, lockstate })
    }

    pub async fn try_writable<T: AccountProvider, U: DelegationRecordParser>(
        pubkey: Pubkey,
        lockbox: &AccountLockStateProvider<T, U>,
        payer: &Pubkey,
    ) -> TranswiseResult<Self> {
        let lockstate = lockbox.try_lockstate_of_pubkey(&pubkey).await?;
        let is_payer = pubkey == *payer;
        Ok(TransAccountMeta::Writable {
            pubkey,
            lockstate,
            is_payer,
        })
    }

    pub fn pubkey(&self) -> &Pubkey {
        match self {
            TransAccountMeta::Readonly { pubkey, .. } => pubkey,
            TransAccountMeta::Writable { pubkey, .. } => pubkey,
        }
    }

    pub fn lockstate(&self) -> &AccountLockState {
        match self {
            TransAccountMeta::Readonly { lockstate, .. } => lockstate,
            TransAccountMeta::Writable { lockstate, .. } => lockstate,
        }
    }

    pub fn is_payer(&self) -> bool {
        matches!(self, TransAccountMeta::Writable { is_payer: true, .. })
    }

    pub fn is_program(&self) -> bool {
        matches!(
            self,
            TransAccountMeta::Readonly {
                lockstate: AccountLockState::Unlocked { is_program: true },
                ..
            }
        )
    }
}

// -----------------
// Endpoint
// -----------------
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Endpoint {
    Chain(TransAccountMetas),
    Ephemeral(TransAccountMetas),
    Unroutable {
        account_metas: TransAccountMetas,
        reason: UnroutableReason,
    },
}

impl Endpoint {
    pub fn is_ephemeral(&self) -> bool {
        matches!(self, Endpoint::Ephemeral(_))
    }
    pub fn is_chain(&self) -> bool {
        matches!(self, Endpoint::Chain(_))
    }
    pub fn is_unroutable(&self) -> bool {
        matches!(self, Endpoint::Unroutable { .. })
    }
    pub fn into_account_metas(self) -> TransAccountMetas {
        use Endpoint::*;
        match self {
            Chain(account_metas)
            | Ephemeral(account_metas)
            | Unroutable { account_metas, .. } => account_metas,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnroutableReason {
    InconsistentLocksEncountered {
        inconsistent_pubkeys: Vec<Pubkey>,
    },
    BothLockedAndUnlocked {
        writable_delegated_pubkeys: Vec<Pubkey>,
        writable_undelegated_non_payer_pubkeys: Vec<Pubkey>,
    },
}

// -----------------
// TransAccountMetas
// -----------------
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct TransAccountMetas(pub Vec<TransAccountMeta>);

impl Deref for TransAccountMetas {
    type Target = Vec<TransAccountMeta>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TransAccountMetas {
    pub async fn from_versioned_transaction<
        T: AccountProvider,
        U: DelegationRecordParser,
    >(
        tx: &VersionedTransaction,
        lockbox: &AccountLockStateProvider<T, U>,
    ) -> TranswiseResult<Self> {
        let tx_accounts = TransactionAccountsHolder::try_from(tx)?;
        Self::from_accounts_holder(&tx_accounts, lockbox).await
    }

    pub async fn from_sanitized_transaction<
        T: AccountProvider,
        U: DelegationRecordParser,
    >(
        tx: &SanitizedTransaction,
        lockbox: &AccountLockStateProvider<T, U>,
    ) -> TranswiseResult<Self> {
        let tx_accounts = TransactionAccountsHolder::try_from(tx)?;
        Self::from_accounts_holder(&tx_accounts, lockbox).await
    }

    pub async fn from_accounts_holder<
        T: AccountProvider,
        U: AccountsHolder,
        V: DelegationRecordParser,
    >(
        holder: &U,
        lockbox: &AccountLockStateProvider<T, V>,
    ) -> TranswiseResult<Self> {
        let mut account_metas = Vec::new();
        let readonly = holder.get_readonly();
        let writable = holder.get_writable();
        for pubkey in readonly.into_iter() {
            account_metas
                .push(TransAccountMeta::try_readonly(pubkey, lockbox).await?);
        }
        for pubkey in writable.into_iter() {
            let account_meta = TransAccountMeta::try_writable(
                pubkey,
                lockbox,
                holder.get_payer(),
            )
            .await?;
            account_metas.push(account_meta);
        }
        Ok(Self(account_metas))
    }

    pub fn into_endpoint(self) -> Endpoint {
        use Endpoint::*;
        use UnroutableReason::*;

        // If any account is in a bugged delegation state, we can't do anything
        let inconsistent_pubkeys = self.inconsistent_pubkeys();
        if !inconsistent_pubkeys.is_empty() {
            return Unroutable {
                account_metas: self,
                reason: InconsistentLocksEncountered {
                    inconsistent_pubkeys,
                },
            };
        }

        // If there are no writable delegated account in the transaction, we can route to chain
        let writable_delegated_pubkeys = self.writable_delegated_pubkeys();
        if writable_delegated_pubkeys.is_empty() {
            return Chain(self);
        }

        let writable_undelegated_non_payer_pubkeys =
            self.writable_undelegated_non_payer_pubkeys();

        // If we got here, we are planning to route to ephemeral,
        // so there cannot be any writable undelegated except the payer
        // If there are, we cannot route this transaction
        let has_writable_undelegated_non_payer =
            !writable_undelegated_non_payer_pubkeys.is_empty();
        if has_writable_undelegated_non_payer {
            return Unroutable {
                account_metas: self,
                reason: BothLockedAndUnlocked {
                    writable_delegated_pubkeys,
                    writable_undelegated_non_payer_pubkeys,
                },
            };
        }

        // If we got here, we only have delegated writables
        // or payers that are writable
        // So we can route to ephemeral
        Ephemeral(self)
    }

    pub fn undelegated_accounts(&self) -> Vec<ValidatedUndelegatedAccount> {
        self.iter()
            .flat_map(|x| match x.lockstate() {
                AccountLockState::NewAccount {} => {
                    Some(ValidatedUndelegatedAccount {
                        pubkey: *x.pubkey(),
                        is_program: false,
                        is_new: true,
                    })
                }
                AccountLockState::Unlocked { is_program } => {
                    Some(ValidatedUndelegatedAccount {
                        pubkey: *x.pubkey(),
                        is_program: *is_program,
                        is_new: false,
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn delegated_accounts(&self) -> Vec<ValidatedDelegatedAccount> {
        self.iter()
            .flat_map(|x| match x.lockstate() {
                AccountLockState::Locked { config, .. } => {
                    Some(ValidatedDelegatedAccount {
                        pubkey: *x.pubkey(),
                        lock_config: config.clone(),
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn payer_pubkey(&self) -> Pubkey {
        self.iter().find(|x| x.is_payer()).map(f) // TODO(vbrunet) - finish this
    }

    pub fn inconsistent_pubkeys(&self) -> Vec<Pubkey> {
        self.iter()
            .filter(|x| match x {
                TransAccountMeta::Writable { lockstate, .. } => {
                    lockstate.is_inconsistent()
                }
                TransAccountMeta::Readonly { lockstate, .. } => {
                    lockstate.is_inconsistent()
                }
            })
            .map(|x| *x.pubkey())
            .collect()
    }

    pub fn writable_delegated_pubkeys(&self) -> Vec<Pubkey> {
        self.iter()
            .filter(|x| match x {
                TransAccountMeta::Writable { lockstate, .. } => {
                    lockstate.is_locked()
                }
                _ => false,
            })
            .map(|x| *x.pubkey())
            .collect()
    }

    pub fn writable_undelegated_non_payer_pubkeys(&self) -> Vec<Pubkey> {
        self.iter()
            .filter(|x| match x {
                TransAccountMeta::Writable {
                    is_payer: false,
                    lockstate,
                    ..
                } if !lockstate.is_locked() => true,
                _ => false,
            })
            .map(|x| *x.pubkey())
            .collect()
    }
}
