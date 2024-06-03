
# Summary

This crate provides a few important capabilities

- Provides conversions between account information types
- Can compute an `Endpoint` from any account information types

# Details

*Important symbols:*

- `TransactionAccountsHolder` struct
  - Parsed transaction pubkey Vecs

- `TransactionAccountsExtractor` trait
  - allow conversion from solana transactions to `TransactionAccountsHolder`

- `ValidatedAccounts` struct
  - vec of `ValidatedReadonlyAccount`
  - vec of `ValidatedWritableAccount`
  - classified accounts with meta info and delegation state

- `ValidatedAccountsProvider` trait
  - allow conversions from solana transactions to `ValidatedAccounts`
  - allow conversion from `TransactionAccountsHolder` to `ValidatedAccounts`

- `TransAccountMeta` struct
  - enum of Writable or Readable
  - contains delegation state and meta info with a pubkey

- `TransAccountMetas` struct
  - vec of `TransAccountMeta`

- `Endpoint` enum
  - enum Chain or Ephemeral or Unroutable

- `Transwise` struct
  - implements `TransactionAccountsExtractor`
  - implements `ValidatedAccountsProvider`
  - contains a `AccountLockStateProvider`


*Important processes:*

- Solana transactions can be converted to `TransAccountMetas`
- Solana transactions can be converted to `TransactionAccountsHolder`

- Solana transactions can be converted to `ValidatedAccounts`
- `TransactionAccountsHolder` can be converted to `ValidatedAccounts`
- `TransAccountMetas` can be converted to `ValidatedAccounts`

- Solana transactions can be converted to `Endpoint`
- `TransAccountMetas` can be used to decide on an `Endpoint`

# Notes

Important dependencies:

- Provides `AccountLockStateProvider`: [lockbox](../lockbox/README.md) 
