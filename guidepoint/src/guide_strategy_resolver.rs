use conjunto_core::{AccountProvider, GuideStrategy, RequestEndpoint};
use log::*;

pub struct GuideStrategyResolver<T: AccountProvider> {
    pub ephemeral_account_provider: T,
}

impl<T: AccountProvider> GuideStrategyResolver<T> {
    pub fn new(ephemeral_account_provider: T) -> Self {
        Self {
            ephemeral_account_provider,
        }
    }

    pub async fn resolve(&self, strategy: &GuideStrategy) -> RequestEndpoint {
        use GuideStrategy::*;

        match strategy {
            Chain => RequestEndpoint::Chain,
            Ephemeral => RequestEndpoint::Ephemeral,
            Both => RequestEndpoint::Both,
            TryEphemeralForAccount(address, is_subscription) => {
                self.guide_by_address(address, false, *is_subscription)
                    .await
            }
            TryEphemeralForProgram(program_id, is_subscription) => {
                self.guide_by_address(program_id, true, *is_subscription)
                    .await
            }
            TryEphemeralForSignature(_signature, _is_subscription) => {
                // TODO(thlorenz)
                // 1. Try to find in ephemeral and route there if found
                // 2. Otherwise find on chain and route there if found (may skip if that is too
                //    slow)
                // 3. Route to both for subscriptions and to chain for single requests
                RequestEndpoint::Both
            }
        }
    }

    async fn guide_by_address(
        &self,
        address: &str,
        is_program: bool,
        is_subscription: bool,
    ) -> RequestEndpoint {
        // If we find an invalid pubkey provided as an address then we forward
        // that to chain which will provide an error to the user
        let pubkey = match address.parse() {
            Ok(pubkey) => pubkey,
            Err(_) => return RequestEndpoint::Chain,
        };
        let account =
            match self.ephemeral_account_provider.get_account(&pubkey).await {
                Ok(Some(acc)) => acc,
                // If the ephemeral validator does not have he account then we go to chain for
                // single requests and to both for subscriptions (since the account may be created
                // after the subscription)
                Ok(None) if is_subscription => return RequestEndpoint::Both,
                Ok(None) => return RequestEndpoint::Chain,
                Err(err) => {
                    warn!("Error while fetching account: {:?}", err);
                    // In case of an error the account does not exist or the RPC client
                    // ran into an issue. In both cases we default to chain
                    return RequestEndpoint::Chain;
                }
            };
        if is_program && !account.executable {
            RequestEndpoint::Chain
        } else {
            RequestEndpoint::Ephemeral
        }
    }
}
