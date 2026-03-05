use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};
use crate::utils::RevokeMode;

#[derive(CodamaType)]
pub struct RecipientRevokedEvent {
    pub distribution: Address,
    pub recipient: Address,
    pub revoke_mode: RevokeMode,
    pub vested_transferred: u64,
    pub unvested_returned: u64,
}

impl EventDiscriminator for RecipientRevokedEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::RecipientRevoked as u8;
}

impl EventSerialize for RecipientRevokedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.distribution.as_ref());
        data.extend_from_slice(self.recipient.as_ref());
        data.push(self.revoke_mode.to_byte());
        data.extend_from_slice(&self.vested_transferred.to_le_bytes());
        data.extend_from_slice(&self.unvested_returned.to_le_bytes());
        data
    }
}

impl RecipientRevokedEvent {
    pub const DATA_LEN: usize = 32 + 32 + 1 + 8 + 8; // distribution + recipient + revoke_mode + vested_transferred + unvested_returned

    #[inline(always)]
    pub fn new(
        distribution: Address,
        recipient: Address,
        revoke_mode: RevokeMode,
        vested_transferred: u64,
        unvested_returned: u64,
    ) -> Self {
        Self { distribution, recipient, revoke_mode, vested_transferred, unvested_returned }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_recipient_revoked_event_new() {
        let distribution = Address::new_from_array([1u8; 32]);
        let recipient = Address::new_from_array([2u8; 32]);

        let event = RecipientRevokedEvent::new(distribution, recipient, RevokeMode::NonVested, 300, 500);

        assert_eq!(event.distribution, distribution);
        assert_eq!(event.recipient, recipient);
        assert_eq!(event.revoke_mode, RevokeMode::NonVested);
        assert_eq!(event.vested_transferred, 300);
        assert_eq!(event.unvested_returned, 500);
    }

    #[test]
    fn test_recipient_revoked_event_to_bytes_inner() {
        let distribution = Address::new_from_array([1u8; 32]);
        let recipient = Address::new_from_array([2u8; 32]);
        let event = RecipientRevokedEvent::new(distribution, recipient, RevokeMode::Full, 0, 1000);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), RecipientRevokedEvent::DATA_LEN);
        assert_eq!(&bytes[..32], distribution.as_ref());
        assert_eq!(&bytes[32..64], recipient.as_ref());
        assert_eq!(bytes[64], 1);
        assert_eq!(&bytes[65..73], &0u64.to_le_bytes());
        assert_eq!(&bytes[73..81], &1000u64.to_le_bytes());
    }

    #[test]
    fn test_recipient_revoked_event_to_bytes() {
        let distribution = Address::new_from_array([1u8; 32]);
        let recipient = Address::new_from_array([2u8; 32]);
        let event = RecipientRevokedEvent::new(distribution, recipient, RevokeMode::NonVested, 300, 500);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + RecipientRevokedEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::RecipientRevoked as u8);
        assert_eq!(&bytes[9..41], distribution.as_ref());
    }
}
