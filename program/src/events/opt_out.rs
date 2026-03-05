use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct OptOutEvent {
    pub reward_pool: Address,
    pub user: Address,
    pub rewards_claimed: u64,
}

impl EventDiscriminator for OptOutEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::OptedOut as u8;
}

impl EventSerialize for OptOutEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.reward_pool.as_ref());
        data.extend_from_slice(self.user.as_ref());
        data.extend_from_slice(&self.rewards_claimed.to_le_bytes());
        data
    }
}

impl OptOutEvent {
    pub const DATA_LEN: usize = 32 + 32 + 8; // reward_pool + user + rewards_claimed

    #[inline(always)]
    pub fn new(reward_pool: Address, user: Address, rewards_claimed: u64) -> Self {
        Self { reward_pool, user, rewards_claimed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_opt_out_event_new() {
        let reward_pool = Address::new_from_array([1u8; 32]);
        let user = Address::new_from_array([2u8; 32]);

        let event = OptOutEvent::new(reward_pool, user, 50_000);

        assert_eq!(event.reward_pool, reward_pool);
        assert_eq!(event.user, user);
        assert_eq!(event.rewards_claimed, 50_000);
    }

    #[test]
    fn test_opt_out_event_to_bytes_inner() {
        let reward_pool = Address::new_from_array([1u8; 32]);
        let user = Address::new_from_array([2u8; 32]);
        let event = OptOutEvent::new(reward_pool, user, 100_000);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), OptOutEvent::DATA_LEN);
        assert_eq!(&bytes[..32], reward_pool.as_ref());
        assert_eq!(&bytes[32..64], user.as_ref());
        assert_eq!(&bytes[64..72], &100_000u64.to_le_bytes());
    }

    #[test]
    fn test_opt_out_event_to_bytes() {
        let reward_pool = Address::new_from_array([1u8; 32]);
        let user = Address::new_from_array([2u8; 32]);
        let event = OptOutEvent::new(reward_pool, user, 50_000);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + OptOutEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::OptedOut as u8);
        assert_eq!(&bytes[9..41], reward_pool.as_ref());
    }
}
