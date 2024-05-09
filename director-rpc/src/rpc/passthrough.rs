use jsonrpsee::{
    core::{RegisterMethodError, RpcResult},
    types::Params,
    RpcModule,
};
use log::*;

use super::DirectorRpc;

pub fn register_passthrough_methods(
    module: &mut RpcModule<DirectorRpc>,
) -> Result<(), RegisterMethodError> {
    macro_rules! register_passthrough {
        ($method:literal) => {
            module.register_async_method(
                $method,
                |params, ctx| async move {
                    passthrough($method, params, &ctx).await
                },
            )?;
        };
    }

    register_passthrough!("getAccountInfo");
    register_passthrough!("getBalance");
    register_passthrough!("getBlock");
    register_passthrough!("getBlockCommitment");
    register_passthrough!("getBlockHeight");
    register_passthrough!("getBlockProduction");
    register_passthrough!("getBlockTime");
    register_passthrough!("getBlocks");
    register_passthrough!("getBlocksWithLimit");
    register_passthrough!("getClusterNodes");
    register_passthrough!("getEpochInfo");
    register_passthrough!("getEpochSchedule");
    register_passthrough!("getFeeForMessage");
    register_passthrough!("getFirstAvailableBlock");
    register_passthrough!("getGenesisHash");
    register_passthrough!("getHealth");
    register_passthrough!("getHighestSnapshotSlot");
    register_passthrough!("getIdentity");
    register_passthrough!("getInflationGovernor");
    register_passthrough!("getInflationRate");
    register_passthrough!("getInflationReward");
    register_passthrough!("getLargestAccounts");
    register_passthrough!("getLatestBlockhash");
    register_passthrough!("getLeaderSchedule");
    register_passthrough!("getMaxRetransmitSlot");
    register_passthrough!("getMaxShredInsertSlot");
    register_passthrough!("getMinimumBalanceForRentExemption");
    register_passthrough!("getMultipleAccounts");
    register_passthrough!("getProgramAccounts");
    register_passthrough!("getRecentPerformanceSamples");
    register_passthrough!("getRecentPrioritizationFees");
    register_passthrough!("getSignatureStatuses");
    register_passthrough!("getSignaturesForAddress");
    register_passthrough!("getSlot");
    register_passthrough!("getSlotLeader");
    register_passthrough!("getSlotLeaders");
    register_passthrough!("getStakeActivation");
    register_passthrough!("getStakeMinimumDelegation");
    register_passthrough!("getSupply");
    register_passthrough!("getTokenAccountBalance");
    register_passthrough!("getTokenAccountsByDelegate");
    register_passthrough!("getTokenAccountsByOwner");
    register_passthrough!("getTokenLargestAccounts");
    register_passthrough!("getTokenSupply");
    register_passthrough!("getTransaction");
    register_passthrough!("getTransactionCount");
    register_passthrough!("getVersion");
    register_passthrough!("getVoteAccounts");
    register_passthrough!("isBlockhashValid");
    register_passthrough!("minimumLedgerSlot");
    register_passthrough!("requestAirdrop");
    register_passthrough!("simulateTransaction");

    Ok(())
}

async fn passthrough(
    method: &str,
    params: Params<'static>,
    _ctx: &DirectorRpc,
) -> RpcResult<()> {
    debug!("{}: {:#?}", method, params);
    RpcResult::Ok(())
}
