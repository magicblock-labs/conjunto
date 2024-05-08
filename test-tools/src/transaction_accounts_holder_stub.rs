use conjunto_core::TransactionAccountsHolder;
use solana_sdk::pubkey::Pubkey;

pub struct TransactionAccountsHolderStub(Vec<Pubkey>, Vec<Pubkey>);
impl TransactionAccountsHolder for TransactionAccountsHolderStub {
    fn get_writable(&self) -> Vec<Pubkey> {
        self.0.clone()
    }
    fn get_readonly(&self) -> Vec<Pubkey> {
        self.1.clone()
    }
}
