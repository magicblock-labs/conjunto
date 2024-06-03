
# Summary

Implements account and signature fetching traits using fetching on a RPC client

# Details

*Important symbols:*

- `RpcAccountProvider` struct
  - depends on a `RpcClient`
  - implements `AccountProvider` from core

- `RpcSignatureStatusProvider` struct
  - depends on a `RpcClient`
  - implements `SignatureStatusProvider` from core
  

# Notes


*Important dependencies:*

- Provides `AccountProvider` and `SignatureStatusProvider`: [core](../core/README.md) 
