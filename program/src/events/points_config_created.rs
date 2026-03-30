use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct PointsConfigCreatedEvent {
    pub authority: Address,
    pub seed: Address,
    pub transferable: u8,
    pub revocable: u8,
}

impl EventDiscriminator for PointsConfigCreatedEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::PointsConfigCreated as u8;
}

impl EventSerialize for PointsConfigCreatedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.authority.as_ref());
        data.extend_from_slice(self.seed.as_ref());
        data.push(self.transferable);
        data.push(self.revocable);
        data
    }
}

impl PointsConfigCreatedEvent {
    pub const DATA_LEN: usize = 32 + 32 + 1 + 1; // 66

    #[inline(always)]
    pub fn new(authority: Address, seed: Address, transferable: u8, revocable: u8) -> Self {
        Self { authority, seed, transferable, revocable }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_points_config_created_event() {
        let authority = Address::new_from_array([1u8; 32]);
        let seed = Address::new_from_array([2u8; 32]);
        let event = PointsConfigCreatedEvent::new(authority, seed, 1, 0);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), PointsConfigCreatedEvent::DATA_LEN);
        assert_eq!(&bytes[..32], authority.as_ref());
        assert_eq!(&bytes[32..64], seed.as_ref());
        assert_eq!(bytes[64], 1);
        assert_eq!(bytes[65], 0);
    }

    #[test]
    fn test_points_config_created_event_to_bytes() {
        let authority = Address::new_from_array([1u8; 32]);
        let seed = Address::new_from_array([2u8; 32]);
        let event = PointsConfigCreatedEvent::new(authority, seed, 1, 1);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + PointsConfigCreatedEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::PointsConfigCreated as u8);
    }
}
