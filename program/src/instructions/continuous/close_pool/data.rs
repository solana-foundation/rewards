use pinocchio::error::ProgramError;

use crate::traits::InstructionData;

const NEW_DECRYPTABLE_LEN: usize = 36;

pub struct CloseContinuousPoolData {
    /// Required when `pool.confidential_rewards != 0` and there are unclaimed rewards.
    /// AES-GCM ciphertext of 0 (the vault's remaining available balance after withdrawing all).
    pub new_decryptable_available_balance: Option<[u8; NEW_DECRYPTABLE_LEN]>,
}

impl<'a> TryFrom<&'a [u8]> for CloseContinuousPoolData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let new_decryptable_available_balance = if data.len() >= NEW_DECRYPTABLE_LEN {
            Some(data[0..NEW_DECRYPTABLE_LEN].try_into().map_err(|_| ProgramError::InvalidInstructionData)?)
        } else {
            None
        };
        Ok(Self { new_decryptable_available_balance })
    }
}

impl<'a> InstructionData<'a> for CloseContinuousPoolData {
    const LEN: usize = 0;
}
