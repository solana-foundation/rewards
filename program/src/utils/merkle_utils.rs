use const_crypto::sha3::Keccak256;
use pinocchio::{error::ProgramError, Address};

use crate::errors::RewardsProgramError;

/// Leaf prefix to prevent second preimage attacks.
pub const LEAF_PREFIX: &[u8] = &[0];
pub const HASH_SIZE: usize = 32;
pub const VEC_PREFIX_LEN: usize = 4;

/// Maximum byte length of a leaf's inner hash input:
/// 32 (claimant) + 8 (total_amount) + 25 (max schedule = CliffLinear)
const MAX_LEAF_DATA_LEN: usize = 65;

fn keccak256(data: &[u8]) -> [u8; 32] {
    Keccak256::new().update(data).finalize()
}

/// Compute the merkle leaf hash for a claim.
///
/// The leaf format is:
/// `hash(LEAF_PREFIX || hash(claimant || total_amount || schedule_bytes))`
pub fn compute_leaf_hash(claimant: &Address, total_amount: u64, schedule_bytes: &[u8]) -> [u8; 32] {
    let schedule_len = schedule_bytes.len();
    let inner_len = HASH_SIZE + 8 + schedule_len;
    let mut inner_data = [0u8; MAX_LEAF_DATA_LEN];
    inner_data[0..HASH_SIZE].copy_from_slice(claimant.as_ref());
    inner_data[HASH_SIZE..HASH_SIZE + 8].copy_from_slice(&total_amount.to_le_bytes());
    inner_data[HASH_SIZE + 8..HASH_SIZE + 8 + schedule_len].copy_from_slice(schedule_bytes);

    let inner_hash = keccak256(&inner_data[..inner_len]);

    // Outer hash: hash(LEAF_PREFIX || inner_hash)
    let mut outer_data = [0u8; 1 + HASH_SIZE]; // 33 bytes
    outer_data[0..1].copy_from_slice(LEAF_PREFIX);
    outer_data[1..1 + HASH_SIZE].copy_from_slice(&inner_hash);

    keccak256(&outer_data)
}

/// Verify a merkle proof against a root.
///
/// The proof is an array of sibling hashes from leaf to root.
/// For each level, if the current hash is less than the sibling,
/// hash(current || sibling), otherwise hash(sibling || current).
/// This sorted pair ordering ensures deterministic tree construction.
pub fn verify_proof(proof: &[[u8; 32]], root: &[u8; 32], leaf: &[u8; 32]) -> bool {
    let mut computed_hash = *leaf;

    for sibling in proof {
        computed_hash = hash_pair(&computed_hash, sibling);
    }

    computed_hash == *root
}

/// Hash two nodes together in sorted order (smaller first).
/// This ensures deterministic tree construction regardless of proof ordering.
fn hash_pair(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut data = [0u8; HASH_SIZE * 2];
    if a < b {
        data[0..HASH_SIZE].copy_from_slice(a);
        data[HASH_SIZE..HASH_SIZE * 2].copy_from_slice(b);
    } else {
        data[0..HASH_SIZE].copy_from_slice(b);
        data[HASH_SIZE..HASH_SIZE * 2].copy_from_slice(a);
    }
    keccak256(&data)
}

/// Verify a merkle proof and return error if invalid.
pub fn verify_proof_or_error(proof: &[[u8; 32]], root: &[u8; 32], leaf: &[u8; 32]) -> Result<(), ProgramError> {
    if verify_proof(proof, root, leaf) {
        Ok(())
    } else {
        Err(RewardsProgramError::InvalidMerkleProof.into())
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::*;
    use crate::utils::VestingSchedule;

    fn schedule_bytes(schedule: VestingSchedule) -> Vec<u8> {
        schedule.to_bytes()
    }

    #[test]
    fn test_compute_leaf_hash_deterministic() {
        let claimant = Address::new_from_array([1u8; 32]);
        let sb = schedule_bytes(VestingSchedule::Linear { start_ts: 100, end_ts: 200 });

        let hash1 = compute_leaf_hash(&claimant, 1000, &sb);
        let hash2 = compute_leaf_hash(&claimant, 1000, &sb);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_leaf_hash_different_inputs() {
        let claimant1 = Address::new_from_array([1u8; 32]);
        let claimant2 = Address::new_from_array([2u8; 32]);
        let sb = schedule_bytes(VestingSchedule::Linear { start_ts: 100, end_ts: 200 });

        let hash1 = compute_leaf_hash(&claimant1, 1000, &sb);
        let hash2 = compute_leaf_hash(&claimant2, 1000, &sb);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compute_leaf_hash_different_amounts() {
        let claimant = Address::new_from_array([1u8; 32]);
        let sb = schedule_bytes(VestingSchedule::Linear { start_ts: 100, end_ts: 200 });

        let hash1 = compute_leaf_hash(&claimant, 1000, &sb);
        let hash2 = compute_leaf_hash(&claimant, 2000, &sb);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compute_leaf_hash_different_timestamps() {
        let claimant = Address::new_from_array([1u8; 32]);
        let sb1 = schedule_bytes(VestingSchedule::Linear { start_ts: 100, end_ts: 200 });
        let sb2 = schedule_bytes(VestingSchedule::Linear { start_ts: 100, end_ts: 300 });

        let hash1 = compute_leaf_hash(&claimant, 1000, &sb1);
        let hash2 = compute_leaf_hash(&claimant, 1000, &sb2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compute_leaf_hash_different_schedule_types() {
        let claimant = Address::new_from_array([1u8; 32]);
        let sb1 = schedule_bytes(VestingSchedule::Immediate {});
        let sb2 = schedule_bytes(VestingSchedule::Linear { start_ts: 100, end_ts: 200 });

        let hash1 = compute_leaf_hash(&claimant, 1000, &sb1);
        let hash2 = compute_leaf_hash(&claimant, 1000, &sb2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_pair_commutative() {
        let a = [1u8; 32];
        let b = [2u8; 32];

        let hash1 = hash_pair(&a, &b);
        let hash2 = hash_pair(&b, &a);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_pair_deterministic() {
        let a = [1u8; 32];
        let b = [2u8; 32];

        let hash1 = hash_pair(&a, &b);
        let hash2 = hash_pair(&a, &b);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_verify_proof_single_leaf() {
        let claimant = Address::new_from_array([1u8; 32]);
        let sb = schedule_bytes(VestingSchedule::Linear { start_ts: 100, end_ts: 200 });
        let leaf = compute_leaf_hash(&claimant, 1000, &sb);
        let root = leaf;
        let proof: Vec<[u8; 32]> = vec![];

        assert!(verify_proof(&proof, &root, &leaf));
    }

    #[test]
    fn test_verify_proof_two_leaves() {
        let claimant1 = Address::new_from_array([1u8; 32]);
        let claimant2 = Address::new_from_array([2u8; 32]);
        let sb = schedule_bytes(VestingSchedule::Linear { start_ts: 100, end_ts: 200 });

        let leaf1 = compute_leaf_hash(&claimant1, 1000, &sb);
        let leaf2 = compute_leaf_hash(&claimant2, 2000, &sb);

        let root = hash_pair(&leaf1, &leaf2);

        assert!(verify_proof(&[leaf2], &root, &leaf1));
        assert!(verify_proof(&[leaf1], &root, &leaf2));
    }

    #[test]
    fn test_verify_proof_four_leaves() {
        let sb = schedule_bytes(VestingSchedule::Linear { start_ts: 100, end_ts: 200 });
        let leaves: Vec<[u8; 32]> = (0..4)
            .map(|i| {
                let mut addr = [0u8; 32];
                addr[0] = i;
                compute_leaf_hash(&Address::new_from_array(addr), 1000 * (i as u64 + 1), &sb)
            })
            .collect();

        let n01 = hash_pair(&leaves[0], &leaves[1]);
        let n23 = hash_pair(&leaves[2], &leaves[3]);
        let root = hash_pair(&n01, &n23);

        assert!(verify_proof(&[leaves[1], n23], &root, &leaves[0]));
        assert!(verify_proof(&[leaves[0], n23], &root, &leaves[1]));
        assert!(verify_proof(&[leaves[3], n01], &root, &leaves[2]));
        assert!(verify_proof(&[leaves[2], n01], &root, &leaves[3]));
    }

    #[test]
    fn test_verify_proof_invalid_proof() {
        let claimant1 = Address::new_from_array([1u8; 32]);
        let claimant2 = Address::new_from_array([2u8; 32]);
        let sb = schedule_bytes(VestingSchedule::Linear { start_ts: 100, end_ts: 200 });

        let leaf1 = compute_leaf_hash(&claimant1, 1000, &sb);
        let leaf2 = compute_leaf_hash(&claimant2, 2000, &sb);

        let root = hash_pair(&leaf1, &leaf2);

        let wrong_sibling = [99u8; 32];
        assert!(!verify_proof(&[wrong_sibling], &root, &leaf1));
    }

    #[test]
    fn test_verify_proof_invalid_leaf() {
        let claimant1 = Address::new_from_array([1u8; 32]);
        let claimant2 = Address::new_from_array([2u8; 32]);
        let sb = schedule_bytes(VestingSchedule::Linear { start_ts: 100, end_ts: 200 });

        let leaf1 = compute_leaf_hash(&claimant1, 1000, &sb);
        let leaf2 = compute_leaf_hash(&claimant2, 2000, &sb);

        let root = hash_pair(&leaf1, &leaf2);

        let wrong_leaf = compute_leaf_hash(&claimant1, 9999, &sb);
        assert!(!verify_proof(&[leaf2], &root, &wrong_leaf));
    }

    #[test]
    fn test_verify_proof_or_error_success() {
        let claimant = Address::new_from_array([1u8; 32]);
        let sb = schedule_bytes(VestingSchedule::Linear { start_ts: 100, end_ts: 200 });
        let leaf = compute_leaf_hash(&claimant, 1000, &sb);
        let root = leaf;
        let proof: Vec<[u8; 32]> = vec![];

        assert!(verify_proof_or_error(&proof, &root, &leaf).is_ok());
    }

    #[test]
    fn test_verify_proof_or_error_failure() {
        let claimant = Address::new_from_array([1u8; 32]);
        let sb = schedule_bytes(VestingSchedule::Linear { start_ts: 100, end_ts: 200 });
        let leaf = compute_leaf_hash(&claimant, 1000, &sb);
        let wrong_root = [99u8; 32];
        let proof: Vec<[u8; 32]> = vec![];

        assert!(verify_proof_or_error(&proof, &wrong_root, &leaf).is_err());
    }
}
