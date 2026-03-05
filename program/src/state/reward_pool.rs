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
    AccountParse, AccountSerialize, AccountSize, AccountValidation, Discriminator, PdaAccount, PdaSeeds,
    RewardsAccountDiscriminators, Versioned,
};
use crate::utils::BalanceSource;
use crate::{require_account_len, validate_discriminator};

/// RewardPool account state
///
/// Represents a continuous reward pool where users earn rewards proportional
/// to their held balance of a tracked token over time. Uses a reward-per-token
/// accumulator pattern for gas-efficient distribution.
///
/// # PDA Seeds
/// `[b"reward_pool", reward_mint.as_ref(), tracked_mint.as_ref(), authority.as_ref(), seed.as_ref()]`
#[derive(Clone, Debug, PartialEq, CodamaAccount)]
pub struct RewardPool {
    pub bump: u8,
    pub balance_source: BalanceSource,
    pub revocable: u8,
    _padding: [u8; 5],
    pub authority: Address,
    pub tracked_mint: Address,
    pub reward_mint: Address,
    pub seed: Address,
    pub reward_per_token: u128,
    pub opted_in_supply: u64,
    pub total_distributed: u64,
    pub total_claimed: u64,
    pub clawback_ts: i64,
}

impl Discriminator for RewardPool {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::RewardPool as u8;
}

impl Versioned for RewardPool {
    const VERSION: u8 = 1;
}

impl AccountSize for RewardPool {
    const DATA_LEN: usize = 1 + 1 + 1 + 5 + 32 + 32 + 32 + 32 + 16 + 8 + 8 + 8 + 8; // 184
}

impl AccountParse for RewardPool {
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        require_account_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);

        let data = &data[2..];

        let bump = data[0];
        let balance_source = BalanceSource::try_from(data[1])?;
        let revocable = data[2];
        let authority =
            Address::new_from_array(data[8..40].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let tracked_mint =
            Address::new_from_array(data[40..72].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let reward_mint =
            Address::new_from_array(data[72..104].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let seed =
            Address::new_from_array(data[104..136].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let reward_per_token =
            u128::from_le_bytes(data[136..152].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let opted_in_supply =
            u64::from_le_bytes(data[152..160].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let total_distributed =
            u64::from_le_bytes(data[160..168].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let total_claimed =
            u64::from_le_bytes(data[168..176].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let clawback_ts =
            i64::from_le_bytes(data[176..184].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);

        Ok(Self {
            bump,
            balance_source,
            revocable,
            _padding: [0u8; 5],
            authority,
            tracked_mint,
            reward_mint,
            seed,
            reward_per_token,
            opted_in_supply,
            total_distributed,
            total_claimed,
            clawback_ts,
        })
    }
}

impl AccountSerialize for RewardPool {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data.push(self.balance_source.to_byte());
        data.push(self.revocable);
        data.extend_from_slice(&[0u8; 5]);
        data.extend_from_slice(self.authority.as_ref());
        data.extend_from_slice(self.tracked_mint.as_ref());
        data.extend_from_slice(self.reward_mint.as_ref());
        data.extend_from_slice(self.seed.as_ref());
        data.extend_from_slice(&self.reward_per_token.to_le_bytes());
        data.extend_from_slice(&self.opted_in_supply.to_le_bytes());
        data.extend_from_slice(&self.total_distributed.to_le_bytes());
        data.extend_from_slice(&self.total_claimed.to_le_bytes());
        data.extend_from_slice(&self.clawback_ts.to_le_bytes());
        data
    }
}

impl AccountValidation for RewardPool {}

impl PdaSeeds for RewardPool {
    const PREFIX: &'static [u8] = b"reward_pool";

    fn seeds(&self) -> Vec<&[u8]> {
        vec![
            Self::PREFIX,
            self.reward_mint.as_ref(),
            self.tracked_mint.as_ref(),
            self.authority.as_ref(),
            self.seed.as_ref(),
        ]
    }

    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
        vec![
            Seed::from(Self::PREFIX),
            Seed::from(self.reward_mint.as_ref()),
            Seed::from(self.tracked_mint.as_ref()),
            Seed::from(self.authority.as_ref()),
            Seed::from(self.seed.as_ref()),
            Seed::from(bump.as_slice()),
        ]
    }
}

impl PdaAccount for RewardPool {
    #[inline(always)]
    fn bump(&self) -> u8 {
        self.bump
    }
}

impl RewardPool {
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bump: u8,
        balance_source: BalanceSource,
        revocable: u8,
        clawback_ts: i64,
        authority: Address,
        tracked_mint: Address,
        reward_mint: Address,
        seed: Address,
    ) -> Self {
        Self {
            bump,
            balance_source,
            revocable,
            _padding: [0u8; 5],
            authority,
            tracked_mint,
            reward_mint,
            seed,
            reward_per_token: 0,
            opted_in_supply: 0,
            total_distributed: 0,
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
    pub fn validate_authority(&self, authority: &Address) -> Result<(), ProgramError> {
        if &self.authority != authority {
            return Err(RewardsProgramError::UnauthorizedAuthority.into());
        }
        Ok(())
    }

    #[inline(always)]
    pub fn validate_tracked_mint(&self, mint: &Address) -> Result<(), ProgramError> {
        if &self.tracked_mint != mint {
            return Err(RewardsProgramError::TrackedMintMismatch.into());
        }
        Ok(())
    }

    #[inline(always)]
    pub fn validate_reward_mint(&self, mint: &Address) -> Result<(), ProgramError> {
        if &self.reward_mint != mint {
            return Err(RewardsProgramError::RewardMintMismatch.into());
        }
        Ok(())
    }

    pub fn with_signer<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Signer<'_, '_>]) -> R,
    {
        let bump_seed = [self.bump];
        let pda_seeds = [
            Seed::from(Self::PREFIX),
            Seed::from(self.reward_mint.as_ref()),
            Seed::from(self.tracked_mint.as_ref()),
            Seed::from(self.authority.as_ref()),
            Seed::from(self.seed.as_ref()),
            Seed::from(bump_seed.as_slice()),
        ];
        let signers = [Signer::from(&pda_seeds)];
        f(&signers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::PdaAccount;

    fn create_test_pool() -> RewardPool {
        RewardPool::new(
            255,
            BalanceSource::OnChain,
            0,
            0,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([3u8; 32]),
            Address::new_from_array([4u8; 32]),
        )
    }

    #[test]
    fn test_reward_pool_new() {
        let pool = create_test_pool();
        assert_eq!(pool.bump, 255);
        assert_eq!(pool.balance_source, BalanceSource::OnChain);
        assert_eq!(pool.revocable, 0);
        assert_eq!(pool.reward_per_token, 0);
        assert_eq!(pool.opted_in_supply, 0);
        assert_eq!(pool.total_distributed, 0);
        assert_eq!(pool.total_claimed, 0);
        assert_eq!(pool.clawback_ts, 0);
    }

    #[test]
    fn test_reward_pool_to_bytes_inner() {
        let pool = create_test_pool();
        let bytes = pool.to_bytes_inner();
        assert_eq!(bytes.len(), RewardPool::DATA_LEN);
        assert_eq!(bytes[0], 255);
        assert_eq!(bytes[1], BalanceSource::OnChain.to_byte());
    }

    #[test]
    fn test_reward_pool_to_bytes() {
        let pool = create_test_pool();
        let bytes = pool.to_bytes();
        assert_eq!(bytes.len(), RewardPool::LEN);
        assert_eq!(bytes[0], RewardPool::DISCRIMINATOR);
        assert_eq!(bytes[1], RewardPool::VERSION);
        assert_eq!(bytes[2], 255);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let pool = create_test_pool();
        let bytes = pool.to_bytes();
        let deserialized = RewardPool::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.bump, pool.bump);
        assert_eq!(deserialized.balance_source, pool.balance_source);
        assert_eq!(deserialized.revocable, pool.revocable);
        assert_eq!(deserialized.authority, pool.authority);
        assert_eq!(deserialized.tracked_mint, pool.tracked_mint);
        assert_eq!(deserialized.reward_mint, pool.reward_mint);
        assert_eq!(deserialized.seed, pool.seed);
        assert_eq!(deserialized.reward_per_token, pool.reward_per_token);
        assert_eq!(deserialized.opted_in_supply, pool.opted_in_supply);
        assert_eq!(deserialized.total_distributed, pool.total_distributed);
        assert_eq!(deserialized.total_claimed, pool.total_claimed);
        assert_eq!(deserialized.clawback_ts, pool.clawback_ts);
    }

    #[test]
    fn test_roundtrip_with_values() {
        let mut pool = create_test_pool();
        pool.reward_per_token = 1_000_000_000_000;
        pool.opted_in_supply = 500_000;
        pool.total_distributed = 1_000_000;
        pool.total_claimed = 250_000;
        pool.clawback_ts = 1700000000;

        let bytes = pool.to_bytes();
        let deserialized = RewardPool::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.reward_per_token, 1_000_000_000_000);
        assert_eq!(deserialized.opted_in_supply, 500_000);
        assert_eq!(deserialized.total_distributed, 1_000_000);
        assert_eq!(deserialized.total_claimed, 250_000);
        assert_eq!(deserialized.clawback_ts, 1700000000);
    }

    #[test]
    fn test_pda_seeds() {
        let pool = create_test_pool();
        let seeds = pool.seeds();
        assert_eq!(seeds.len(), 5);
        assert_eq!(seeds[0], RewardPool::PREFIX);
        assert_eq!(seeds[1], pool.reward_mint.as_ref());
        assert_eq!(seeds[2], pool.tracked_mint.as_ref());
        assert_eq!(seeds[3], pool.authority.as_ref());
        assert_eq!(seeds[4], pool.seed.as_ref());
    }

    #[test]
    fn test_validate_authority_success() {
        let pool = create_test_pool();
        let authority = Address::new_from_array([1u8; 32]);
        assert!(pool.validate_authority(&authority).is_ok());
    }

    #[test]
    fn test_validate_authority_fail() {
        let pool = create_test_pool();
        let wrong_authority = Address::new_from_array([99u8; 32]);
        assert!(pool.validate_authority(&wrong_authority).is_err());
    }

    #[test]
    fn test_validate_tracked_mint_success() {
        let pool = create_test_pool();
        let mint = Address::new_from_array([2u8; 32]);
        assert!(pool.validate_tracked_mint(&mint).is_ok());
    }

    #[test]
    fn test_validate_tracked_mint_fail() {
        let pool = create_test_pool();
        let wrong_mint = Address::new_from_array([99u8; 32]);
        assert!(pool.validate_tracked_mint(&wrong_mint).is_err());
    }

    #[test]
    fn test_validate_reward_mint_success() {
        let pool = create_test_pool();
        let mint = Address::new_from_array([3u8; 32]);
        assert!(pool.validate_reward_mint(&mint).is_ok());
    }

    #[test]
    fn test_validate_reward_mint_fail() {
        let pool = create_test_pool();
        let wrong_mint = Address::new_from_array([99u8; 32]);
        assert!(pool.validate_reward_mint(&wrong_mint).is_err());
    }

    #[test]
    fn test_bump() {
        let pool = create_test_pool();
        assert_eq!(PdaAccount::bump(&pool), 255);
    }

    #[test]
    fn test_authority_set_balance_source() {
        let pool = RewardPool::new(
            200,
            BalanceSource::AuthoritySet,
            0,
            0,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([3u8; 32]),
            Address::new_from_array([4u8; 32]),
        );
        let bytes = pool.to_bytes();
        let deserialized = RewardPool::parse_from_bytes(&bytes).unwrap();
        assert_eq!(deserialized.balance_source, BalanceSource::AuthoritySet);
    }
}
