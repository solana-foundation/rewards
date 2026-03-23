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

/// UserPointsAccount state
///
/// Tracks a single user's points balance within a PointsConfig.
/// Points are a simple u64 counter — no token programs involved.
///
/// # PDA Seeds
/// `[b"user_points", points_config.as_ref(), user.as_ref()]`
#[derive(Clone, Debug, PartialEq, CodamaAccount)]
pub struct UserPointsAccount {
    pub bump: u8,
    _padding: [u8; 7],
    pub balance: u64,
}

impl Discriminator for UserPointsAccount {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::UserPointsAccount as u8;
}

impl Versioned for UserPointsAccount {
    const VERSION: u8 = 1;
}

impl AccountSize for UserPointsAccount {
    const DATA_LEN: usize = 1 + 7 + 8; // 16
}

impl AccountParse for UserPointsAccount {
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        require_account_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);

        let data = &data[2..];

        let bump = data[0];
        let balance = u64::from_le_bytes(data[8..16].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);

        Ok(Self { bump, _padding: [0u8; 7], balance })
    }
}

impl AccountSerialize for UserPointsAccount {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data.extend_from_slice(&[0u8; 7]);
        data.extend_from_slice(&self.balance.to_le_bytes());
        data
    }
}

impl AccountValidation for UserPointsAccount {}

/// Seed helper for deriving UserPointsAccount PDA without having the full state
pub struct UserPointsAccountSeeds {
    pub points_config: Address,
    pub user: Address,
}

impl PdaSeeds for UserPointsAccountSeeds {
    const PREFIX: &'static [u8] = b"user_points";

    #[inline(always)]
    fn seeds(&self) -> Vec<&[u8]> {
        vec![Self::PREFIX, self.points_config.as_ref(), self.user.as_ref()]
    }

    #[inline(always)]
    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
        vec![
            Seed::from(Self::PREFIX),
            Seed::from(self.points_config.as_ref()),
            Seed::from(self.user.as_ref()),
            Seed::from(bump.as_slice()),
        ]
    }
}

impl UserPointsAccount {
    #[inline(always)]
    pub fn new(bump: u8) -> Self {
        Self { bump, _padding: [0u8; 7], balance: 0 }
    }

    #[inline(always)]
    pub fn from_account(
        data: &[u8],
        account: &AccountView,
        program_id: &Address,
        points_config: &Address,
        user: &Address,
    ) -> Result<Self, ProgramError> {
        let state = Self::parse_from_bytes(data)?;
        let seeds = UserPointsAccountSeeds { points_config: *points_config, user: *user };
        seeds.validate_pda(account, program_id, state.bump)?;
        Ok(state)
    }

    #[inline(always)]
    pub fn validate_balance(&self, amount: u64) -> Result<(), ProgramError> {
        if self.balance < amount {
            return Err(RewardsProgramError::InsufficientPointsBalance.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_account() -> UserPointsAccount {
        UserPointsAccount::new(255)
    }

    #[test]
    fn test_user_points_account_new() {
        let account = create_test_account();
        assert_eq!(account.bump, 255);
        assert_eq!(account.balance, 0);
    }

    #[test]
    fn test_to_bytes_inner() {
        let account = create_test_account();
        let bytes = account.to_bytes_inner();
        assert_eq!(bytes.len(), UserPointsAccount::DATA_LEN);
        assert_eq!(bytes[0], 255);
    }

    #[test]
    fn test_to_bytes() {
        let account = create_test_account();
        let bytes = account.to_bytes();
        assert_eq!(bytes.len(), UserPointsAccount::LEN);
        assert_eq!(bytes[0], UserPointsAccount::DISCRIMINATOR);
        assert_eq!(bytes[1], UserPointsAccount::VERSION);
        assert_eq!(bytes[2], 255);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let account = create_test_account();
        let bytes = account.to_bytes();
        let deserialized = UserPointsAccount::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.bump, account.bump);
        assert_eq!(deserialized.balance, account.balance);
    }

    #[test]
    fn test_roundtrip_with_values() {
        let mut account = create_test_account();
        account.balance = 500_000;

        let bytes = account.to_bytes();
        let deserialized = UserPointsAccount::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.balance, 500_000);
    }

    #[test]
    fn test_pda_seeds() {
        let seeds = UserPointsAccountSeeds {
            points_config: Address::new_from_array([1u8; 32]),
            user: Address::new_from_array([2u8; 32]),
        };
        let pda_seeds = seeds.seeds();
        assert_eq!(pda_seeds.len(), 3);
        assert_eq!(pda_seeds[0], UserPointsAccountSeeds::PREFIX);
        assert_eq!(pda_seeds[1], seeds.points_config.as_ref());
        assert_eq!(pda_seeds[2], seeds.user.as_ref());
    }

    #[test]
    fn test_validate_balance_sufficient() {
        let mut account = create_test_account();
        account.balance = 100;
        assert!(account.validate_balance(100).is_ok());
    }

    #[test]
    fn test_validate_balance_insufficient() {
        let mut account = create_test_account();
        account.balance = 50;
        assert!(account.validate_balance(100).is_err());
    }
}
