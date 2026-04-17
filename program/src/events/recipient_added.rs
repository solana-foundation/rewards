use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::{
    traits::{EventDiscriminator, EventSerialize},
    utils::VestingSchedule,
};

#[derive(CodamaType)]
pub struct RecipientAddedEvent {
    pub distribution: Address,
    pub recipient: Address,
    pub amount: u64,
    pub schedule: VestingSchedule,
}

impl EventDiscriminator for RecipientAddedEvent {
    const DISCRIMINATOR: [u8; 8] = [0xda, 0xf1, 0x3c, 0xf4, 0x08, 0x58, 0xbe, 0x61];
}

impl EventSerialize for RecipientAddedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let schedule_bytes = self.schedule.to_bytes();
        let mut data = Vec::with_capacity(Self::BASE_DATA_LEN + schedule_bytes.len());
        data.extend_from_slice(self.distribution.as_ref());
        data.extend_from_slice(self.recipient.as_ref());
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.extend_from_slice(&schedule_bytes);
        data
    }
}

impl RecipientAddedEvent {
    /// distribution(32) + recipient(32) + amount(8)
    pub const BASE_DATA_LEN: usize = 32 + 32 + 8;

    #[inline(always)]
    pub fn new(distribution: Address, recipient: Address, amount: u64, schedule: VestingSchedule) -> Self {
        Self { distribution, recipient, amount, schedule }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_recipient_added_event_new() {
        let distribution = Address::new_from_array([1u8; 32]);
        let recipient = Address::new_from_array([2u8; 32]);
        let schedule = VestingSchedule::Linear { start_ts: 100, end_ts: 200 };

        let event = RecipientAddedEvent::new(distribution, recipient, 1000, schedule);

        assert_eq!(event.distribution, distribution);
        assert_eq!(event.recipient, recipient);
        assert_eq!(event.amount, 1000);
        assert_eq!(event.schedule, schedule);
    }

    #[test]
    fn test_recipient_added_event_to_bytes_inner_immediate() {
        let distribution = Address::new_from_array([1u8; 32]);
        let recipient = Address::new_from_array([2u8; 32]);
        let event = RecipientAddedEvent::new(distribution, recipient, 5000, VestingSchedule::Immediate {});

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), RecipientAddedEvent::BASE_DATA_LEN + 1); // schedule = 1 byte
        assert_eq!(&bytes[..32], distribution.as_ref());
        assert_eq!(&bytes[32..64], recipient.as_ref());
        assert_eq!(&bytes[64..72], &5000u64.to_le_bytes());
        assert_eq!(bytes[72], 0); // Immediate discriminant
    }

    #[test]
    fn test_recipient_added_event_to_bytes_inner_linear() {
        let distribution = Address::new_from_array([1u8; 32]);
        let recipient = Address::new_from_array([2u8; 32]);
        let schedule = VestingSchedule::Linear { start_ts: 100, end_ts: 200 };
        let event = RecipientAddedEvent::new(distribution, recipient, 5000, schedule);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), RecipientAddedEvent::BASE_DATA_LEN + 17); // schedule = 17 bytes
    }

    #[test]
    fn test_recipient_added_event_to_bytes() {
        let distribution = Address::new_from_array([1u8; 32]);
        let recipient = Address::new_from_array([2u8; 32]);
        let schedule = VestingSchedule::Linear { start_ts: 100, end_ts: 200 };
        let event = RecipientAddedEvent::new(distribution, recipient, 1000, schedule);

        let bytes = event.to_bytes();
        let expected_len = EVENT_DISCRIMINATOR_LEN + RecipientAddedEvent::BASE_DATA_LEN + 17;
        assert_eq!(bytes.len(), expected_len);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(&bytes[8..16], RecipientAddedEvent::DISCRIMINATOR);
    }

    #[test]
    fn test_recipient_added_event_with_cliff_linear() {
        let distribution = Address::new_from_array([1u8; 32]);
        let recipient = Address::new_from_array([2u8; 32]);
        let schedule = VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 };
        let event = RecipientAddedEvent::new(distribution, recipient, 1000, schedule);

        assert_eq!(event.schedule, schedule);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), RecipientAddedEvent::BASE_DATA_LEN + 25); // schedule = 25 bytes
    }
}
