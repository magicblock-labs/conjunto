use jsonrpsee::{
    core::{client::ClientT, RegisterMethodError},
    RpcModule,
};
use log::*;
use solana_account_decoder::UiAccount;
use solana_rpc_client_api::response::Response as ChainRpcResponse;

use crate::{
    rpc::params::RawParams,
    utils::{server_error, ServerErrorCode},
};

use super::DirectorRpc;

pub fn register_passthrough_methods(
    module: &mut RpcModule<DirectorRpc>,
) -> Result<(), RegisterMethodError> {
    macro_rules! register_passthrough {
        ($method:literal, $return_type:ty) => {
            module.register_async_method(
                $method,
                |params, rpc| async move {
                    debug!("{}", $method);
                    trace!("{:#?}", params);

                    let params = RawParams(params);
                    match rpc
                        .rpc_chain_client
                        .request::<ChainRpcResponse<$return_type>, RawParams>($method, params)
                        .await
                    {
                        Ok(res) => Ok(res),
                        Err(err) => Err(server_error(
                            format!("Failed to forward to on-chain RPC: {err:?}"),
                            ServerErrorCode::RpcClientError,
                        )),
                    }
                },
            )?;
        };
    }

    // register_passthrough!("getAccountInfo");
    // register_passthrough!("getBalance");
    // register_passthrough!("getBlock");
    // register_passthrough!("getBlockCommitment");
    // register_passthrough!("getBlockHeight");
    // register_passthrough!("getBlockProduction");
    // register_passthrough!("getBlockTime");
    // register_passthrough!("getBlocks");
    // register_passthrough!("getBlocksWithLimit");
    // register_passthrough!("getClusterNodes");
    // register_passthrough!("getEpochInfo");
    // register_passthrough!("getEpochSchedule");
    // register_passthrough!("getFeeForMessage");
    // register_passthrough!("getFirstAvailableBlock");
    // register_passthrough!("getGenesisHash");
    // register_passthrough!("getHealth");
    // register_passthrough!("getHighestSnapshotSlot");
    // register_passthrough!("getIdentity");
    // register_passthrough!("getInflationGovernor");
    // register_passthrough!("getInflationRate");
    // register_passthrough!("getInflationReward");
    // register_passthrough!("getLargestAccounts");
    // register_passthrough!("getLatestBlockhash");
    // register_passthrough!("getLeaderSchedule");
    // register_passthrough!("getMaxRetransmitSlot");
    // register_passthrough!("getMaxShredInsertSlot");
    // register_passthrough!("getMinimumBalanceForRentExemption");
    register_passthrough!("getMultipleAccounts", Vec<Option<UiAccount>>);
    // register_passthrough!("getProgramAccounts");
    // register_passthrough!("getRecentPerformanceSamples");
    // register_passthrough!("getRecentPrioritizationFees");
    // register_passthrough!("getSignatureStatuses");
    // register_passthrough!("getSignaturesForAddress");
    // register_passthrough!("getSlot");
    // register_passthrough!("getSlotLeader");
    // register_passthrough!("getSlotLeaders");
    // register_passthrough!("getStakeActivation");
    // register_passthrough!("getStakeMinimumDelegation");
    // register_passthrough!("getSupply");
    // register_passthrough!("getTokenAccountBalance");
    // register_passthrough!("getTokenAccountsByDelegate");
    // register_passthrough!("getTokenAccountsByOwner");
    // register_passthrough!("getTokenLargestAccounts");
    // register_passthrough!("getTokenSupply");
    // register_passthrough!("getTransaction");
    // register_passthrough!("getTransactionCount");
    // register_passthrough!("getVersion");
    // register_passthrough!("getVoteAccounts");
    // register_passthrough!("isBlockhashValid");
    // register_passthrough!("minimumLedgerSlot");
    // register_passthrough!("requestAirdrop");
    // register_passthrough!("simulateTransaction");

    Ok(())
}
