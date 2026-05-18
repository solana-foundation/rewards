use rewards_program_client::types::VestingSchedule;
use solana_sdk::pubkey::Pubkey;
use tiny_keccak::{Hasher, Keccak};

const LEAF_PREFIX: &[u8] = &[0];

/// Maximum byte length of a leaf's inner hash input:
/// 32 (claimant) + 8 (total_amount) + 25 (max schedule = CliffLinear)
const MAX_LEAF_DATA_LEN: usize = 65;

fn schedule_to_bytes(schedule: &VestingSchedule) -> Vec<u8> {
    match schedule {
        VestingSchedule::Immediate => vec![0],
        VestingSchedule::Linear { start_ts, end_ts } => {
            let mut bytes = Vec::with_capacity(17);
            bytes.push(1);
            bytes.extend_from_slice(&start_ts.to_le_bytes());
            bytes.extend_from_slice(&end_ts.to_le_bytes());
            bytes
        }
        VestingSchedule::Cliff { cliff_ts } => {
            let mut bytes = Vec::with_capacity(9);
            bytes.push(2);
            bytes.extend_from_slice(&cliff_ts.to_le_bytes());
            bytes
        }
        VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts } => {
            let mut bytes = Vec::with_capacity(25);
            bytes.push(3);
            bytes.extend_from_slice(&start_ts.to_le_bytes());
            bytes.extend_from_slice(&cliff_ts.to_le_bytes());
            bytes.extend_from_slice(&end_ts.to_le_bytes());
            bytes
        }
    }
}

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(data);
    hasher.finalize(&mut output);
    output
}

/// Compute the merkle leaf hash for a claim.
/// Matches the on-chain computation in merkle_utils.rs
pub fn compute_leaf_hash(claimant: &Pubkey, total_amount: u64, schedule: &VestingSchedule) -> [u8; 32] {
    let schedule_bytes = schedule_to_bytes(schedule);
    let schedule_len = schedule_bytes.len();
    let inner_len = 32 + 8 + schedule_len;
    let mut inner_data = [0u8; MAX_LEAF_DATA_LEN];
    inner_data[0..32].copy_from_slice(claimant.as_ref());
    inner_data[32..40].copy_from_slice(&total_amount.to_le_bytes());
    inner_data[40..40 + schedule_len].copy_from_slice(&schedule_bytes);

    let inner_hash = keccak256(&inner_data[..inner_len]);

    let mut outer_data = [0u8; 1 + 32];
    outer_data[0..1].copy_from_slice(LEAF_PREFIX);
    outer_data[1..33].copy_from_slice(&inner_hash);

    keccak256(&outer_data)
}

/// Hash two nodes together in sorted order (smaller first).
pub fn hash_pair(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut data = [0u8; 64];
    if a < b {
        data[0..32].copy_from_slice(a);
        data[32..64].copy_from_slice(b);
    } else {
        data[0..32].copy_from_slice(b);
        data[32..64].copy_from_slice(a);
    }
    keccak256(&data)
}

/// Represents a merkle tree leaf with all claim data
#[derive(Clone, Debug)]
pub struct MerkleLeaf {
    pub claimant: Pubkey,
    pub total_amount: u64,
    pub schedule: VestingSchedule,
    pub leaf_hash: [u8; 32],
}

impl MerkleLeaf {
    pub fn new(claimant: Pubkey, total_amount: u64, schedule: VestingSchedule) -> Self {
        let leaf_hash = compute_leaf_hash(&claimant, total_amount, &schedule);
        Self { claimant, total_amount, schedule, leaf_hash }
    }
}

/// A simple merkle tree builder for testing
pub struct MerkleTree {
    pub leaves: Vec<MerkleLeaf>,
    pub root: [u8; 32],
}

impl MerkleTree {
    pub fn new(leaves: Vec<MerkleLeaf>) -> Self {
        assert!(!leaves.is_empty(), "Merkle tree must have at least one leaf");
        let root = Self::compute_root(&leaves);
        Self { leaves, root }
    }

    fn compute_root(leaves: &[MerkleLeaf]) -> [u8; 32] {
        if leaves.len() == 1 {
            return leaves[0].leaf_hash;
        }

        let leaf_hashes: Vec<[u8; 32]> = leaves.iter().map(|l| l.leaf_hash).collect();
        Self::compute_root_from_hashes(&leaf_hashes)
    }

    fn compute_root_from_hashes(hashes: &[[u8; 32]]) -> [u8; 32] {
        if hashes.len() == 1 {
            return hashes[0];
        }

        let mut next_level = Vec::new();
        let mut i = 0;
        while i < hashes.len() {
            if i + 1 < hashes.len() {
                next_level.push(hash_pair(&hashes[i], &hashes[i + 1]));
            } else {
                // Odd leaf: promote to next level
                next_level.push(hashes[i]);
            }
            i += 2;
        }

        Self::compute_root_from_hashes(&next_level)
    }

    /// Get the merkle proof for a leaf at the given index
    pub fn get_proof(&self, index: usize) -> Vec<[u8; 32]> {
        assert!(index < self.leaves.len(), "Index out of bounds");

        if self.leaves.len() == 1 {
            return vec![];
        }

        let leaf_hashes: Vec<[u8; 32]> = self.leaves.iter().map(|l| l.leaf_hash).collect();
        Self::get_proof_recursive(&leaf_hashes, index)
    }

    fn get_proof_recursive(hashes: &[[u8; 32]], index: usize) -> Vec<[u8; 32]> {
        if hashes.len() == 1 {
            return vec![];
        }

        let mut proof = Vec::new();

        // Get sibling at current level
        let sibling_index = if index.is_multiple_of(2) { index + 1 } else { index - 1 };
        if sibling_index < hashes.len() {
            proof.push(hashes[sibling_index]);
        }

        // Compute next level
        let mut next_level = Vec::new();
        let mut i = 0;
        while i < hashes.len() {
            if i + 1 < hashes.len() {
                next_level.push(hash_pair(&hashes[i], &hashes[i + 1]));
            } else {
                next_level.push(hashes[i]);
            }
            i += 2;
        }

        // Recurse to next level
        let next_index = index / 2;
        proof.extend(Self::get_proof_recursive(&next_level, next_index));

        proof
    }

    /// Find the index of a leaf by claimant
    pub fn find_leaf_index(&self, claimant: &Pubkey) -> Option<usize> {
        self.leaves.iter().position(|l| l.claimant == *claimant)
    }

    /// Get proof for a claimant
    pub fn get_proof_for_claimant(&self, claimant: &Pubkey) -> Option<Vec<[u8; 32]>> {
        self.find_leaf_index(claimant).map(|idx| self.get_proof(idx))
    }

    /// Get the leaf data for a claimant
    pub fn get_leaf(&self, claimant: &Pubkey) -> Option<&MerkleLeaf> {
        self.leaves.iter().find(|l| l.claimant == *claimant)
    }
}
