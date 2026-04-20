use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{event_discriminator, EventDiscriminator, EventSerialize};

#[derive(CodamaType)]
pub struct PointsTransferredEvent {
    pub points_config: Address,
    pub authority: Address,
    pub seed: Address,
    pub transferable: u8,
    pub revocable: u8,
    pub from: Address,
    pub to: Address,
    pub quantity: u64,
}

impl EventDiscriminator for PointsTransferredEvent {
    const DISCRIMINATOR: [u8; 8] = event_discriminator(b"PointsTransferredEvent");
}

impl EventSerialize for PointsTransferredEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.points_config.as_ref());
        data.extend_from_slice(self.authority.as_ref());
        data.extend_from_slice(self.seed.as_ref());
        data.push(self.transferable);
        data.push(self.revocable);
        data.extend_from_slice(self.from.as_ref());
        data.extend_from_slice(self.to.as_ref());
        data.extend_from_slice(&self.quantity.to_le_bytes());
        data
    }
}

impl PointsTransferredEvent {
    // 32 + 32 + 32 + 1 + 1 + 32 + 32 + 8 = 170
    pub const DATA_LEN: usize = 32 + 32 + 32 + 1 + 1 + 32 + 32 + 8;

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        points_config: Address,
        authority: Address,
        seed: Address,
        transferable: u8,
        revocable: u8,
        from: Address,
        to: Address,
        quantity: u64,
    ) -> Self {
        Self { points_config, authority, seed, transferable, revocable, from, to, quantity }
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
        let authority = Address::new_from_array([4u8; 32]);
        let seed = Address::new_from_array([5u8; 32]);
        let from = Address::new_from_array([2u8; 32]);
        let to = Address::new_from_array([3u8; 32]);
        let event = PointsTransferredEvent::new(config, authority, seed, 1, 0, from, to, 100);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), PointsTransferredEvent::DATA_LEN);
    }

    #[test]
    fn test_points_transferred_event_to_bytes() {
        let config = Address::new_from_array([1u8; 32]);
        let authority = Address::new_from_array([4u8; 32]);
        let seed = Address::new_from_array([5u8; 32]);
        let from = Address::new_from_array([2u8; 32]);
        let to = Address::new_from_array([3u8; 32]);
        let event = PointsTransferredEvent::new(config, authority, seed, 1, 1, from, to, 100);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + PointsTransferredEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(&bytes[8..16], PointsTransferredEvent::DISCRIMINATOR);
    }
}
