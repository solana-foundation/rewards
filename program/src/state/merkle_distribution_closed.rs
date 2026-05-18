use alloc::vec::Vec;
use codama::CodamaAccount;
use pinocchio::error::ProgramError;

use crate::traits::{
    AccountParse, AccountSerialize, AccountSize, AccountValidation, Discriminator, RewardsAccountDiscriminators,
    Versioned,
};
use crate::{require_account_len, validate_discriminator, validate_version};

#[derive(Clone, Debug, PartialEq, CodamaAccount)]
#[repr(C)]
pub struct MerkleDistributionClosed {
    pub bump: u8,
}

impl Discriminator for MerkleDistributionClosed {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::MerkleDistributionClosed as u8;
}

impl Versioned for MerkleDistributionClosed {
    const VERSION: u8 = 1;
}

impl AccountSize for MerkleDistributionClosed {
    const DATA_LEN: usize = 1;
}

impl AccountParse for MerkleDistributionClosed {
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        require_account_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);
        validate_version!(data, Self::VERSION);

        Ok(Self { bump: data[2] })
    }
}

impl AccountSerialize for MerkleDistributionClosed {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data
    }
}

impl AccountValidation for MerkleDistributionClosed {}

impl MerkleDistributionClosed {
    #[inline(always)]
    pub fn new(bump: u8) -> Self {
        Self { bump }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_distribution_closed_roundtrip() {
        let closed = MerkleDistributionClosed::new(255);
        let bytes = closed.to_bytes();
        assert_eq!(bytes.len(), MerkleDistributionClosed::LEN);
        assert_eq!(bytes[0], MerkleDistributionClosed::DISCRIMINATOR);
        assert_eq!(bytes[1], MerkleDistributionClosed::VERSION);
        assert_eq!(bytes[2], 255);

        let decoded = MerkleDistributionClosed::parse_from_bytes(&bytes).unwrap();
        assert_eq!(decoded, closed);
    }

    #[test]
    fn test_parse_rejects_wrong_discriminator() {
        let closed = MerkleDistributionClosed::new(100);
        let mut bytes = closed.to_bytes();
        bytes[0] = 0xFF;
        assert!(MerkleDistributionClosed::parse_from_bytes(&bytes).is_err());
    }
}
