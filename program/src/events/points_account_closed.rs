use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct PointsAccountClosedEvent {
    pub points_config: Address,
    pub user: Address,
}

impl EventDiscriminator for PointsAccountClosedEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::PointsAccountClosed as u8;
}

impl EventSerialize for PointsAccountClosedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.points_config.as_ref());
        data.extend_from_slice(self.user.as_ref());
        data
    }
}

impl PointsAccountClosedEvent {
    pub const DATA_LEN: usize = 32 + 32; // 64

    #[inline(always)]
    pub fn new(points_config: Address, user: Address) -> Self {
        Self { points_config, user }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_points_account_closed_event() {
        let config = Address::new_from_array([1u8; 32]);
        let user = Address::new_from_array([2u8; 32]);
        let event = PointsAccountClosedEvent::new(config, user);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), PointsAccountClosedEvent::DATA_LEN);
        assert_eq!(&bytes[..32], config.as_ref());
        assert_eq!(&bytes[32..64], user.as_ref());
    }

    #[test]
    fn test_points_account_closed_event_to_bytes() {
        let config = Address::new_from_array([1u8; 32]);
        let user = Address::new_from_array([2u8; 32]);
        let event = PointsAccountClosedEvent::new(config, user);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + PointsAccountClosedEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::PointsAccountClosed as u8);
    }
}
