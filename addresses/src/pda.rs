use paste::paste;

use crate::consts::{BUFFER, COMMIT_RECORD, DELEGATION, STATE_DIFF};

// -----------------
// Seeds
// -----------------
macro_rules! seeds {
    ($prefix:ident, $bytes_const:expr) => {
        paste! {
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _seeds>]<'a>(pda_id: &'a [u8]) -> [&'a [u8]; 3] {
                [&crate::consts::DELEGATION_PROGRAM_ARRAY, $bytes_const, pda_id]
            }
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _seeds_with_bump>]<'a>(pda_id: &'a [u8], bump: &'a [u8; 1]) -> [&'a [u8]; 4] {
                [&crate::consts::DELEGATION_PROGRAM_ARRAY, $bytes_const, pda_id, bump]
            }
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _seeds_from_pubkey>]<'a>(pda_id: &'a ::solana_sdk::pubkey::Pubkey) -> [&'a [u8]; 3] {
                [&crate::consts::DELEGATION_PROGRAM_ARRAY, $bytes_const, pda_id.as_ref()]
            }
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _seeds_with_bump_from_pubkey>]<'a>(
                pda_id: &'a ::solana_sdk::pubkey::Pubkey,
                bump: &'a [u8; 1],
            ) -> [&'a [u8]; 4] {
                [&crate::consts::DELEGATION_PROGRAM_ARRAY, $bytes_const, pda_id.as_ref(), bump]
            }
        }
    };
}

seeds! {delegation, DELEGATION }
seeds! {buffer, BUFFER }
seeds! {state_diff, STATE_DIFF }
seeds! {commit_record, COMMIT_RECORD }

// -----------------
// PDAs
// -----------------
/*
macro_rules! pdas {
    ($prefix:ident) => {
        paste! {
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _pda>]<'a>(id: &'a [u8]) -> ::solana_sdk::pubkey::Pubkey {
                let seeds = [<$prefix _seeds>](id);
                ::solana_sdk::pubkey::Pubkey::find_program_address(
                    &[&seeds, DELEGATION_PROGRAM_ID.as_ref(), id],
                    &DELEGATION_PROGRAM_ID
                ).0
            }
        }
    };
}
*/

// pdas! {delegation }

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey::Pubkey;

    use crate::consts::{
        BUFFER, COMMIT_RECORD, DELEGATION, DELEGATION_PROGRAM_ARRAY, STATE_DIFF,
    };

    use super::*;

    // -----------------
    // Delegation Seeds
    // -----------------
    #[test]
    fn test_delegation_seeds() {
        let id = [1, 2, 3];
        let seeds = delegation_seeds(&id);
        assert_eq!(seeds, [&DELEGATION_PROGRAM_ARRAY, DELEGATION, &id]);
    }

    #[test]
    fn test_delegation_seeds_with_bump() {
        let id = [1, 2, 3];
        let bump = [4];
        let seeds = delegation_seeds_with_bump(&id, &bump);
        assert_eq!(seeds, [&DELEGATION_PROGRAM_ARRAY, DELEGATION, &id, &bump]);
    }

    #[test]
    fn test_delegation_seeds_from_pubkey() {
        let id = Pubkey::new_unique();
        let seeds = delegation_seeds_from_pubkey(&id);
        assert_eq!(seeds, [&DELEGATION_PROGRAM_ARRAY, DELEGATION, id.as_ref()]);
    }

    #[test]
    fn test_delegation_seeds_with_bump_from_pubkey() {
        let id = Pubkey::new_unique();
        let bump = [4];
        let seeds = delegation_seeds_with_bump_from_pubkey(&id, &bump);
        assert_eq!(
            seeds,
            [&DELEGATION_PROGRAM_ARRAY, DELEGATION, id.as_ref(), &bump]
        );
    }

    // -----------------
    // Buffer Seeds
    // -----------------
    #[test]
    fn test_buffer_seeds() {
        let id = [1, 2, 3];
        let seeds = buffer_seeds(&id);
        assert_eq!(seeds, [&DELEGATION_PROGRAM_ARRAY, BUFFER, &id]);
    }

    #[test]
    fn test_buffer_seeds_with_bump() {
        let id = [1, 2, 3];
        let bump = [4];
        let seeds = buffer_seeds_with_bump(&id, &bump);
        assert_eq!(seeds, [&DELEGATION_PROGRAM_ARRAY, BUFFER, &id, &bump]);
    }

    #[test]
    fn test_buffer_seeds_from_pubkey() {
        let id = Pubkey::new_unique();
        let seeds = buffer_seeds_from_pubkey(&id);
        assert_eq!(seeds, [&DELEGATION_PROGRAM_ARRAY, BUFFER, id.as_ref()]);
    }

    #[test]
    fn test_buffer_seeds_with_bump_from_pubkey() {
        let id = Pubkey::new_unique();
        let bump = [4];
        let seeds = buffer_seeds_with_bump_from_pubkey(&id, &bump);
        assert_eq!(
            seeds,
            [&DELEGATION_PROGRAM_ARRAY, BUFFER, id.as_ref(), &bump]
        );
    }

    // -----------------
    // State Diff Seeds
    // -----------------
    #[test]
    fn test_state_diff_seeds() {
        let id = [1, 2, 3];
        let seeds = state_diff_seeds(&id);
        assert_eq!(seeds, [&DELEGATION_PROGRAM_ARRAY, STATE_DIFF, &id]);
    }

    #[test]
    fn test_state_diff_seeds_with_bump() {
        let id = [1, 2, 3];
        let bump = [4];
        let seeds = state_diff_seeds_with_bump(&id, &bump);
        assert_eq!(seeds, [&DELEGATION_PROGRAM_ARRAY, STATE_DIFF, &id, &bump]);
    }

    #[test]
    fn test_state_diff_seeds_from_pubkey() {
        let id = Pubkey::new_unique();
        let seeds = state_diff_seeds_from_pubkey(&id);
        assert_eq!(seeds, [&DELEGATION_PROGRAM_ARRAY, STATE_DIFF, id.as_ref()]);
    }

    #[test]
    fn test_state_diff_seeds_with_bump_from_pubkey() {
        let id = Pubkey::new_unique();
        let bump = [4];
        let seeds = state_diff_seeds_with_bump_from_pubkey(&id, &bump);
        assert_eq!(
            seeds,
            [&DELEGATION_PROGRAM_ARRAY, STATE_DIFF, id.as_ref(), &bump]
        );
    }

    // -----------------
    // Commit Record Seeds
    // -----------------
    #[test]
    fn test_commit_record_seeds() {
        let id = [1, 2, 3];
        let seeds = commit_record_seeds(&id);
        assert_eq!(seeds, [&DELEGATION_PROGRAM_ARRAY, COMMIT_RECORD, &id]);
    }

    #[test]
    fn test_commit_record_seeds_with_bump() {
        let id = [1, 2, 3];
        let bump = [4];
        let seeds = commit_record_seeds_with_bump(&id, &bump);
        assert_eq!(
            seeds,
            [&DELEGATION_PROGRAM_ARRAY, COMMIT_RECORD, &id, &bump]
        );
    }

    #[test]
    fn test_commit_record_seeds_from_pubkey() {
        let id = Pubkey::new_unique();
        let seeds = commit_record_seeds_from_pubkey(&id);
        assert_eq!(
            seeds,
            [&DELEGATION_PROGRAM_ARRAY, COMMIT_RECORD, id.as_ref()]
        );
    }

    #[test]
    fn test_commit_record_seeds_with_bump_from_pubkey() {
        let id = Pubkey::new_unique();
        let bump = [4];
        let seeds = commit_record_seeds_with_bump_from_pubkey(&id, &bump);
        assert_eq!(
            seeds,
            [&DELEGATION_PROGRAM_ARRAY, COMMIT_RECORD, id.as_ref(), &bump]
        );
    }
}
