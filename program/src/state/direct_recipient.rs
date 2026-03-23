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
    AccountParse, AccountSerialize, AccountSize, AccountValidation, ClaimTracker, Discriminator, PdaAccount, PdaSeeds,
    RewardsAccountDiscriminators, Versioned, VestingParams, ACCOUNT_HEADER_SIZE,
};
use crate::utils::VestingSchedule;
use crate::{require_account_len, validate_discriminator, validate_version};

/// DirectRecipient account state
///
/// Represents a recipient's allocation within a direct distribution.
/// Each recipient has their own vesting schedule.
///
/// Fixed fields first, variable-length schedule last. Account size
/// depends on the schedule variant (116–140 bytes total).
///
/// # PDA Seeds
/// `[b"direct_recipient", distribution.as_ref(), recipient.as_ref()]`
#[derive(Clone, Debug, PartialEq, CodamaAccount)]
pub struct DirectRecipient {
    pub bump: u8,
    pub distribution: Address,
    pub recipient: Address,
    pub payer: Address,
    pub total_amount: u64,
    pub claimed_amount: u64,
    pub schedule: VestingSchedule,
}

/// Fixed fields size: bump(1) + distribution(32) + recipient(32) + payer(32) + total_amount(8) + claimed_amount(8)
const FIXED_DATA_LEN: usize = 1 + 32 + 32 + 32 + 8 + 8;

impl Discriminator for DirectRecipient {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::DirectRecipient as u8;
}

impl Versioned for DirectRecipient {
    const VERSION: u8 = 1;
}

impl AccountSize for DirectRecipient {
    /// Minimum DATA_LEN: fixed fields (113) + smallest schedule variant (Immediate = 1 byte) = 114
    const DATA_LEN: usize = FIXED_DATA_LEN + 1;
}

impl AccountParse for DirectRecipient {
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        require_account_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);
        validate_version!(data, Self::VERSION);

        // Skip discriminator (byte 0) and version (byte 1)
        let data = &data[2..];

        let bump = data[0];
        let distribution =
            Address::new_from_array(data[1..33].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let recipient =
            Address::new_from_array(data[33..65].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let payer =
            Address::new_from_array(data[65..97].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let total_amount =
            u64::from_le_bytes(data[97..105].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let claimed_amount =
            u64::from_le_bytes(data[105..113].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let (schedule, _) = VestingSchedule::from_bytes(&data[113..])?;

        Ok(Self { bump, distribution, recipient, payer, total_amount, claimed_amount, schedule })
    }
}

impl AccountSerialize for DirectRecipient {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(FIXED_DATA_LEN + self.schedule.byte_len());
        data.push(self.bump);
        data.extend_from_slice(self.distribution.as_ref());
        data.extend_from_slice(self.recipient.as_ref());
        data.extend_from_slice(self.payer.as_ref());
        data.extend_from_slice(&self.total_amount.to_le_bytes());
        data.extend_from_slice(&self.claimed_amount.to_le_bytes());
        data.extend_from_slice(&self.schedule.to_bytes());
        data
    }
}

impl AccountValidation for DirectRecipient {}

impl PdaSeeds for DirectRecipient {
    const PREFIX: &'static [u8] = b"direct_recipient";

    #[inline(always)]
    fn seeds(&self) -> Vec<&[u8]> {
        vec![Self::PREFIX, self.distribution.as_ref(), self.recipient.as_ref()]
    }

    #[inline(always)]
    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
        vec![
            Seed::from(Self::PREFIX),
            Seed::from(self.distribution.as_ref()),
            Seed::from(self.recipient.as_ref()),
            Seed::from(bump.as_slice()),
        ]
    }
}

impl PdaAccount for DirectRecipient {
    #[inline(always)]
    fn bump(&self) -> u8 {
        self.bump
    }
}

impl ClaimTracker for DirectRecipient {
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

impl VestingParams for DirectRecipient {
    #[inline(always)]
    fn total_amount(&self) -> u64 {
        self.total_amount
    }

    #[inline(always)]
    fn vesting_schedule(&self) -> VestingSchedule {
        self.schedule
    }
}

impl DirectRecipient {
    pub fn calculate_account_size(schedule: &VestingSchedule) -> usize {
        ACCOUNT_HEADER_SIZE + FIXED_DATA_LEN + schedule.byte_len()
    }

    #[inline(always)]
    pub fn new(
        bump: u8,
        distribution: Address,
        recipient: Address,
        payer: Address,
        total_amount: u64,
        schedule: VestingSchedule,
    ) -> Self {
        Self { bump, distribution, recipient, payer, total_amount, claimed_amount: 0, schedule }
    }

    #[inline(always)]
    pub fn from_account(data: &[u8], account: &AccountView, program_id: &Address) -> Result<Self, ProgramError> {
        let state = Self::parse_from_bytes(data)?;
        state.validate_self(account, program_id)?;
        Ok(state)
    }

    /// Execute a CPI with this recipient PDA as signer
    #[inline(always)]
    pub fn with_signer<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Signer<'_, '_>]) -> R,
    {
        let bump_seed = [self.bump];
        let seeds = [
            Seed::from(Self::PREFIX),
            Seed::from(self.distribution.as_ref()),
            Seed::from(self.recipient.as_ref()),
            Seed::from(bump_seed.as_slice()),
        ];
        let signers = [Signer::from(&seeds)];
        f(&signers)
    }

    pub fn remaining_amount(&self) -> Result<u64, RewardsProgramError> {
        self.total_amount.checked_sub(self.claimed_amount).ok_or(RewardsProgramError::MathOverflow)
    }

    #[inline(always)]
    pub fn validate_distribution(&self, distribution: &Address) -> Result<(), ProgramError> {
        if &self.distribution != distribution {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    #[inline(always)]
    pub fn validate_recipient(&self, recipient: &Address) -> Result<(), ProgramError> {
        if &self.recipient != recipient {
            return Err(RewardsProgramError::UnauthorizedRecipient.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{ClaimTracker, VestingParams};

    fn create_test_recipient() -> DirectRecipient {
        DirectRecipient::new(
            255,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([3u8; 32]),
            1000,
            VestingSchedule::Linear { start_ts: 100, end_ts: 200 },
        )
    }

    #[test]
    fn test_direct_recipient_new() {
        let recipient = create_test_recipient();
        assert_eq!(recipient.bump, 255);
        assert_eq!(recipient.distribution, Address::new_from_array([1u8; 32]));
        assert_eq!(recipient.recipient, Address::new_from_array([2u8; 32]));
        assert_eq!(recipient.payer, Address::new_from_array([3u8; 32]));
        assert_eq!(recipient.total_amount, 1000);
        assert_eq!(recipient.claimed_amount, 0);
        assert_eq!(recipient.schedule, VestingSchedule::Linear { start_ts: 100, end_ts: 200 });
    }

    #[test]
    fn test_direct_recipient_new_cliff_linear() {
        let schedule = VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 };
        let recipient = DirectRecipient::new(
            255,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([3u8; 32]),
            1000,
            schedule,
        );
        assert_eq!(recipient.schedule, schedule);
    }

    #[test]
    fn test_direct_recipient_to_bytes_inner() {
        let recipient = create_test_recipient();
        let bytes = recipient.to_bytes_inner();

        // Linear schedule = 17 bytes, so inner = 113 + 17 = 130
        assert_eq!(bytes.len(), FIXED_DATA_LEN + recipient.schedule.byte_len());
        assert_eq!(bytes[0], 255); // bump
        assert_eq!(&bytes[1..33], &[1u8; 32]); // distribution
        assert_eq!(&bytes[33..65], &[2u8; 32]); // recipient
        assert_eq!(&bytes[65..97], &[3u8; 32]); // payer
    }

    #[test]
    fn test_direct_recipient_to_bytes() {
        let recipient = create_test_recipient();
        let bytes = recipient.to_bytes();

        assert_eq!(bytes.len(), DirectRecipient::calculate_account_size(&recipient.schedule));
        assert_eq!(bytes[0], DirectRecipient::DISCRIMINATOR);
        assert_eq!(bytes[1], DirectRecipient::VERSION);
        assert_eq!(bytes[2], 255); // bump
    }

    #[test]
    fn test_direct_recipient_pda_seeds() {
        let recipient = create_test_recipient();
        let seeds = recipient.seeds();

        assert_eq!(seeds.len(), 3);
        assert_eq!(seeds[0], DirectRecipient::PREFIX);
        assert_eq!(seeds[1], recipient.distribution.as_ref());
        assert_eq!(seeds[2], recipient.recipient.as_ref());
    }

    #[test]
    fn test_claimable_amount() {
        let recipient = create_test_recipient();
        // unlocked 500, claimed 0 -> claimable 500
        assert_eq!(ClaimTracker::claimable_amount(&recipient, 500).unwrap(), 500);
    }

    #[test]
    fn test_claimable_amount_with_prior_claims() {
        let mut recipient = create_test_recipient();
        recipient.claimed_amount = 200;
        // unlocked 500, claimed 200 -> claimable 300
        assert_eq!(ClaimTracker::claimable_amount(&recipient, 500).unwrap(), 300);
    }

    #[test]
    fn test_remaining_amount() {
        let mut recipient = create_test_recipient();
        recipient.claimed_amount = 300;
        // total 1000, claimed 300 -> remaining 700
        assert_eq!(recipient.remaining_amount().unwrap(), 700);
    }

    #[test]
    fn test_calculate_unlocked() {
        let recipient = create_test_recipient();
        // start=100, end=200, at ts=150 (midpoint), should unlock 50%
        assert_eq!(VestingParams::calculate_unlocked(&recipient, 150).unwrap(), 500);
    }

    #[test]
    fn test_calculate_unlocked_before_start() {
        let recipient = create_test_recipient();
        assert_eq!(VestingParams::calculate_unlocked(&recipient, 50).unwrap(), 0);
    }

    #[test]
    fn test_calculate_unlocked_after_end() {
        let recipient = create_test_recipient();
        assert_eq!(VestingParams::calculate_unlocked(&recipient, 250).unwrap(), 1000);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let mut recipient = create_test_recipient();
        recipient.claimed_amount = 500;

        let bytes = recipient.to_bytes();
        let deserialized = DirectRecipient::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.bump, recipient.bump);
        assert_eq!(deserialized.distribution, recipient.distribution);
        assert_eq!(deserialized.recipient, recipient.recipient);
        assert_eq!(deserialized.payer, recipient.payer);
        assert_eq!(deserialized.total_amount, recipient.total_amount);
        assert_eq!(deserialized.claimed_amount, recipient.claimed_amount);
        assert_eq!(deserialized.schedule, recipient.schedule);
    }

    #[test]
    fn test_roundtrip_serialization_cliff_linear() {
        let schedule = VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 };
        let recipient = DirectRecipient::new(
            200,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([3u8; 32]),
            5000,
            schedule,
        );

        let bytes = recipient.to_bytes();
        let deserialized = DirectRecipient::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.schedule, schedule);
    }

    #[test]
    fn test_parse_rejects_invalid_version() {
        let recipient = create_test_recipient();
        let mut bytes = recipient.to_bytes();
        bytes[1] = DirectRecipient::VERSION + 1;

        let result = DirectRecipient::parse_from_bytes(&bytes);
        assert_eq!(result, Err(ProgramError::InvalidAccountData));
    }

    #[test]
    fn test_validate_distribution_success() {
        let recipient = create_test_recipient();
        let dist = Address::new_from_array([1u8; 32]);
        assert!(recipient.validate_distribution(&dist).is_ok());
    }

    #[test]
    fn test_validate_distribution_fail() {
        let recipient = create_test_recipient();
        let wrong_dist = Address::new_from_array([99u8; 32]);
        assert!(recipient.validate_distribution(&wrong_dist).is_err());
    }

    #[test]
    fn test_validate_recipient_success() {
        let recipient = create_test_recipient();
        let recip = Address::new_from_array([2u8; 32]);
        assert!(recipient.validate_recipient(&recip).is_ok());
    }

    #[test]
    fn test_validate_recipient_fail() {
        let recipient = create_test_recipient();
        let wrong_recip = Address::new_from_array([99u8; 32]);
        assert!(recipient.validate_recipient(&wrong_recip).is_err());
    }

    #[test]
    fn test_remaining_amount_full() {
        let recipient = create_test_recipient();
        assert_eq!(recipient.remaining_amount().unwrap(), 1000);
    }

    #[test]
    fn test_claimable_amount_exceeds_unlocked() {
        let mut recipient = create_test_recipient();
        recipient.claimed_amount = 600;
        // unlocked 500, claimed 600 -> error (invariant violation)
        assert!(ClaimTracker::claimable_amount(&recipient, 500).is_err());
    }

    #[test]
    fn test_remaining_amount_overflow() {
        let mut recipient = create_test_recipient();
        recipient.claimed_amount = 1500;
        // claimed > total -> error (invariant violation)
        assert!(recipient.remaining_amount().is_err());
    }

    #[test]
    fn test_claim_tracker_trait() {
        let mut recipient = create_test_recipient();
        assert_eq!(ClaimTracker::claimed_amount(&recipient), 0);
        ClaimTracker::set_claimed_amount(&mut recipient, 500).unwrap();
        assert_eq!(ClaimTracker::claimed_amount(&recipient), 500);
    }

    #[test]
    fn test_set_claimed_amount_rejects_decrease() {
        let mut recipient = create_test_recipient();
        ClaimTracker::set_claimed_amount(&mut recipient, 500).unwrap();
        assert!(ClaimTracker::set_claimed_amount(&mut recipient, 400).is_err());
        assert_eq!(ClaimTracker::claimed_amount(&recipient), 500);
    }

    #[test]
    fn test_vesting_params_trait() {
        let recipient = create_test_recipient();
        assert_eq!(VestingParams::total_amount(&recipient), 1000);
        assert_eq!(VestingParams::vesting_schedule(&recipient), VestingSchedule::Linear { start_ts: 100, end_ts: 200 });
    }

    #[test]
    fn test_vesting_params_calculate_unlocked() {
        let recipient = create_test_recipient();
        assert_eq!(VestingParams::calculate_unlocked(&recipient, 150).unwrap(), 500);
    }

    #[test]
    fn test_cliff_linear_unlocked_amount() {
        let recipient = DirectRecipient::new(
            255,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([3u8; 32]),
            1000,
            VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 },
        );
        // Before cliff
        assert_eq!(VestingParams::calculate_unlocked(&recipient, 50).unwrap(), 0);
        // At cliff: 1000 * 100/400 = 250
        assert_eq!(VestingParams::calculate_unlocked(&recipient, 100).unwrap(), 250);
        // After cliff, midpoint: 1000 * 200/400 = 500
        assert_eq!(VestingParams::calculate_unlocked(&recipient, 200).unwrap(), 500);
        // At end
        assert_eq!(VestingParams::calculate_unlocked(&recipient, 400).unwrap(), 1000);
    }
}
