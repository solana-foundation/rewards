use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct PointsUsedEvent {
    pub points_config: Address,
    pub authority: Address,
    pub seed: Address,
    pub transferable: u8,
    pub revocable: u8,
    pub user: Address,
    pub quantity: u64,
    pub new_balance: u64,
}

impl EventDiscriminator for PointsUsedEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::PointsUsed as u8;
}

impl EventSerialize for PointsUsedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.points_config.as_ref());
        data.extend_from_slice(self.authority.as_ref());
        data.extend_from_slice(self.seed.as_ref());
        data.push(self.transferable);
        data.push(self.revocable);
        data.extend_from_slice(self.user.as_ref());
        data.extend_from_slice(&self.quantity.to_le_bytes());
        data.extend_from_slice(&self.new_balance.to_le_bytes());
        data
    }
}

impl PointsUsedEvent {
    // 32 + 32 + 32 + 1 + 1 + 32 + 8 + 8 = 146
    pub const DATA_LEN: usize = 32 + 32 + 32 + 1 + 1 + 32 + 8 + 8;

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        points_config: Address,
        authority: Address,
        seed: Address,
        transferable: u8,
        revocable: u8,
        user: Address,
        quantity: u64,
        new_balance: u64,
    ) -> Self {
        Self { points_config, authority, seed, transferable, revocable, user, quantity, new_balance }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_points_used_event() {
        let config = Address::new_from_array([1u8; 32]);
        let authority = Address::new_from_array([3u8; 32]);
        let seed = Address::new_from_array([4u8; 32]);
        let user = Address::new_from_array([2u8; 32]);
        let event = PointsUsedEvent::new(config, authority, seed, 1, 0, user, 200, 300);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), PointsUsedEvent::DATA_LEN);
    }

    #[test]
    fn test_points_used_event_to_bytes() {
        let config = Address::new_from_array([1u8; 32]);
        let authority = Address::new_from_array([3u8; 32]);
        let seed = Address::new_from_array([4u8; 32]);
        let user = Address::new_from_array([2u8; 32]);
        let event = PointsUsedEvent::new(config, authority, seed, 1, 1, user, 100, 400);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + PointsUsedEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::PointsUsed as u8);
    }
}
