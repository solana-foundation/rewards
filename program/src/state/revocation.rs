use alloc::vec;
use alloc::vec::Vec;
use codama::CodamaAccount;
use pinocchio::{account::AccountView, cpi::Seed, error::ProgramError, Address};

use crate::traits::{
    AccountParse, AccountSerialize, AccountSize, AccountValidation, Discriminator, PdaSeeds,
    RewardsAccountDiscriminators, Versioned,
};
use crate::{require_account_len, validate_discriminator};

/// Revocation account state
///
/// Minimal PDA recording that a user has been revoked from a distribution
/// or reward pool. Its existence blocks future claims or opt-ins.
/// Shared across merkle distributions and continuous reward pools.
///
/// # PDA Seeds
/// `[b"revocation", parent.as_ref(), user.as_ref()]`
///
/// Where `parent` is the distribution or reward pool address.
#[derive(Clone, Debug, PartialEq, CodamaAccount)]
pub struct Revocation {
    pub bump: u8,
}

impl Discriminator for Revocation {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::Revocation as u8;
}

impl Versioned for Revocation {
    const VERSION: u8 = 1;
}

impl AccountSize for Revocation {
    const DATA_LEN: usize = 1; // bump
}

impl AccountParse for Revocation {
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        require_account_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);

        // Skip discriminator (byte 0) and version (byte 1)
        let data = &data[2..];

        let bump = data[0];

        Ok(Self { bump })
    }
}

impl AccountSerialize for Revocation {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data
    }
}

impl AccountValidation for Revocation {}

/// Seed helper for deriving Revocation PDA
///
/// `parent` is the distribution (merkle/direct) or reward pool (continuous) address.
/// `user` is the claimant or user address.
pub struct RevocationSeeds {
    pub parent: Address,
    pub user: Address,
}

impl PdaSeeds for RevocationSeeds {
    const PREFIX: &'static [u8] = b"revocation";

    #[inline(always)]
    fn seeds(&self) -> Vec<&[u8]> {
        vec![Self::PREFIX, self.parent.as_ref(), self.user.as_ref()]
    }

    #[inline(always)]
    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
        vec![
            Seed::from(Self::PREFIX),
            Seed::from(self.parent.as_ref()),
            Seed::from(self.user.as_ref()),
            Seed::from(bump.as_slice()),
        ]
    }
}

impl Revocation {
    #[inline(always)]
    pub fn new(bump: u8) -> Self {
        Self { bump }
    }

    #[inline(always)]
    pub fn from_account(
        data: &[u8],
        account: &AccountView,
        program_id: &Address,
        parent: &Address,
        user: &Address,
    ) -> Result<Self, ProgramError> {
        let state = Self::parse_from_bytes(data)?;
        let seeds = RevocationSeeds { parent: *parent, user: *user };
        seeds.validate_pda(account, program_id, state.bump)?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_revocation() -> Revocation {
        Revocation::new(255)
    }

    #[test]
    fn test_revocation_new() {
        let revocation = create_test_revocation();
        assert_eq!(revocation.bump, 255);
    }

    #[test]
    fn test_revocation_to_bytes_inner() {
        let revocation = create_test_revocation();
        let bytes = revocation.to_bytes_inner();

        assert_eq!(bytes.len(), Revocation::DATA_LEN);
        assert_eq!(bytes[0], 255);
    }

    #[test]
    fn test_revocation_to_bytes() {
        let revocation = create_test_revocation();
        let bytes = revocation.to_bytes();

        assert_eq!(bytes.len(), Revocation::LEN);
        assert_eq!(bytes[0], Revocation::DISCRIMINATOR);
        assert_eq!(bytes[1], Revocation::VERSION);
        assert_eq!(bytes[2], 255);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let revocation = create_test_revocation();

        let bytes = revocation.to_bytes();
        let deserialized = Revocation::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.bump, revocation.bump);
    }

    #[test]
    fn test_revocation_seeds_pda_seeds() {
        let seeds =
            RevocationSeeds { parent: Address::new_from_array([1u8; 32]), user: Address::new_from_array([2u8; 32]) };
        let pda_seeds = seeds.seeds();
        assert_eq!(pda_seeds.len(), 3);
        assert_eq!(pda_seeds[0], RevocationSeeds::PREFIX);
        assert_eq!(pda_seeds[1], seeds.parent.as_ref());
        assert_eq!(pda_seeds[2], seeds.user.as_ref());
    }

    #[test]
    fn test_account_size() {
        assert_eq!(Revocation::DATA_LEN, 1);
        assert_eq!(Revocation::LEN, 3);
    }
}
