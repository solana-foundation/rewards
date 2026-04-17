use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventSerialize};

#[derive(CodamaType)]
pub struct ClaimClosedEvent {
    pub distribution: Address,
    pub claimant: Address,
}

impl EventDiscriminator for ClaimClosedEvent {
    const DISCRIMINATOR: [u8; 8] = [0x8b, 0xa6, 0xd7, 0xc3, 0xb8, 0xe9, 0x1a, 0xd9];
}

impl EventSerialize for ClaimClosedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.distribution.as_ref());
        data.extend_from_slice(self.claimant.as_ref());
        data
    }
}

impl ClaimClosedEvent {
    pub const DATA_LEN: usize = 32 + 32; // distribution + claimant

    #[inline(always)]
    pub fn new(distribution: Address, claimant: Address) -> Self {
        Self { distribution, claimant }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_claim_closed_event_new() {
        let distribution = Address::new_from_array([1u8; 32]);
        let claimant = Address::new_from_array([2u8; 32]);

        let event = ClaimClosedEvent::new(distribution, claimant);

        assert_eq!(event.distribution, distribution);
        assert_eq!(event.claimant, claimant);
    }

    #[test]
    fn test_claim_closed_event_to_bytes_inner() {
        let distribution = Address::new_from_array([1u8; 32]);
        let claimant = Address::new_from_array([2u8; 32]);
        let event = ClaimClosedEvent::new(distribution, claimant);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), ClaimClosedEvent::DATA_LEN);
        assert_eq!(&bytes[..32], distribution.as_ref());
        assert_eq!(&bytes[32..64], claimant.as_ref());
    }

    #[test]
    fn test_claim_closed_event_to_bytes() {
        let distribution = Address::new_from_array([1u8; 32]);
        let claimant = Address::new_from_array([2u8; 32]);
        let event = ClaimClosedEvent::new(distribution, claimant);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + ClaimClosedEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(&bytes[8..16], ClaimClosedEvent::DISCRIMINATOR);
        assert_eq!(&bytes[16..48], distribution.as_ref());
    }
}
