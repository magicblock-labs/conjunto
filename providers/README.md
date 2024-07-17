
# Summary

Implements account and signature fetching traits using the RPC client

# Details

*Important symbols:*

- `RpcAccountProvider` struct
  - depends on a `RpcClient`
  - implements `AccountProvider` from core

- `RpcSignatureStatusProvider` struct
  - depends on a `RpcClient`
  - implements `SignatureStatusProvider` from core

- `RpcAccountWatcher` struct
  - depends on a `RpcClient`
  - implements `AccountWatcher` from core

# Notes

*Important dependencies:*

- Provides `AccountProvider`, `AccountWatcher` and `SignatureStatusProvider`: [core](../core/README.md)
