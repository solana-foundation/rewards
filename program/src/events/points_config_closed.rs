use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct PointsConfigClosedEvent {
    pub points_config: Address,
}

impl EventDiscriminator for PointsConfigClosedEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::PointsConfigClosed as u8;
}

impl EventSerialize for PointsConfigClosedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.points_config.as_ref());
        data
    }
}

impl PointsConfigClosedEvent {
    pub const DATA_LEN: usize = 32;

    #[inline(always)]
    pub fn new(points_config: Address) -> Self {
        Self { points_config }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_points_config_closed_event() {
        let config = Address::new_from_array([1u8; 32]);
        let event = PointsConfigClosedEvent::new(config);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), PointsConfigClosedEvent::DATA_LEN);
        assert_eq!(&bytes[..32], config.as_ref());
    }

    #[test]
    fn test_points_config_closed_event_to_bytes() {
        let config = Address::new_from_array([1u8; 32]);
        let event = PointsConfigClosedEvent::new(config);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + PointsConfigClosedEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::PointsConfigClosed as u8);
    }
}
