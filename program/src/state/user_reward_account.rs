use alloc::vec;
use alloc::vec::Vec;
use codama::CodamaAccount;
use pinocchio::{account::AccountView, cpi::Seed, error::ProgramError, Address};

use crate::errors::RewardsProgramError;
use crate::traits::{
    AccountParse, AccountSerialize, AccountSize, AccountValidation, Discriminator, PdaSeeds,
    RewardsAccountDiscriminators, Versioned,
};
use crate::{require_account_len, validate_discriminator};

/// UserRewardAccount state
///
/// Tracks a single user's participation in a continuous reward pool.
/// Stores the user's snapshot of the global reward-per-token value
/// and their accumulated unclaimed rewards.
///
/// # PDA Seeds
/// `[b"user_reward", reward_pool.as_ref(), user.as_ref()]`
#[derive(Clone, Debug, PartialEq, CodamaAccount)]
pub struct UserRewardAccount {
    pub bump: u8,
    _padding: [u8; 7],
    pub reward_per_token_paid: u128,
    pub accrued_rewards: u64,
    pub last_known_balance: u64,
}

impl Discriminator for UserRewardAccount {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::UserRewardAccount as u8;
}

impl Versioned for UserRewardAccount {
    const VERSION: u8 = 1;
}

impl AccountSize for UserRewardAccount {
    const DATA_LEN: usize = 1 + 7 + 16 + 8 + 8; // 40
}

impl AccountParse for UserRewardAccount {
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        require_account_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);

        let data = &data[2..];

        let bump = data[0];
        let reward_per_token_paid =
            u128::from_le_bytes(data[8..24].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let accrued_rewards =
            u64::from_le_bytes(data[24..32].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let last_known_balance =
            u64::from_le_bytes(data[32..40].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);

        Ok(Self { bump, _padding: [0u8; 7], reward_per_token_paid, accrued_rewards, last_known_balance })
    }
}

impl AccountSerialize for UserRewardAccount {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data.extend_from_slice(&[0u8; 7]);
        data.extend_from_slice(&self.reward_per_token_paid.to_le_bytes());
        data.extend_from_slice(&self.accrued_rewards.to_le_bytes());
        data.extend_from_slice(&self.last_known_balance.to_le_bytes());
        data
    }
}

impl AccountValidation for UserRewardAccount {}

/// Seed helper for deriving UserRewardAccount PDA without having the full state
pub struct UserRewardAccountSeeds {
    pub reward_pool: Address,
    pub user: Address,
}

impl PdaSeeds for UserRewardAccountSeeds {
    const PREFIX: &'static [u8] = b"user_reward";

    #[inline(always)]
    fn seeds(&self) -> Vec<&[u8]> {
        vec![Self::PREFIX, self.reward_pool.as_ref(), self.user.as_ref()]
    }

    #[inline(always)]
    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
        vec![
            Seed::from(Self::PREFIX),
            Seed::from(self.reward_pool.as_ref()),
            Seed::from(self.user.as_ref()),
            Seed::from(bump.as_slice()),
        ]
    }
}

impl UserRewardAccount {
    #[inline(always)]
    pub fn new(bump: u8, reward_per_token_paid: u128, last_known_balance: u64) -> Self {
        Self { bump, _padding: [0u8; 7], reward_per_token_paid, accrued_rewards: 0, last_known_balance }
    }

    #[inline(always)]
    pub fn from_account(
        data: &[u8],
        account: &AccountView,
        program_id: &Address,
        reward_pool: &Address,
        user: &Address,
    ) -> Result<Self, ProgramError> {
        let state = Self::parse_from_bytes(data)?;
        let seeds = UserRewardAccountSeeds { reward_pool: *reward_pool, user: *user };
        seeds.validate_pda(account, program_id, state.bump)?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_account() -> UserRewardAccount {
        UserRewardAccount::new(255, 0, 1000)
    }

    #[test]
    fn test_user_reward_account_new() {
        let account = create_test_account();
        assert_eq!(account.bump, 255);
        assert_eq!(account.reward_per_token_paid, 0);
        assert_eq!(account.accrued_rewards, 0);
        assert_eq!(account.last_known_balance, 1000);
    }

    #[test]
    fn test_to_bytes_inner() {
        let account = create_test_account();
        let bytes = account.to_bytes_inner();
        assert_eq!(bytes.len(), UserRewardAccount::DATA_LEN);
        assert_eq!(bytes[0], 255);
    }

    #[test]
    fn test_to_bytes() {
        let account = create_test_account();
        let bytes = account.to_bytes();
        assert_eq!(bytes.len(), UserRewardAccount::LEN);
        assert_eq!(bytes[0], UserRewardAccount::DISCRIMINATOR);
        assert_eq!(bytes[1], UserRewardAccount::VERSION);
        assert_eq!(bytes[2], 255);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let account = create_test_account();
        let bytes = account.to_bytes();
        let deserialized = UserRewardAccount::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.bump, account.bump);
        assert_eq!(deserialized.reward_per_token_paid, account.reward_per_token_paid);
        assert_eq!(deserialized.accrued_rewards, account.accrued_rewards);
        assert_eq!(deserialized.last_known_balance, account.last_known_balance);
    }

    #[test]
    fn test_roundtrip_with_values() {
        let mut account = create_test_account();
        account.reward_per_token_paid = 1_000_000_000_000;
        account.accrued_rewards = 500;
        account.last_known_balance = 2000;

        let bytes = account.to_bytes();
        let deserialized = UserRewardAccount::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.reward_per_token_paid, 1_000_000_000_000);
        assert_eq!(deserialized.accrued_rewards, 500);
        assert_eq!(deserialized.last_known_balance, 2000);
    }

    #[test]
    fn test_pda_seeds() {
        let seeds = UserRewardAccountSeeds {
            reward_pool: Address::new_from_array([1u8; 32]),
            user: Address::new_from_array([2u8; 32]),
        };
        let pda_seeds = seeds.seeds();
        assert_eq!(pda_seeds.len(), 3);
        assert_eq!(pda_seeds[0], UserRewardAccountSeeds::PREFIX);
        assert_eq!(pda_seeds[1], seeds.reward_pool.as_ref());
        assert_eq!(pda_seeds[2], seeds.user.as_ref());
    }

    #[test]
    fn test_bump() {
        let account = create_test_account();
        assert_eq!(account.bump, 255);
    }
}
