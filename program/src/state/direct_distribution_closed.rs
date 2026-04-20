use alloc::vec::Vec;
use codama::CodamaAccount;
use pinocchio::error::ProgramError;

use crate::traits::{
    AccountParse, AccountSerialize, AccountSize, AccountValidation, Discriminator, RewardsAccountDiscriminators,
    Versioned,
};
use crate::{require_account_len, validate_discriminator, validate_version};

/// `DirectDistributionClosed` is the permanently-closed state of a direct
/// distribution PDA. The same PDA address is reused — only the discriminator
/// flips on close — so no separate tombstone account is needed.
///
/// After close, the distribution account is resized down to `LEN` (3 bytes:
/// discriminator + version + bump), and the freed rent is refunded to the
/// authority. On subsequent `create_direct_distribution` calls, the presence
/// of this discriminator at the PDA address triggers `DistributionPermanentlyClosed`.
#[derive(Clone, Debug, PartialEq, CodamaAccount)]
#[repr(C)]
pub struct DirectDistributionClosed {
    pub bump: u8,
}

impl Discriminator for DirectDistributionClosed {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::DirectDistributionClosed as u8;
}

impl Versioned for DirectDistributionClosed {
    const VERSION: u8 = 1;
}

impl AccountSize for DirectDistributionClosed {
    const DATA_LEN: usize = 1; // bump
}

impl AccountParse for DirectDistributionClosed {
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        require_account_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);
        validate_version!(data, Self::VERSION);

        Ok(Self { bump: data[2] })
    }
}

impl AccountSerialize for DirectDistributionClosed {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data
    }
}

impl AccountValidation for DirectDistributionClosed {}

impl DirectDistributionClosed {
    #[inline(always)]
    pub fn new(bump: u8) -> Self {
        Self { bump }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_distribution_closed_roundtrip() {
        let closed = DirectDistributionClosed::new(255);
        let bytes = closed.to_bytes();
        assert_eq!(bytes.len(), DirectDistributionClosed::LEN);
        assert_eq!(bytes[0], DirectDistributionClosed::DISCRIMINATOR);
        assert_eq!(bytes[1], DirectDistributionClosed::VERSION);
        assert_eq!(bytes[2], 255);

        let decoded = DirectDistributionClosed::parse_from_bytes(&bytes).unwrap();
        assert_eq!(decoded, closed);
    }

    #[test]
    fn test_parse_rejects_wrong_discriminator() {
        let closed = DirectDistributionClosed::new(100);
        let mut bytes = closed.to_bytes();
        bytes[0] = 0xFF; // corrupt discriminator
        assert!(DirectDistributionClosed::parse_from_bytes(&bytes).is_err());
    }
}
