// -----------------
// GuideStrategy
// -----------------
#[derive(Debug, PartialEq, Eq)]
pub enum GuideStrategy {
    /// Forward to chain
    Chain,
    /// Forward to ephemeral
    Ephemeral,
    /// Forward to both chain and ephemeral
    Both,
    /// Forward to ephemeral if that validator has the account of given address,
    /// otherwise forward to chain
    TryEphemeralForAccount(String),
    /// Forward to ephemeral if that validator has the program of given address,
    /// otherwise forward to chain
    TryEphemeralForProgram(String),
    /// Forward to ephemeral if that validator has the transaction signature,
    /// otherwise forward to chain
    TryEphemeralForSignature(String),
}

// -----------------
// RequestEndpoint
// -----------------
pub enum RequestEndpoint {
    /// Forward to chain only
    Chain,
    /// Forward to ephemeral only
    Ephemeral,
    /// Forward to both chain and ephemeral
    Both,
    /// Request is unroutable which is an error case
    Unroutable,
}
