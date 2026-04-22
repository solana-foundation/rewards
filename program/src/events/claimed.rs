use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{event_discriminator, EventDiscriminator, EventSerialize};

#[derive(CodamaType)]
pub struct ClaimedEvent {
    pub distribution: Address,
    pub claimant: Address,
    pub amount: u64,
}

impl EventDiscriminator for ClaimedEvent {
    const DISCRIMINATOR: [u8; 8] = event_discriminator(b"ClaimedEvent");
}

impl EventSerialize for ClaimedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.distribution.as_ref());
        data.extend_from_slice(self.claimant.as_ref());
        data.extend_from_slice(&self.amount.to_le_bytes());
        data
    }
}

impl ClaimedEvent {
    pub const DATA_LEN: usize = 32 + 32 + 8; // distribution + claimant + amount

    #[inline(always)]
    pub fn new(distribution: Address, claimant: Address, amount: u64) -> Self {
        Self { distribution, claimant, amount }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_claimed_event_new() {
        let distribution = Address::new_from_array([1u8; 32]);
        let claimant = Address::new_from_array([2u8; 32]);

        let event = ClaimedEvent::new(distribution, claimant, 1000);

        assert_eq!(event.distribution, distribution);
        assert_eq!(event.claimant, claimant);
        assert_eq!(event.amount, 1000);
    }

    #[test]
    fn test_claimed_event_to_bytes_inner() {
        let distribution = Address::new_from_array([1u8; 32]);
        let claimant = Address::new_from_array([2u8; 32]);
        let event = ClaimedEvent::new(distribution, claimant, 5000);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), ClaimedEvent::DATA_LEN);
        assert_eq!(&bytes[..32], distribution.as_ref());
        assert_eq!(&bytes[32..64], claimant.as_ref());
        assert_eq!(&bytes[64..72], &5000u64.to_le_bytes());
    }

    #[test]
    fn test_claimed_event_to_bytes() {
        let distribution = Address::new_from_array([1u8; 32]);
        let claimant = Address::new_from_array([2u8; 32]);
        let event = ClaimedEvent::new(distribution, claimant, 1000);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + ClaimedEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(&bytes[8..16], ClaimedEvent::DISCRIMINATOR);
        assert_eq!(&bytes[16..48], distribution.as_ref());
    }
}
