use dlp::pda::ephemeral_balance_pda_from_payer;
use serde::{Deserialize, Serialize};
use solana_sdk::{clock::Slot, pubkey::Pubkey};

use crate::account_chain_state::AccountChainState;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AccountChainSnapshot {
    pub pubkey: Pubkey,
    pub at_slot: Slot,
    pub chain_state: AccountChainState,
}

impl AccountChainSnapshot {
    pub fn ephemeral_balance_pda(pubkey: &Pubkey) -> Pubkey {
        ephemeral_balance_pda_from_payer(pubkey, 0)
    }
}
