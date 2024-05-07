use paste::paste;

// -----------------
// Seeds
// -----------------
macro_rules! seeds {
    ($prefix:ident, $bytes_const:expr) => {
        paste! {
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _seeds>]<'a>(id: &'a [u8]) -> [&'a [u8]; 2] {
                [$bytes_const, id]
            }
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _seeds_with_bump>]<'a>(id: &'a [u8], bump: &'a [u8; 1]) -> [&'a [u8]; 3] {
                [$bytes_const, id, bump]
            }
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _seeds_from_pubkey>]<'a>(id: &'a ::solana_sdk::pubkey::Pubkey) -> [&'a [u8]; 2] {
                [$bytes_const, id.as_ref()]
            }
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _seeds_with_bump_from_pubkey>]<'a>(
                id: &'a ::solana_sdk::pubkey::Pubkey,
                bump: &'a [u8; 1],
            ) -> [&'a [u8]; 3] {
                [$bytes_const, id.as_ref(), bump]
            }
        }
    };
}

seeds! {delegation, b"delegation" }
seeds! {buffer, b"buffer" }
seeds! {state_diff, b"state-diff" }
seeds! {commit_record, b"commit-state-record" }

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey::Pubkey;

    use crate::consts::{BUFFER, COMMIT_RECORD, DELEGATION, STATE_DIFF};

    use super::*;

    // -----------------
    // Delegation Seeds
    // -----------------
    #[test]
    fn test_delegation_seeds() {
        let id = [1, 2, 3];
        let seeds = delegation_seeds(&id);
        assert_eq!(seeds, [DELEGATION, &id]);
    }

    #[test]
    fn test_delegation_seeds_with_bump() {
        let id = [1, 2, 3];
        let bump = [4];
        let seeds = delegation_seeds_with_bump(&id, &bump);
        assert_eq!(seeds, [DELEGATION, &id, &bump]);
    }

    #[test]
    fn test_delegation_seeds_from_pubkey() {
        let id = Pubkey::new_unique();
        let seeds = delegation_seeds_from_pubkey(&id);
        assert_eq!(seeds, [DELEGATION, id.as_ref()]);
    }

    #[test]
    fn test_delegation_seeds_with_bump_from_pubkey() {
        let id = Pubkey::new_unique();
        let bump = [4];
        let seeds = delegation_seeds_with_bump_from_pubkey(&id, &bump);
        assert_eq!(seeds, [DELEGATION, id.as_ref(), &bump]);
    }

    // -----------------
    // Buffer Seeds
    // -----------------
    #[test]
    fn test_buffer_seeds() {
        let id = [1, 2, 3];
        let seeds = buffer_seeds(&id);
        assert_eq!(seeds, [BUFFER, &id]);
    }

    #[test]
    fn test_buffer_seeds_with_bump() {
        let id = [1, 2, 3];
        let bump = [4];
        let seeds = buffer_seeds_with_bump(&id, &bump);
        assert_eq!(seeds, [BUFFER, &id, &bump]);
    }

    #[test]
    fn test_buffer_seeds_from_pubkey() {
        let id = Pubkey::new_unique();
        let seeds = buffer_seeds_from_pubkey(&id);
        assert_eq!(seeds, [BUFFER, id.as_ref()]);
    }

    #[test]
    fn test_buffer_seeds_with_bump_from_pubkey() {
        let id = Pubkey::new_unique();
        let bump = [4];
        let seeds = buffer_seeds_with_bump_from_pubkey(&id, &bump);
        assert_eq!(seeds, [BUFFER, id.as_ref(), &bump]);
    }

    // -----------------
    // State Diff Seeds
    // -----------------
    #[test]
    fn test_state_diff_seeds() {
        let id = [1, 2, 3];
        let seeds = state_diff_seeds(&id);
        assert_eq!(seeds, [STATE_DIFF, &id]);
    }

    #[test]
    fn test_state_diff_seeds_with_bump() {
        let id = [1, 2, 3];
        let bump = [4];
        let seeds = state_diff_seeds_with_bump(&id, &bump);
        assert_eq!(seeds, [STATE_DIFF, &id, &bump]);
    }

    #[test]
    fn test_state_diff_seeds_from_pubkey() {
        let id = Pubkey::new_unique();
        let seeds = state_diff_seeds_from_pubkey(&id);
        assert_eq!(seeds, [STATE_DIFF, id.as_ref()]);
    }

    #[test]
    fn test_state_diff_seeds_with_bump_from_pubkey() {
        let id = Pubkey::new_unique();
        let bump = [4];
        let seeds = state_diff_seeds_with_bump_from_pubkey(&id, &bump);
        assert_eq!(seeds, [STATE_DIFF, id.as_ref(), &bump]);
    }

    // -----------------
    // Commit Record Seeds
    // -----------------
    #[test]
    fn test_commit_record_seeds() {
        let id = [1, 2, 3];
        let seeds = commit_record_seeds(&id);
        assert_eq!(seeds, [COMMIT_RECORD, &id]);
    }

    #[test]
    fn test_commit_record_seeds_with_bump() {
        let id = [1, 2, 3];
        let bump = [4];
        let seeds = commit_record_seeds_with_bump(&id, &bump);
        assert_eq!(seeds, [COMMIT_RECORD, &id, &bump]);
    }

    #[test]
    fn test_commit_record_seeds_from_pubkey() {
        let id = Pubkey::new_unique();
        let seeds = commit_record_seeds_from_pubkey(&id);
        assert_eq!(seeds, [COMMIT_RECORD, id.as_ref()]);
    }

    #[test]
    fn test_commit_record_seeds_with_bump_from_pubkey() {
        let id = Pubkey::new_unique();
        let bump = [4];
        let seeds = commit_record_seeds_with_bump_from_pubkey(&id, &bump);
        assert_eq!(seeds, [COMMIT_RECORD, id.as_ref(), &bump]);
    }
}
