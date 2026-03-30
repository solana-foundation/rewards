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
use crate::{require_account_len, validate_discriminator};

/// Space required for the Token-2022 points mint account.
///
/// 82 (Mint base data) + 83 (zero-padding to BASE_ACCOUNT_LENGTH) = 165
/// + 1 (AccountType discriminator)
/// + 4 (NonTransferable TLV: 2 type + 2 length + 0 data)
/// + 36 (PermanentDelegate TLV: 2 type + 2 length + 32 pubkey)
/// + 36 (MintCloseAuthority TLV: 2 type + 2 length + 32 pubkey)
/// = 242
pub const POINTS_MINT_SPACE: usize = 242;

/// PointsConfig account state
///
/// Represents a points system configuration where an authority can issue,
/// use, and transfer points. Points are backed by a Token-2022 mint with
/// NonTransferable + PermanentDelegate + MintCloseAuthority extensions.
/// The PointsConfig PDA serves as mint authority, permanent delegate, and
/// close authority for the associated points mint.
///
/// # PDA Seeds
/// `[b"points_config", authority.as_ref(), seed.as_ref()]`
#[derive(Clone, Debug, PartialEq, CodamaAccount)]
pub struct PointsConfig {
    pub bump: u8,
    pub transferable: u8,
    pub revocable: u8,
    pub mint_bump: u8,
    _padding: [u8; 4],
    pub authority: Address,
    pub seed: Address,
}

impl Discriminator for PointsConfig {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::PointsConfig as u8;
}

impl Versioned for PointsConfig {
    const VERSION: u8 = 1;
}

impl AccountSize for PointsConfig {
    const DATA_LEN: usize = 1 + 1 + 1 + 1 + 4 + 32 + 32; // 72
}

impl AccountParse for PointsConfig {
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        require_account_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);

        let data = &data[2..];

        let bump = data[0];
        let transferable = data[1];
        let revocable = data[2];
        let mint_bump = data[3];
        let authority =
            Address::new_from_array(data[8..40].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let seed =
            Address::new_from_array(data[40..72].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);

        Ok(Self { bump, transferable, revocable, mint_bump, _padding: [0u8; 4], authority, seed })
    }
}

impl AccountSerialize for PointsConfig {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data.push(self.transferable);
        data.push(self.revocable);
        data.push(self.mint_bump);
        data.extend_from_slice(&[0u8; 4]);
        data.extend_from_slice(self.authority.as_ref());
        data.extend_from_slice(self.seed.as_ref());
        data
    }
}

impl AccountValidation for PointsConfig {}

impl PdaSeeds for PointsConfig {
    const PREFIX: &'static [u8] = b"points_config";

    fn seeds(&self) -> Vec<&[u8]> {
        vec![Self::PREFIX, self.authority.as_ref(), self.seed.as_ref()]
    }

    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
        vec![
            Seed::from(Self::PREFIX),
            Seed::from(self.authority.as_ref()),
            Seed::from(self.seed.as_ref()),
            Seed::from(bump.as_slice()),
        ]
    }
}

impl PdaAccount for PointsConfig {
    #[inline(always)]
    fn bump(&self) -> u8 {
        self.bump
    }
}

/// Seed helper for deriving the points mint PDA from a config address.
///
/// # PDA Seeds
/// `[b"mint", points_config.as_ref()]`
pub struct PointsMintSeeds {
    pub points_config: Address,
}

impl PdaSeeds for PointsMintSeeds {
    const PREFIX: &'static [u8] = b"mint";

    #[inline(always)]
    fn seeds(&self) -> Vec<&[u8]> {
        vec![Self::PREFIX, self.points_config.as_ref()]
    }

    #[inline(always)]
    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
        vec![Seed::from(Self::PREFIX), Seed::from(self.points_config.as_ref()), Seed::from(bump.as_slice())]
    }
}

impl PointsConfig {
    #[inline(always)]
    pub fn new(bump: u8, transferable: u8, revocable: u8, mint_bump: u8, authority: Address, seed: Address) -> Self {
        Self { bump, transferable, revocable, mint_bump, _padding: [0u8; 4], authority, seed }
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
    pub fn validate_transferable(&self) -> Result<(), ProgramError> {
        if self.transferable == 0 {
            return Err(RewardsProgramError::PointsTransfersDisabled.into());
        }
        Ok(())
    }

    #[inline(always)]
    pub fn validate_revocable(&self) -> Result<(), ProgramError> {
        if self.revocable == 0 {
            return Err(RewardsProgramError::PointsNotRevocable.into());
        }
        Ok(())
    }

    /// Signs a CPI as the PointsConfig PDA. The config PDA serves as
    /// mint authority, permanent delegate, and close authority for the
    /// associated Token-2022 points mint.
    pub fn with_signer<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Signer<'_, '_>]) -> R,
    {
        let bump_seed = [self.bump];
        let pda_seeds = [
            Seed::from(Self::PREFIX),
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

    fn create_test_config() -> PointsConfig {
        PointsConfig::new(255, 1, 0, 254, Address::new_from_array([1u8; 32]), Address::new_from_array([2u8; 32]))
    }

    #[test]
    fn test_points_config_new() {
        let config = create_test_config();
        assert_eq!(config.bump, 255);
        assert_eq!(config.transferable, 1);
        assert_eq!(config.revocable, 0);
        assert_eq!(config.mint_bump, 254);
    }

    #[test]
    fn test_to_bytes_inner() {
        let config = create_test_config();
        let bytes = config.to_bytes_inner();
        assert_eq!(bytes.len(), PointsConfig::DATA_LEN);
        assert_eq!(bytes[0], 255); // bump
        assert_eq!(bytes[1], 1); // transferable
        assert_eq!(bytes[2], 0); // revocable
        assert_eq!(bytes[3], 254); // mint_bump
    }

    #[test]
    fn test_to_bytes() {
        let config = create_test_config();
        let bytes = config.to_bytes();
        assert_eq!(bytes.len(), PointsConfig::LEN);
        assert_eq!(bytes[0], PointsConfig::DISCRIMINATOR);
        assert_eq!(bytes[1], PointsConfig::VERSION);
        assert_eq!(bytes[2], 255);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let config = create_test_config();
        let bytes = config.to_bytes();
        let deserialized = PointsConfig::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.bump, config.bump);
        assert_eq!(deserialized.transferable, config.transferable);
        assert_eq!(deserialized.revocable, config.revocable);
        assert_eq!(deserialized.mint_bump, config.mint_bump);
        assert_eq!(deserialized.authority, config.authority);
        assert_eq!(deserialized.seed, config.seed);
    }

    #[test]
    fn test_pda_seeds() {
        let config = create_test_config();
        let seeds = config.seeds();
        assert_eq!(seeds.len(), 3);
        assert_eq!(seeds[0], PointsConfig::PREFIX);
        assert_eq!(seeds[1], config.authority.as_ref());
        assert_eq!(seeds[2], config.seed.as_ref());
    }

    #[test]
    fn test_points_mint_seeds() {
        let config_addr = Address::new_from_array([1u8; 32]);
        let seeds = PointsMintSeeds { points_config: config_addr };
        let pda_seeds = seeds.seeds();
        assert_eq!(pda_seeds.len(), 2);
        assert_eq!(pda_seeds[0], PointsMintSeeds::PREFIX);
        assert_eq!(pda_seeds[1], config_addr.as_ref());
    }

    #[test]
    fn test_validate_authority_success() {
        let config = create_test_config();
        let authority = Address::new_from_array([1u8; 32]);
        assert!(config.validate_authority(&authority).is_ok());
    }

    #[test]
    fn test_validate_authority_fail() {
        let config = create_test_config();
        let wrong = Address::new_from_array([99u8; 32]);
        assert!(config.validate_authority(&wrong).is_err());
    }

    #[test]
    fn test_validate_transferable_enabled() {
        let config = create_test_config();
        assert!(config.validate_transferable().is_ok());
    }

    #[test]
    fn test_validate_transferable_disabled() {
        let config = PointsConfig::new(
            255,
            0, // not transferable
            0,
            254,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
        );
        assert!(config.validate_transferable().is_err());
    }

    #[test]
    fn test_validate_revocable_enabled() {
        let config =
            PointsConfig::new(255, 1, 1, 254, Address::new_from_array([1u8; 32]), Address::new_from_array([2u8; 32]));
        assert!(config.validate_revocable().is_ok());
    }

    #[test]
    fn test_validate_revocable_disabled() {
        let config = create_test_config(); // revocable = 0
        assert!(config.validate_revocable().is_err());
    }

    #[test]
    fn test_bump() {
        let config = create_test_config();
        assert_eq!(PdaAccount::bump(&config), 255);
    }

    #[test]
    fn test_data_len() {
        assert_eq!(PointsConfig::DATA_LEN, 72);
        assert_eq!(PointsConfig::LEN, 74);
    }
}
