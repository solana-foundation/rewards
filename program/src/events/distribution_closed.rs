use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventSerialize};

#[derive(CodamaType)]
pub struct DistributionClosedEvent {
    pub distribution: Address,
    pub remaining_amount: u64,
}

impl EventDiscriminator for DistributionClosedEvent {
    const DISCRIMINATOR: [u8; 8] = [0x8a, 0x16, 0xe6, 0xc0, 0x81, 0x40, 0xd1, 0x01];
}

impl EventSerialize for DistributionClosedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.distribution.as_ref());
        data.extend_from_slice(&self.remaining_amount.to_le_bytes());
        data
    }
}

impl DistributionClosedEvent {
    pub const DATA_LEN: usize = 32 + 8; // distribution + remaining_amount

    #[inline(always)]
    pub fn new(distribution: Address, remaining_amount: u64) -> Self {
        Self { distribution, remaining_amount }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_distribution_closed_event_new() {
        let distribution = Address::new_from_array([1u8; 32]);

        let event = DistributionClosedEvent::new(distribution, 500);

        assert_eq!(event.distribution, distribution);
        assert_eq!(event.remaining_amount, 500);
    }

    #[test]
    fn test_distribution_closed_event_to_bytes_inner() {
        let distribution = Address::new_from_array([1u8; 32]);
        let event = DistributionClosedEvent::new(distribution, 500);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), DistributionClosedEvent::DATA_LEN);
        assert_eq!(&bytes[..32], distribution.as_ref());
        assert_eq!(&bytes[32..40], &500u64.to_le_bytes());
    }

    #[test]
    fn test_distribution_closed_event_to_bytes() {
        let distribution = Address::new_from_array([1u8; 32]);
        let event = DistributionClosedEvent::new(distribution, 0);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + DistributionClosedEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(&bytes[8..16], DistributionClosedEvent::DISCRIMINATOR);
        assert_eq!(&bytes[16..48], distribution.as_ref());
    }
}
