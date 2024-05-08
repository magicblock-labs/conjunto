use crate::errors::TranswiseResult;
use conjunto_core::AccountProvider;
use conjunto_lockbox::{AccountLockState, AccountLockStateProvider};
use solana_sdk::{pubkey::Pubkey, transaction::SanitizedTransaction};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccountMeta {
    Writable {
        pubkey: Pubkey,
        lockstate: AccountLockState,
    },
    Readonly {
        pubkey: Pubkey,
    },
}

impl AccountMeta {
    pub fn readonly(pubkey: Pubkey) -> Self {
        AccountMeta::Readonly { pubkey }
    }

    pub async fn try_writable<T: AccountProvider>(
        pubkey: Pubkey,
        lockbox: &AccountLockStateProvider<T>,
    ) -> TranswiseResult<Self> {
        let lockstate = lockbox.try_lockstate_of_pubkey(&pubkey).await?;
        Ok(AccountMeta::Writable { pubkey, lockstate })
    }

    pub async fn account_metas_from_transaction<T: AccountProvider>(
        tx: &SanitizedTransaction,
        lockbox: &AccountLockStateProvider<T>,
    ) -> TranswiseResult<Vec<AccountMeta>> {
        let mut account_metas = Vec::new();
        let account_locks = tx.get_account_locks_unchecked();
        for pubkey in account_locks.readonly {
            account_metas.push(AccountMeta::readonly(*pubkey));
        }
        for pubkey in account_locks.writable {
            let account_meta =
                AccountMeta::try_writable(*pubkey, lockbox).await?;
            account_metas.push(account_meta);
        }

        Ok(account_metas)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use solana_sdk::message::{
        v0::{LoadedAddresses, LoadedMessage},
        Message, SanitizedMessage,
    };

    use super::*;

    fn create_sanitized_transactions(
        readonly: Vec<Pubkey>,
        writable: Vec<Pubkey>,
    ) -> SanitizedTransaction {
        let addresses = LoadedAddresses {
            readonly,
            writable,
            ..Default::default()
        };
        let message = SanitizedMessage::V0(LoadedMessage {
            loaded_addresses: Cow::Owned(addresses),
            message: Cow::Owned(Default::default()),
            is_writable_account_cache: Default::default(),
        });
        let tx = SanitizedTransaction {
            message,
            hash: Default::default(),
            is_simple_vote_tx: Default::default(),
            signatures: Default::default(),
        };
    }
}
