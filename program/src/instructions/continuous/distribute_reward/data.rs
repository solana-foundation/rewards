use pinocchio::error::ProgramError;

use crate::{errors::RewardsProgramError, require_len, traits::InstructionData};

const NEW_DECRYPTABLE_LEN: usize = 36;

pub struct DistributeContinuousRewardData {
    pub amount: u64,
    /// Required when `pool.confidential_rewards != 0`. Must be `counter + 1`
    /// where `counter` is the vault's current `pending_balance_credit_counter`
    /// read off-chain before building the transaction (one `ConfidentialDeposit`
    /// increments it in the same tx).
    pub expected_pending_balance_credit_counter: u64,
    /// Required when `pool.confidential_rewards != 0`.
    /// AES-GCM ciphertext of the vault's new available balance after applying
    /// the pending deposit — computed by the authority off-chain.
    pub new_decryptable_available_balance: Option<[u8; NEW_DECRYPTABLE_LEN]>,
}

impl<'a> TryFrom<&'a [u8]> for DistributeContinuousRewardData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let amount = u64::from_le_bytes(data[0..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        let expected_pending_balance_credit_counter =
            u64::from_le_bytes(data[8..16].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        let new_decryptable_available_balance = if data.len() >= Self::LEN + NEW_DECRYPTABLE_LEN {
            Some(
                data[Self::LEN..Self::LEN + NEW_DECRYPTABLE_LEN]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            )
        } else {
            None
        };

        Ok(Self { amount, expected_pending_balance_credit_counter, new_decryptable_available_balance })
    }
}

impl<'a> InstructionData<'a> for DistributeContinuousRewardData {
    const LEN: usize = 16;

    fn validate(&self) -> Result<(), ProgramError> {
        if self.amount == 0 {
            return Err(RewardsProgramError::InvalidAmount.into());
        }
        Ok(())
    }
}
