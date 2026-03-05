use pinocchio::error::ProgramError;

use crate::{require_len, traits::InstructionData, utils::RevokeMode};

pub struct RevokeDirectRecipientData {
    pub revoke_mode: RevokeMode,
}

impl<'a> TryFrom<&'a [u8]> for RevokeDirectRecipientData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let revoke_mode = RevokeMode::try_from(data[0])?;

        Ok(Self { revoke_mode })
    }
}

impl<'a> InstructionData<'a> for RevokeDirectRecipientData {
    const LEN: usize = 1; // revoke_mode discriminant

    fn validate(&self) -> Result<(), ProgramError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::RewardsProgramError;

    #[test]
    fn test_try_from_valid_non_vested() {
        let data = [0u8];
        let result = RevokeDirectRecipientData::try_from(&data[..]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().revoke_mode, RevokeMode::NonVested);
    }

    #[test]
    fn test_try_from_valid_full() {
        let data = [1u8];
        let result = RevokeDirectRecipientData::try_from(&data[..]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().revoke_mode, RevokeMode::Full);
    }

    #[test]
    fn test_try_from_data_too_short() {
        let data: [u8; 0] = [];
        let result = RevokeDirectRecipientData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_try_from_invalid_mode() {
        let data = [2u8];
        let result = RevokeDirectRecipientData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::Custom(RewardsProgramError::InvalidRevokeMode as u32)));
    }

    #[test]
    fn test_validate_always_ok() {
        let data = [0u8];
        let parsed = RevokeDirectRecipientData::try_from(&data[..]).unwrap();
        assert!(parsed.validate().is_ok());
    }
}
