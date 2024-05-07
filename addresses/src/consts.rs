// NOTE: originally created in delegation program:
// https://github.com/magicblock-labs/delegation-program/blob/104d7772882e5fbeb871f90a1a33a95ccf98a62c/src/consts.rs

/// The seed of the authority account PDA.
pub const DELEGATION: &[u8] = b"delegation";

/// The seed of the buffer account PDA.
pub const BUFFER: &[u8] = b"buffer";

/// The seed of the state-diff PDA.
pub const STATE_DIFF: &[u8] = b"state-diff";

/// The seed of a commit state record PDA.
pub const COMMIT_RECORD: &[u8] = b"commit-state-record";

/// The address of the delegation program
pub const DELEGATION_PROGRAM_ADDR: &str = "DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh";
