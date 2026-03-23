use alloc::vec;
use alloc::vec::Vec;
use codama::CodamaAccount;
use pinocchio::{
    account::AccountView,
    cpi::{Seed, Signer},
    error::ProgramError,
    Address,
};

use crate::errors::RewardsProgramError;
use crate::traits::{
    AccountParse, AccountSerialize, AccountSize, AccountValidation, ClaimTracker, Discriminator, PdaSeeds,
    RewardsAccountDiscriminators, Versioned,
};
use crate::{assert_no_padding, require_account_len, validate_discriminator, validate_version};

/// MerkleClaim account state
///
/// Minimal PDA tracking how much a user has claimed from a merkle distribution.
/// Rent is paid by claimant and refundable when claim is closed.
///
/// # PDA Seeds
/// `[b"merkle_claim", distribution.as_ref(), claimant.as_ref()]`
#[derive(Clone, Debug, PartialEq, CodamaAccount)]
#[repr(C)]
pub struct MerkleClaim {
    pub bump: u8,
    _padding: [u8; 7],
    pub claimed_amount: u64,
}

assert_no_padding!(MerkleClaim, 1 + 7 + 8);

impl Discriminator for MerkleClaim {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::MerkleClaim as u8;
}

impl Versioned for MerkleClaim {
    const VERSION: u8 = 1;
}

impl AccountSize for MerkleClaim {
    const DATA_LEN: usize = 1 + 7 + 8; // 16
}

impl AccountParse for MerkleClaim {
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        require_account_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);
        validate_version!(data, Self::VERSION);

        // Skip discriminator (byte 0) and version (byte 1)
        let data = &data[2..];

        let bump = data[0];
        // Skip padding bytes [1..8]
        let claimed_amount =
            u64::from_le_bytes(data[8..16].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);

        Ok(Self { bump, _padding: [0u8; 7], claimed_amount })
    }
}

impl AccountSerialize for MerkleClaim {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data.extend_from_slice(&[0u8; 7]); // padding
        data.extend_from_slice(&self.claimed_amount.to_le_bytes());
        data
    }
}

impl AccountValidation for MerkleClaim {}

/// Seed helper for deriving MerkleClaim PDA without having the full state
pub struct MerkleClaimSeeds {
    pub distribution: Address,
    pub claimant: Address,
}

impl PdaSeeds for MerkleClaimSeeds {
    const PREFIX: &'static [u8] = b"merkle_claim";

    #[inline(always)]
    fn seeds(&self) -> Vec<&[u8]> {
        vec![Self::PREFIX, self.distribution.as_ref(), self.claimant.as_ref()]
    }

    #[inline(always)]
    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
        vec![
            Seed::from(Self::PREFIX),
            Seed::from(self.distribution.as_ref()),
            Seed::from(self.claimant.as_ref()),
            Seed::from(bump.as_slice()),
        ]
    }
}

impl ClaimTracker for MerkleClaim {
    #[inline(always)]
    fn claimed_amount(&self) -> u64 {
        self.claimed_amount
    }

    #[inline(always)]
    fn set_claimed_amount(&mut self, amount: u64) -> Result<(), ProgramError> {
        if amount < self.claimed_amount {
            return Err(RewardsProgramError::ClaimedAmountDecreased.into());
        }
        self.claimed_amount = amount;
        Ok(())
    }
}

impl MerkleClaim {
    #[inline(always)]
    pub fn new(bump: u8) -> Self {
        Self { bump, _padding: [0u8; 7], claimed_amount: 0 }
    }

    #[inline(always)]
    pub fn from_account(
        data: &[u8],
        account: &AccountView,
        program_id: &Address,
        distribution: &Address,
        claimant: &Address,
    ) -> Result<Self, ProgramError> {
        let state = Self::parse_from_bytes(data)?;
        let seeds = MerkleClaimSeeds { distribution: *distribution, claimant: *claimant };
        seeds.validate_pda(account, program_id, state.bump)?;
        Ok(state)
    }

    #[inline(always)]
    pub fn with_signer<F, R>(&self, distribution: &Address, claimant: &Address, f: F) -> R
    where
        F: FnOnce(&[Signer<'_, '_>]) -> R,
    {
        let bump_seed = [self.bump];
        let seeds = [
            Seed::from(MerkleClaimSeeds::PREFIX),
            Seed::from(distribution.as_ref()),
            Seed::from(claimant.as_ref()),
            Seed::from(bump_seed.as_slice()),
        ];
        let signers = [Signer::from(&seeds)];
        f(&signers)
    }

    pub fn remaining_amount(&self, total_amount: u64) -> Result<u64, RewardsProgramError> {
        total_amount.checked_sub(self.claimed_amount).ok_or(RewardsProgramError::MathOverflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::ClaimTracker;

    fn create_test_claim() -> MerkleClaim {
        MerkleClaim::new(255)
    }

    #[test]
    fn test_merkle_claim_new() {
        let claim = create_test_claim();
        assert_eq!(claim.bump, 255);
        assert_eq!(claim.claimed_amount, 0);
    }

    #[test]
    fn test_merkle_claim_to_bytes_inner() {
        let claim = create_test_claim();
        let bytes = claim.to_bytes_inner();

        assert_eq!(bytes.len(), MerkleClaim::DATA_LEN);
        assert_eq!(bytes[0], 255); // bump
        assert_eq!(&bytes[1..8], &[0u8; 7]); // padding
    }

    #[test]
    fn test_merkle_claim_to_bytes() {
        let claim = create_test_claim();
        let bytes = claim.to_bytes();

        assert_eq!(bytes.len(), MerkleClaim::LEN);
        assert_eq!(bytes[0], MerkleClaim::DISCRIMINATOR);
        assert_eq!(bytes[1], MerkleClaim::VERSION);
        assert_eq!(bytes[2], 255); // bump
    }

    #[test]
    fn test_roundtrip_serialization() {
        let mut claim = create_test_claim();
        claim.claimed_amount = 500_000;

        let bytes = claim.to_bytes();
        let deserialized = MerkleClaim::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.bump, claim.bump);
        assert_eq!(deserialized.claimed_amount, claim.claimed_amount);
    }

    #[test]
    fn test_parse_rejects_invalid_version() {
        let claim = create_test_claim();
        let mut bytes = claim.to_bytes();
        bytes[1] = MerkleClaim::VERSION + 1;

        let result = MerkleClaim::parse_from_bytes(&bytes);
        assert_eq!(result, Err(ProgramError::InvalidAccountData));
    }

    #[test]
    fn test_merkle_claim_seeds_pda_seeds() {
        let seeds = MerkleClaimSeeds {
            distribution: Address::new_from_array([1u8; 32]),
            claimant: Address::new_from_array([2u8; 32]),
        };
        let pda_seeds = seeds.seeds();
        assert_eq!(pda_seeds.len(), 3);
        assert_eq!(pda_seeds[0], MerkleClaimSeeds::PREFIX);
        assert_eq!(pda_seeds[1], seeds.distribution.as_ref());
        assert_eq!(pda_seeds[2], seeds.claimant.as_ref());
    }

    #[test]
    fn test_claimable_amount() {
        let claim = create_test_claim();
        assert_eq!(ClaimTracker::claimable_amount(&claim, 500).unwrap(), 500);
    }

    #[test]
    fn test_claimable_amount_with_prior_claims() {
        let mut claim = create_test_claim();
        claim.claimed_amount = 200;
        assert_eq!(ClaimTracker::claimable_amount(&claim, 500).unwrap(), 300);
    }

    #[test]
    fn test_claimable_amount_overflow() {
        let mut claim = create_test_claim();
        claim.claimed_amount = 600;
        assert!(ClaimTracker::claimable_amount(&claim, 500).is_err());
    }

    #[test]
    fn test_claim_tracker_trait() {
        let mut claim = create_test_claim();
        assert_eq!(ClaimTracker::claimed_amount(&claim), 0);
        ClaimTracker::set_claimed_amount(&mut claim, 500).unwrap();
        assert_eq!(ClaimTracker::claimed_amount(&claim), 500);
    }

    #[test]
    fn test_set_claimed_amount_rejects_decrease() {
        let mut claim = create_test_claim();
        ClaimTracker::set_claimed_amount(&mut claim, 500).unwrap();
        assert!(ClaimTracker::set_claimed_amount(&mut claim, 400).is_err());
        assert_eq!(ClaimTracker::claimed_amount(&claim), 500);
    }

    #[test]
    fn test_remaining_amount() {
        let mut claim = create_test_claim();
        claim.claimed_amount = 300;
        assert_eq!(claim.remaining_amount(1000).unwrap(), 700);
    }

    #[test]
    fn test_remaining_amount_full() {
        let claim = create_test_claim();
        assert_eq!(claim.remaining_amount(1000).unwrap(), 1000);
    }

    #[test]
    fn test_remaining_amount_overflow() {
        let mut claim = create_test_claim();
        claim.claimed_amount = 1500;
        assert!(claim.remaining_amount(1000).is_err());
    }
}
