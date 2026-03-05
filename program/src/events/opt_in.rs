use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct OptInEvent {
    pub reward_pool: Address,
    pub user: Address,
    pub balance: u64,
}

impl EventDiscriminator for OptInEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::OptedIn as u8;
}

impl EventSerialize for OptInEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.reward_pool.as_ref());
        data.extend_from_slice(self.user.as_ref());
        data.extend_from_slice(&self.balance.to_le_bytes());
        data
    }
}

impl OptInEvent {
    pub const DATA_LEN: usize = 32 + 32 + 8; // reward_pool + user + balance

    #[inline(always)]
    pub fn new(reward_pool: Address, user: Address, balance: u64) -> Self {
        Self { reward_pool, user, balance }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_opt_in_event_new() {
        let reward_pool = Address::new_from_array([1u8; 32]);
        let user = Address::new_from_array([2u8; 32]);

        let event = OptInEvent::new(reward_pool, user, 1_000_000);

        assert_eq!(event.reward_pool, reward_pool);
        assert_eq!(event.user, user);
        assert_eq!(event.balance, 1_000_000);
    }

    #[test]
    fn test_opt_in_event_to_bytes_inner() {
        let reward_pool = Address::new_from_array([1u8; 32]);
        let user = Address::new_from_array([2u8; 32]);
        let event = OptInEvent::new(reward_pool, user, 500_000);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), OptInEvent::DATA_LEN);
        assert_eq!(&bytes[..32], reward_pool.as_ref());
        assert_eq!(&bytes[32..64], user.as_ref());
        assert_eq!(&bytes[64..72], &500_000u64.to_le_bytes());
    }

    #[test]
    fn test_opt_in_event_to_bytes() {
        let reward_pool = Address::new_from_array([1u8; 32]);
        let user = Address::new_from_array([2u8; 32]);
        let event = OptInEvent::new(reward_pool, user, 1_000_000);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + OptInEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::OptedIn as u8);
        assert_eq!(&bytes[9..41], reward_pool.as_ref());
    }
}
