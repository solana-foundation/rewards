use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct PointsRevokedEvent {
    pub points_config: Address,
    pub user: Address,
    pub revoked_balance: u64,
}

impl EventDiscriminator for PointsRevokedEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::PointsRevoked as u8;
}

impl EventSerialize for PointsRevokedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.points_config.as_ref());
        data.extend_from_slice(self.user.as_ref());
        data.extend_from_slice(&self.revoked_balance.to_le_bytes());
        data
    }
}

impl PointsRevokedEvent {
    pub const DATA_LEN: usize = 32 + 32 + 8; // 72

    #[inline(always)]
    pub fn new(points_config: Address, user: Address, revoked_balance: u64) -> Self {
        Self { points_config, user, revoked_balance }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_points_revoked_event() {
        let config = Address::new_from_array([1u8; 32]);
        let user = Address::new_from_array([2u8; 32]);
        let event = PointsRevokedEvent::new(config, user, 500);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), PointsRevokedEvent::DATA_LEN);
        assert_eq!(&bytes[..32], config.as_ref());
        assert_eq!(&bytes[32..64], user.as_ref());
        assert_eq!(&bytes[64..72], &500u64.to_le_bytes());
    }

    #[test]
    fn test_points_revoked_event_to_bytes() {
        let config = Address::new_from_array([1u8; 32]);
        let user = Address::new_from_array([2u8; 32]);
        let event = PointsRevokedEvent::new(config, user, 500);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + PointsRevokedEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::PointsRevoked as u8);
    }
}
