use pinocchio::error::ProgramError;

use crate::{
    require_len,
    traits::InstructionData,
    utils::{RevokeMode, CONFIDENTIAL_TRANSFER_DATA_LEN},
};

pub struct RevokeContinuousUserData {
    pub revoke_mode: RevokeMode,
    /// Present when the pool has `confidential_rewards` enabled.
    /// Packed 167-byte buffer; interpreted by the processor via `ConfidentialTransferData::try_from_bytes`.
    pub confidential_transfer_bytes: Option<[u8; CONFIDENTIAL_TRANSFER_DATA_LEN]>,
}

impl<'a> TryFrom<&'a [u8]> for RevokeContinuousUserData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let revoke_mode = RevokeMode::try_from(data[0])?;

        let confidential_transfer_bytes = if data.len() >= Self::LEN + CONFIDENTIAL_TRANSFER_DATA_LEN {
            Some(
                data[Self::LEN..Self::LEN + CONFIDENTIAL_TRANSFER_DATA_LEN]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            )
        } else {
            None
        };

        Ok(Self { revoke_mode, confidential_transfer_bytes })
    }
}

impl<'a> InstructionData<'a> for RevokeContinuousUserData {
    const LEN: usize = 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::RewardsProgramError;

    #[test]
    fn test_try_from_valid_non_vested() {
        let data = [0u8];
        let result = RevokeContinuousUserData::try_from(&data[..]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().revoke_mode, RevokeMode::NonVested);
    }

    #[test]
    fn test_try_from_valid_full() {
        let data = [1u8];
        let result = RevokeContinuousUserData::try_from(&data[..]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().revoke_mode, RevokeMode::Full);
    }

    #[test]
    fn test_try_from_data_too_short() {
        let data: [u8; 0] = [];
        let result = RevokeContinuousUserData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_try_from_invalid_mode() {
        let data = [2u8];
        let result = RevokeContinuousUserData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::Custom(RewardsProgramError::InvalidRevokeMode as u32)));
    }
}
