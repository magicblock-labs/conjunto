
# Summary

Implements account and signature fetching traits using fetching on a RPC client

# Details

*Important symbols:*

- `RpcAccountProvider` struct
  - contains a `RpcClient`
  - implements `AccountProvider` from core

- `RpcSignatureStatusProvider` struct
  - contains a `RpcClient`
  - implements `SignatureStatusProvider` from core
  

# Notes

*Important dependencies:*

- Provides `AccountProvider` and `SignatureStatusProvider`: [core](../core/README.md) 
