use alloc::vec;
use alloc::vec::Vec;
use codama::CodamaAccount;
use pinocchio::{
    account::AccountView,
    cpi::{Seed, Signer},
    error::ProgramError,
    Address, Resize,
};

use crate::errors::RewardsProgramError;
use crate::state::MerkleDistributionClosed;
use crate::traits::{
    AccountParse, AccountSerialize, AccountSize, AccountValidation, Discriminator, Distribution, DistributionSigner,
    PdaAccount, PdaSeeds, RewardsAccountDiscriminators, Versioned,
};
use crate::utils::refund_excess_rent;
use crate::{assert_no_padding, require_account_len, validate_discriminator, validate_version};

/// MerkleDistribution account state
///
/// Represents a merkle tree-based token distribution where users prove
/// their allocation via merkle proofs. Each user has per-user vesting
/// parameters encoded in their merkle leaf.
///
/// # PDA Seeds
/// `[b"merkle_distribution", mint.as_ref(), authority.as_ref(), seeds.as_ref()]`
#[derive(Clone, Debug, PartialEq, CodamaAccount)]
#[repr(C)]
pub struct MerkleDistribution {
    pub bump: u8,
    pub revocable: u8,
    _padding: [u8; 6],
    pub authority: Address,
    pub mint: Address,
    pub seed: Address,
    pub merkle_root: [u8; 32],
    pub total_amount: u64,
    pub total_claimed: u64,
    pub clawback_ts: i64,
}

assert_no_padding!(MerkleDistribution, 1 + 1 + 6 + 32 + 32 + 32 + 32 + 8 + 8 + 8);

impl Discriminator for MerkleDistribution {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::MerkleDistribution as u8;
}

impl Versioned for MerkleDistribution {
    const VERSION: u8 = 1;
}

impl AccountSize for MerkleDistribution {
    const DATA_LEN: usize = 1 + 1 + 6 + 32 + 32 + 32 + 32 + 8 + 8 + 8; // 160
}

impl AccountParse for MerkleDistribution {
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        require_account_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);
        validate_version!(data, Self::VERSION);

        // Skip discriminator (byte 0) and version (byte 1)
        let data = &data[2..];

        let bump = data[0];
        let revocable = data[1];
        // Skip padding bytes [2..8]
        let authority =
            Address::new_from_array(data[8..40].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let mint =
            Address::new_from_array(data[40..72].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let seeds =
            Address::new_from_array(data[72..104].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let merkle_root: [u8; 32] = data[104..136].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?;
        let total_amount =
            u64::from_le_bytes(data[136..144].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let total_claimed =
            u64::from_le_bytes(data[144..152].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let clawback_ts =
            i64::from_le_bytes(data[152..160].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);

        Ok(Self {
            bump,
            revocable,
            _padding: [0u8; 6],
            authority,
            mint,
            seed: seeds,
            merkle_root,
            total_amount,
            total_claimed,
            clawback_ts,
        })
    }
}

impl AccountSerialize for MerkleDistribution {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data.push(self.revocable);
        data.extend_from_slice(&[0u8; 6]); // padding
        data.extend_from_slice(self.authority.as_ref());
        data.extend_from_slice(self.mint.as_ref());
        data.extend_from_slice(self.seed.as_ref());
        data.extend_from_slice(&self.merkle_root);
        data.extend_from_slice(&self.total_amount.to_le_bytes());
        data.extend_from_slice(&self.total_claimed.to_le_bytes());
        data.extend_from_slice(&self.clawback_ts.to_le_bytes());
        data
    }
}

impl AccountValidation for MerkleDistribution {}

impl PdaSeeds for MerkleDistribution {
    const PREFIX: &'static [u8] = b"merkle_distribution";

    fn seeds(&self) -> Vec<&[u8]> {
        vec![Self::PREFIX, self.mint.as_ref(), self.authority.as_ref(), self.seed.as_ref()]
    }

    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
        vec![
            Seed::from(Self::PREFIX),
            Seed::from(self.mint.as_ref()),
            Seed::from(self.authority.as_ref()),
            Seed::from(self.seed.as_ref()),
            Seed::from(bump.as_slice()),
        ]
    }
}

impl PdaAccount for MerkleDistribution {
    #[inline(always)]
    fn bump(&self) -> u8 {
        self.bump
    }
}

impl Distribution for MerkleDistribution {
    #[inline(always)]
    fn mint(&self) -> &Address {
        &self.mint
    }

    #[inline(always)]
    fn authority(&self) -> &Address {
        &self.authority
    }

    #[inline(always)]
    fn seeds_key(&self) -> &Address {
        &self.seed
    }

    #[inline(always)]
    fn total_claimed(&self) -> u64 {
        self.total_claimed
    }

    #[inline(always)]
    fn set_total_claimed(&mut self, amount: u64) -> Result<(), ProgramError> {
        if amount < self.total_claimed {
            return Err(RewardsProgramError::ClaimedAmountDecreased.into());
        }
        self.total_claimed = amount;
        Ok(())
    }
}

impl DistributionSigner for MerkleDistribution {
    #[inline(always)]
    fn with_signer<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Signer<'_, '_>]) -> R,
    {
        let bump_seed = [self.bump];
        let pda_seeds = [
            Seed::from(Self::PREFIX),
            Seed::from(self.mint.as_ref()),
            Seed::from(self.authority.as_ref()),
            Seed::from(self.seed.as_ref()),
            Seed::from(bump_seed.as_slice()),
        ];
        let signers = [Signer::from(&pda_seeds)];
        f(&signers)
    }
}

impl MerkleDistribution {
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub fn new(
        bump: u8,
        revocable: u8,
        authority: Address,
        mint: Address,
        seeds: Address,
        merkle_root: [u8; 32],
        total_amount: u64,
        clawback_ts: i64,
    ) -> Self {
        Self {
            bump,
            revocable,
            _padding: [0u8; 6],
            authority,
            mint,
            seed: seeds,
            merkle_root,
            total_amount,
            total_claimed: 0,
            clawback_ts,
        }
    }

    #[inline(always)]
    pub fn from_account(data: &[u8], account: &AccountView, program_id: &Address) -> Result<Self, ProgramError> {
        let state = Self::parse_from_bytes(data)?;
        state.validate_self(account, program_id)?;
        Ok(state)
    }

    #[inline(always)]
    pub fn is_closed(account: &AccountView, program_id: &Address) -> Result<bool, ProgramError> {
        if !account.owned_by(program_id) {
            return Ok(false);
        }
        let data = account.try_borrow()?;
        Ok(!data.is_empty() && data[0] == MerkleDistributionClosed::DISCRIMINATOR)
    }

    pub fn close_in_place(account: &mut AccountView, rent_recipient: &mut AccountView) -> Result<(), ProgramError> {
        {
            let mut data = account.try_borrow_mut()?;
            data[0] = MerkleDistributionClosed::DISCRIMINATOR;
            data[1] = MerkleDistributionClosed::VERSION;
        }

        account.resize(MerkleDistributionClosed::LEN)?;
        refund_excess_rent(account, rent_recipient, MerkleDistributionClosed::LEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{Distribution, PdaAccount};

    fn create_test_distribution() -> MerkleDistribution {
        MerkleDistribution::new(
            255,
            0,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([3u8; 32]),
            [4u8; 32],
            1_000_000,
            1700000000,
        )
    }

    #[test]
    fn test_merkle_distribution_new() {
        let dist = create_test_distribution();
        assert_eq!(dist.bump, 255);
        assert_eq!(dist.revocable, 0);
        assert_eq!(dist.authority, Address::new_from_array([1u8; 32]));
        assert_eq!(dist.mint, Address::new_from_array([2u8; 32]));
        assert_eq!(dist.seed, Address::new_from_array([3u8; 32]));
        assert_eq!(dist.merkle_root, [4u8; 32]);
        assert_eq!(dist.total_amount, 1_000_000);
        assert_eq!(dist.total_claimed, 0);
        assert_eq!(dist.clawback_ts, 1700000000);
    }

    #[test]
    fn test_merkle_distribution_to_bytes_inner() {
        let dist = create_test_distribution();
        let bytes = dist.to_bytes_inner();

        assert_eq!(bytes.len(), MerkleDistribution::DATA_LEN);
        assert_eq!(bytes[0], 255); // bump
    }

    #[test]
    fn test_merkle_distribution_to_bytes() {
        let dist = create_test_distribution();
        let bytes = dist.to_bytes();

        assert_eq!(bytes.len(), MerkleDistribution::LEN);
        assert_eq!(bytes[0], MerkleDistribution::DISCRIMINATOR);
        assert_eq!(bytes[1], MerkleDistribution::VERSION);
        assert_eq!(bytes[2], 255); // bump
    }

    #[test]
    fn test_roundtrip_serialization() {
        let mut dist = create_test_distribution();
        dist.total_claimed = 500_000;

        let bytes = dist.to_bytes();
        let deserialized = MerkleDistribution::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.bump, dist.bump);
        assert_eq!(deserialized.revocable, dist.revocable);
        assert_eq!(deserialized.authority, dist.authority);
        assert_eq!(deserialized.mint, dist.mint);
        assert_eq!(deserialized.seed, dist.seed);
        assert_eq!(deserialized.merkle_root, dist.merkle_root);
        assert_eq!(deserialized.total_amount, dist.total_amount);
        assert_eq!(deserialized.total_claimed, dist.total_claimed);
        assert_eq!(deserialized.clawback_ts, dist.clawback_ts);
    }

    #[test]
    fn test_roundtrip_serialization_revocable() {
        let dist = MerkleDistribution::new(
            200,
            3,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([3u8; 32]),
            [4u8; 32],
            1_000_000,
            0,
        );
        let bytes = dist.to_bytes();
        let deserialized = MerkleDistribution::parse_from_bytes(&bytes).unwrap();
        assert_eq!(deserialized.revocable, 3);
    }

    #[test]
    fn test_parse_rejects_invalid_version() {
        let dist = create_test_distribution();
        let mut bytes = dist.to_bytes();
        bytes[1] = MerkleDistribution::VERSION + 1;

        let result = MerkleDistribution::parse_from_bytes(&bytes);
        assert_eq!(result, Err(ProgramError::InvalidAccountData));
    }

    #[test]
    fn test_backward_compat_old_bytes_parse_as_non_revocable() {
        let dist = create_test_distribution();
        let bytes = dist.to_bytes();
        let deserialized = MerkleDistribution::parse_from_bytes(&bytes).unwrap();
        assert_eq!(deserialized.revocable, 0);
    }

    #[test]
    fn test_pda_seeds() {
        let dist = create_test_distribution();
        let seeds = dist.seeds();
        assert_eq!(seeds.len(), 4);
        assert_eq!(seeds[0], MerkleDistribution::PREFIX);
        assert_eq!(seeds[1], dist.mint.as_ref());
        assert_eq!(seeds[2], dist.authority.as_ref());
        assert_eq!(seeds[3], dist.seed.as_ref());
    }

    #[test]
    fn test_validate_authority_success() {
        let dist = create_test_distribution();
        let authority = Address::new_from_array([1u8; 32]);
        assert!(Distribution::validate_authority(&dist, &authority).is_ok());
    }

    #[test]
    fn test_validate_authority_fail() {
        let dist = create_test_distribution();
        let wrong_authority = Address::new_from_array([99u8; 32]);
        assert!(Distribution::validate_authority(&dist, &wrong_authority).is_err());
    }

    #[test]
    fn test_distribution_trait_accessors() {
        let dist = create_test_distribution();
        assert_eq!(Distribution::mint(&dist), &dist.mint);
        assert_eq!(Distribution::authority(&dist), &dist.authority);
        assert_eq!(Distribution::seeds_key(&dist), &dist.seed);
        assert_eq!(PdaAccount::bump(&dist), dist.bump);
        assert_eq!(Distribution::total_claimed(&dist), dist.total_claimed);
    }

    #[test]
    fn test_distribution_add_claimed() {
        let mut dist = create_test_distribution();
        Distribution::add_claimed(&mut dist, 500).unwrap();
        assert_eq!(dist.total_claimed, 500);
    }

    #[test]
    fn test_set_total_claimed_rejects_decrease() {
        let mut dist = create_test_distribution();
        Distribution::add_claimed(&mut dist, 500).unwrap();
        assert!(Distribution::set_total_claimed(&mut dist, 400).is_err());
        assert_eq!(Distribution::total_claimed(&dist), 500);
    }
}
