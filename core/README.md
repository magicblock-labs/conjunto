
# Summary

This crate declares types and traits used across the whole codebase.
Doesn't contain any logic directly.

# Details

*Important symbols:*

- `GuideStrategy`/`RequestEndpoint` enums
  - Which endpoint to propagate a request to
  - can be Chain/Ephemeral/Both/Others

- `DelegationRecord` struct
  - Account owner's pubkey
  - `CommitFrequency` of the delegation's dump interval

- `AccountsHolder` trait
  - Writable/Readonly/Payer store for Pubkeys

- `AccountProvider` trait
  - get_account(Pubkey) -> Account

- `SignatureStatusProvider` trait
  - get_signature_status(Signature) -> Result

# Notes

This crate is supposed to be importable from everywhere.
It is not supposed to have any dependency
