use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct RewardDistributedEvent {
    pub reward_pool: Address,
    pub amount: u64,
    pub new_reward_per_token: u128,
}

impl EventDiscriminator for RewardDistributedEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::RewardDistributed as u8;
}

impl EventSerialize for RewardDistributedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.reward_pool.as_ref());
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.extend_from_slice(&self.new_reward_per_token.to_le_bytes());
        data
    }
}

impl RewardDistributedEvent {
    pub const DATA_LEN: usize = 32 + 8 + 16; // reward_pool + amount + new_reward_per_token

    #[inline(always)]
    pub fn new(reward_pool: Address, amount: u64, new_reward_per_token: u128) -> Self {
        Self { reward_pool, amount, new_reward_per_token }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_reward_distributed_event_new() {
        let reward_pool = Address::new_from_array([1u8; 32]);

        let event = RewardDistributedEvent::new(reward_pool, 100_000, 500_000_000_000);

        assert_eq!(event.reward_pool, reward_pool);
        assert_eq!(event.amount, 100_000);
        assert_eq!(event.new_reward_per_token, 500_000_000_000);
    }

    #[test]
    fn test_reward_distributed_event_to_bytes_inner() {
        let reward_pool = Address::new_from_array([1u8; 32]);
        let event = RewardDistributedEvent::new(reward_pool, 200_000, 1_000_000_000_000);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), RewardDistributedEvent::DATA_LEN);
        assert_eq!(&bytes[..32], reward_pool.as_ref());
        assert_eq!(&bytes[32..40], &200_000u64.to_le_bytes());
        assert_eq!(&bytes[40..56], &1_000_000_000_000u128.to_le_bytes());
    }

    #[test]
    fn test_reward_distributed_event_to_bytes() {
        let reward_pool = Address::new_from_array([1u8; 32]);
        let event = RewardDistributedEvent::new(reward_pool, 100_000, 500_000_000_000);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + RewardDistributedEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::RewardDistributed as u8);
        assert_eq!(&bytes[9..41], reward_pool.as_ref());
    }
}
