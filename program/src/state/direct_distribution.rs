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
    AccountParse, AccountSerialize, AccountSize, AccountValidation, Discriminator, Distribution, DistributionSigner,
    PdaAccount, PdaSeeds, RewardsAccountDiscriminators, Versioned,
};
use crate::{assert_no_padding, require_account_len, validate_discriminator, validate_version};

/// DirectDistribution account state
///
/// Represents a direct distribution configuration that holds tokens
/// to be distributed to explicitly-added recipients. Vesting schedules
/// are per-recipient (stored in DirectRecipient accounts).
///
/// # PDA Seeds
/// `[b"direct_distribution", mint.as_ref(), authority.as_ref(), seeds.as_ref()]`
#[derive(Clone, Debug, PartialEq, CodamaAccount)]
#[repr(C)]
pub struct DirectDistribution {
    pub bump: u8,
    pub revocable: u8,
    _padding: [u8; 6],
    pub authority: Address,
    pub mint: Address,
    pub seed: Address,
    pub total_allocated: u64,
    pub total_claimed: u64,
    pub clawback_ts: i64,
}

assert_no_padding!(DirectDistribution, 1 + 1 + 6 + 32 + 32 + 32 + 8 + 8 + 8);

impl Discriminator for DirectDistribution {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::DirectDistribution as u8;
}

impl Versioned for DirectDistribution {
    const VERSION: u8 = 1;
}

impl AccountSize for DirectDistribution {
    const DATA_LEN: usize = 1 + 1 + 6 + 32 + 32 + 32 + 8 + 8 + 8; // 128
}

impl AccountParse for DirectDistribution {
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
        let total_allocated =
            u64::from_le_bytes(data[104..112].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let total_claimed =
            u64::from_le_bytes(data[112..120].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let clawback_ts =
            i64::from_le_bytes(data[120..128].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);

        Ok(Self {
            bump,
            revocable,
            _padding: [0u8; 6],
            authority,
            mint,
            seed: seeds,
            total_allocated,
            total_claimed,
            clawback_ts,
        })
    }
}

impl AccountSerialize for DirectDistribution {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data.push(self.revocable);
        data.extend_from_slice(&[0u8; 6]); // padding
        data.extend_from_slice(self.authority.as_ref());
        data.extend_from_slice(self.mint.as_ref());
        data.extend_from_slice(self.seed.as_ref());
        data.extend_from_slice(&self.total_allocated.to_le_bytes());
        data.extend_from_slice(&self.total_claimed.to_le_bytes());
        data.extend_from_slice(&self.clawback_ts.to_le_bytes());
        data
    }
}

impl AccountValidation for DirectDistribution {}

impl PdaSeeds for DirectDistribution {
    const PREFIX: &'static [u8] = b"direct_distribution";

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

impl PdaAccount for DirectDistribution {
    #[inline(always)]
    fn bump(&self) -> u8 {
        self.bump
    }
}

impl Distribution for DirectDistribution {
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

impl DistributionSigner for DirectDistribution {
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

impl DirectDistribution {
    #[inline(always)]
    pub fn new(bump: u8, revocable: u8, clawback_ts: i64, authority: Address, mint: Address, seeds: Address) -> Self {
        Self {
            bump,
            revocable,
            _padding: [0u8; 6],
            authority,
            mint,
            seed: seeds,
            total_allocated: 0,
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

    /// Returns `true` if the given account holds a `DirectDistributionClosed` marker
    /// — i.e., a distribution that was previously closed at this PDA address.
    ///
    /// An account owned by a different program (including the system program
    /// for a fresh/uninitialized PDA) is never considered closed.
    #[inline(always)]
    pub fn is_closed(account: &AccountView, program_id: &Address) -> Result<bool, ProgramError> {
        if !account.owned_by(program_id) {
            return Ok(false);
        }
        let data = account.try_borrow()?;
        Ok(!data.is_empty() && data[0] == crate::state::DirectDistributionClosed::DISCRIMINATOR)
    }

    pub fn remaining_unallocated(&self, vault_balance: u64) -> Result<u64, RewardsProgramError> {
        let outstanding =
            self.total_allocated.checked_sub(self.total_claimed).ok_or(RewardsProgramError::MathOverflow)?;
        vault_balance.checked_sub(outstanding).ok_or(RewardsProgramError::MathOverflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{Distribution, PdaAccount};

    fn create_test_distribution() -> DirectDistribution {
        DirectDistribution::new(
            255,
            0,
            0,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([3u8; 32]),
        )
    }

    #[test]
    fn test_direct_distribution_new() {
        let dist = create_test_distribution();
        assert_eq!(dist.bump, 255);
        assert_eq!(dist.revocable, 0);
        assert_eq!(dist.total_allocated, 0);
        assert_eq!(dist.total_claimed, 0);
    }

    #[test]
    fn test_direct_distribution_new_revocable() {
        let dist = DirectDistribution::new(
            255,
            1,
            0,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([3u8; 32]),
        );
        assert_eq!(dist.revocable, 1);
    }

    #[test]
    fn test_direct_distribution_to_bytes_inner() {
        let dist = create_test_distribution();
        let bytes = dist.to_bytes_inner();

        assert_eq!(bytes.len(), DirectDistribution::DATA_LEN);
        assert_eq!(bytes[0], 255); // bump
    }

    #[test]
    fn test_direct_distribution_to_bytes() {
        let dist = create_test_distribution();
        let bytes = dist.to_bytes();

        assert_eq!(bytes.len(), DirectDistribution::LEN);
        assert_eq!(bytes[0], DirectDistribution::DISCRIMINATOR);
        assert_eq!(bytes[1], DirectDistribution::VERSION);
        assert_eq!(bytes[2], 255); // bump
    }

    #[test]
    fn test_remaining_unallocated() {
        let mut dist = create_test_distribution();
        dist.total_allocated = 500;
        dist.total_claimed = 100;

        // distribution vault has 1000, allocated 500, claimed 100
        // remaining = 1000 - (500 - 100) = 600
        assert_eq!(dist.remaining_unallocated(1000).unwrap(), 600);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let dist = create_test_distribution();
        let bytes = dist.to_bytes();
        let deserialized = DirectDistribution::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.bump, dist.bump);
        assert_eq!(deserialized.revocable, dist.revocable);
        assert_eq!(deserialized.authority, dist.authority);
        assert_eq!(deserialized.mint, dist.mint);
        assert_eq!(deserialized.seed, dist.seed);
        assert_eq!(deserialized.total_allocated, dist.total_allocated);
        assert_eq!(deserialized.total_claimed, dist.total_claimed);
        assert_eq!(deserialized.clawback_ts, dist.clawback_ts);
    }

    #[test]
    fn test_roundtrip_serialization_revocable() {
        let dist = DirectDistribution::new(
            200,
            1,
            0,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([3u8; 32]),
        );
        let bytes = dist.to_bytes();
        let deserialized = DirectDistribution::parse_from_bytes(&bytes).unwrap();
        assert_eq!(deserialized.revocable, 1);
    }

    #[test]
    fn test_parse_rejects_invalid_version() {
        let dist = create_test_distribution();
        let mut bytes = dist.to_bytes();
        bytes[1] = DirectDistribution::VERSION + 1;

        let result = DirectDistribution::parse_from_bytes(&bytes);
        assert_eq!(result, Err(ProgramError::InvalidAccountData));
    }

    #[test]
    fn test_backward_compat_old_bytes_parse_as_non_revocable() {
        let dist = create_test_distribution();
        let bytes = dist.to_bytes();
        // Old accounts have 0x00 at the revocable offset (was padding)
        let deserialized = DirectDistribution::parse_from_bytes(&bytes).unwrap();
        assert_eq!(deserialized.revocable, 0);
    }

    #[test]
    fn test_roundtrip_serialization_with_clawback_ts() {
        let dist = DirectDistribution::new(
            200,
            0,
            1700000000,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([3u8; 32]),
        );
        let bytes = dist.to_bytes();
        let deserialized = DirectDistribution::parse_from_bytes(&bytes).unwrap();
        assert_eq!(deserialized.clawback_ts, 1700000000);
    }

    #[test]
    fn test_pda_seeds() {
        let dist = create_test_distribution();
        let seeds = dist.seeds();
        assert_eq!(seeds.len(), 4);
        assert_eq!(seeds[0], DirectDistribution::PREFIX);
        assert_eq!(seeds[1], dist.mint.as_ref());
        assert_eq!(seeds[2], dist.authority.as_ref());
        assert_eq!(seeds[3], dist.seed.as_ref());
    }

    #[test]
    fn test_remaining_unallocated_zero_allocated() {
        let dist = create_test_distribution();
        assert_eq!(dist.remaining_unallocated(1000).unwrap(), 1000);
    }

    #[test]
    fn test_remaining_unallocated_insufficient_vault() {
        let mut dist = create_test_distribution();
        dist.total_allocated = 1000;
        dist.total_claimed = 0;
        // distribution vault has less than outstanding -> error (invariant violation)
        assert!(dist.remaining_unallocated(500).is_err());
    }

    #[test]
    fn test_remaining_unallocated_claimed_exceeds_allocated() {
        let mut dist = create_test_distribution();
        dist.total_allocated = 500;
        dist.total_claimed = 600;
        // claimed > allocated -> error (invariant violation)
        assert!(dist.remaining_unallocated(1000).is_err());
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
