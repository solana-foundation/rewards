use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct MerkleRootSetEvent {
    pub reward_pool: Address,
    pub authority: Address,
    pub merkle_root: [u8; 32],
    pub root_version: u64,
}

impl EventDiscriminator for MerkleRootSetEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::MerkleRootSet as u8;
}

impl EventSerialize for MerkleRootSetEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.reward_pool.as_ref());
        data.extend_from_slice(self.authority.as_ref());
        data.extend_from_slice(&self.merkle_root);
        data.extend_from_slice(&self.root_version.to_le_bytes());
        data
    }
}

impl MerkleRootSetEvent {
    pub const DATA_LEN: usize = 32 + 32 + 32 + 8; // reward_pool + authority + merkle_root + root_version

    #[inline(always)]
    pub fn new(reward_pool: Address, authority: Address, merkle_root: [u8; 32], root_version: u64) -> Self {
        Self { reward_pool, authority, merkle_root, root_version }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_merkle_root_set_event_new() {
        let reward_pool = Address::new_from_array([1u8; 32]);
        let authority = Address::new_from_array([2u8; 32]);
        let merkle_root = [3u8; 32];

        let event = MerkleRootSetEvent::new(reward_pool, authority, merkle_root, 7);

        assert_eq!(event.reward_pool, reward_pool);
        assert_eq!(event.authority, authority);
        assert_eq!(event.merkle_root, merkle_root);
        assert_eq!(event.root_version, 7);
    }

    #[test]
    fn test_merkle_root_set_event_to_bytes_inner() {
        let reward_pool = Address::new_from_array([1u8; 32]);
        let authority = Address::new_from_array([2u8; 32]);
        let merkle_root = [3u8; 32];
        let event = MerkleRootSetEvent::new(reward_pool, authority, merkle_root, 9);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), MerkleRootSetEvent::DATA_LEN);
        assert_eq!(&bytes[..32], reward_pool.as_ref());
        assert_eq!(&bytes[32..64], authority.as_ref());
        assert_eq!(&bytes[64..96], &merkle_root);
        assert_eq!(&bytes[96..104], &9u64.to_le_bytes());
    }

    #[test]
    fn test_merkle_root_set_event_to_bytes() {
        let reward_pool = Address::new_from_array([1u8; 32]);
        let authority = Address::new_from_array([2u8; 32]);
        let merkle_root = [3u8; 32];
        let event = MerkleRootSetEvent::new(reward_pool, authority, merkle_root, 11);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + MerkleRootSetEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::MerkleRootSet as u8);
        assert_eq!(&bytes[9..41], reward_pool.as_ref());
    }
}
