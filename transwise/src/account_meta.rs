use crate::errors::TranswiseResult;
use conjunto_core::{AccountProvider, TransactionAccountsHolder};
use conjunto_lockbox::{AccountLockState, AccountLockStateProvider};
use solana_sdk::{pubkey::Pubkey, transaction::SanitizedTransaction};

// -----------------
// SanitizedTransactionAccountsHolder
// -----------------
pub struct SanitizedTransactionAccountsHolder {
    writable: Vec<Pubkey>,
    readonly: Vec<Pubkey>,
}

impl From<&SanitizedTransaction> for SanitizedTransactionAccountsHolder {
    fn from(tx: &SanitizedTransaction) -> Self {
        let loaded = tx.get_account_locks_unchecked();
        let writable = loaded.writable.iter().map(|x| **x).collect();
        let readonly = loaded.readonly.iter().map(|x| **x).collect();
        Self { writable, readonly }
    }
}
impl TransactionAccountsHolder for SanitizedTransactionAccountsHolder {
    fn get_writable(&self) -> Vec<Pubkey> {
        self.writable.clone()
    }
    fn get_readonly(&self) -> Vec<Pubkey> {
        self.readonly.clone()
    }
}

// -----------------
// AccountMeta
// -----------------
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

    pub async fn account_metas_from_sanitized_transaction<
        T: AccountProvider,
    >(
        tx: &SanitizedTransaction,
        lockbox: &AccountLockStateProvider<T>,
    ) -> TranswiseResult<Vec<AccountMeta>> {
        let tx_accounts = SanitizedTransactionAccountsHolder::from(tx);
        let account_metas = Self::account_metas(&tx_accounts, lockbox).await?;
        Ok(account_metas)
    }

    pub async fn account_metas<
        T: AccountProvider,
        U: TransactionAccountsHolder,
    >(
        tx: &U,
        lockbox: &AccountLockStateProvider<T>,
    ) -> TranswiseResult<Vec<AccountMeta>> {
        let mut account_metas = Vec::new();
        let readonly = tx.get_readonly();
        let writable = tx.get_writable();
        for pubkey in readonly.into_iter() {
            account_metas.push(AccountMeta::readonly(pubkey));
        }
        for pubkey in writable.into_iter() {
            let account_meta =
                AccountMeta::try_writable(pubkey, lockbox).await?;
            account_metas.push(account_meta);
        }

        Ok(account_metas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
