use alloc::vec;
use alloc::vec::Vec;
use codama::CodamaAccount;
use pinocchio::{account::AccountView, cpi::Seed, error::ProgramError, Address};

use crate::traits::{
    AccountParse, AccountSerialize, AccountSize, AccountValidation, Discriminator, PdaSeeds,
    RewardsAccountDiscriminators, Versioned,
};
use crate::{require_account_len, validate_discriminator, validate_version};

#[derive(Clone, Debug, PartialEq, CodamaAccount)]
pub struct DirectDistributionTombstone {
    pub bump: u8,
}

impl Discriminator for DirectDistributionTombstone {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::DirectDistributionTombstone as u8;
}

impl Versioned for DirectDistributionTombstone {
    const VERSION: u8 = 1;
}

impl AccountSize for DirectDistributionTombstone {
    const DATA_LEN: usize = 1;
}

impl AccountParse for DirectDistributionTombstone {
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        require_account_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);
        validate_version!(data, Self::VERSION);

        Ok(Self { bump: data[2] })
    }
}

impl AccountSerialize for DirectDistributionTombstone {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data
    }
}

impl AccountValidation for DirectDistributionTombstone {}

pub struct DirectDistributionTombstoneSeeds {
    pub distribution: Address,
}

impl PdaSeeds for DirectDistributionTombstoneSeeds {
    const PREFIX: &'static [u8] = b"direct_distribution_tombstone";

    #[inline(always)]
    fn seeds(&self) -> Vec<&[u8]> {
        vec![Self::PREFIX, self.distribution.as_ref()]
    }

    #[inline(always)]
    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
        vec![Seed::from(Self::PREFIX), Seed::from(self.distribution.as_ref()), Seed::from(bump.as_slice())]
    }
}

impl DirectDistributionTombstone {
    #[inline(always)]
    pub fn new(bump: u8) -> Self {
        Self { bump }
    }

    #[inline(always)]
    pub fn from_account(
        data: &[u8],
        account: &AccountView,
        program_id: &Address,
        distribution: &Address,
    ) -> Result<Self, ProgramError> {
        let state = Self::parse_from_bytes(data)?;
        let seeds = DirectDistributionTombstoneSeeds { distribution: *distribution };
        seeds.validate_pda(account, program_id, state.bump)?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tombstone_roundtrip() {
        let tombstone = DirectDistributionTombstone::new(255);
        let bytes = tombstone.to_bytes();
        let decoded = DirectDistributionTombstone::parse_from_bytes(&bytes).unwrap();
        assert_eq!(decoded, tombstone);
    }

    #[test]
    fn test_tombstone_seeds() {
        let seeds = DirectDistributionTombstoneSeeds { distribution: Address::new_from_array([1u8; 32]) };
        let pda_seeds = seeds.seeds();
        assert_eq!(pda_seeds.len(), 2);
        assert_eq!(pda_seeds[0], DirectDistributionTombstoneSeeds::PREFIX);
        assert_eq!(pda_seeds[1], seeds.distribution.as_ref());
    }
}
