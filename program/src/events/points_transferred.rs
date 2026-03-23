use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct PointsTransferredEvent {
    pub points_config: Address,
    pub from: Address,
    pub to: Address,
    pub quantity: u64,
}

impl EventDiscriminator for PointsTransferredEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::PointsTransferred as u8;
}

impl EventSerialize for PointsTransferredEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.points_config.as_ref());
        data.extend_from_slice(self.from.as_ref());
        data.extend_from_slice(self.to.as_ref());
        data.extend_from_slice(&self.quantity.to_le_bytes());
        data
    }
}

impl PointsTransferredEvent {
    pub const DATA_LEN: usize = 32 + 32 + 32 + 8; // 104

    #[inline(always)]
    pub fn new(points_config: Address, from: Address, to: Address, quantity: u64) -> Self {
        Self { points_config, from, to, quantity }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_points_transferred_event() {
        let config = Address::new_from_array([1u8; 32]);
        let from = Address::new_from_array([2u8; 32]);
        let to = Address::new_from_array([3u8; 32]);
        let event = PointsTransferredEvent::new(config, from, to, 100);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), PointsTransferredEvent::DATA_LEN);
        assert_eq!(&bytes[..32], config.as_ref());
        assert_eq!(&bytes[32..64], from.as_ref());
        assert_eq!(&bytes[64..96], to.as_ref());
        assert_eq!(&bytes[96..104], &100u64.to_le_bytes());
    }

    #[test]
    fn test_points_transferred_event_to_bytes() {
        let config = Address::new_from_array([1u8; 32]);
        let from = Address::new_from_array([2u8; 32]);
        let to = Address::new_from_array([3u8; 32]);
        let event = PointsTransferredEvent::new(config, from, to, 100);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + PointsTransferredEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::PointsTransferred as u8);
    }
}
