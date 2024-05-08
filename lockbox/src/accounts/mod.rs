use async_trait::async_trait;
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::errors::LockboxResult;
pub(crate) mod predicates;

#[async_trait]
pub trait AccountProvider {
    async fn get_account(
        &self,
        pubkey: &Pubkey,
    ) -> LockboxResult<Option<Account>>;
}
