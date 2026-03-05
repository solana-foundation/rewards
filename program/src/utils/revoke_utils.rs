use codama::CodamaType;
use pinocchio::error::ProgramError;

use crate::errors::RewardsProgramError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, CodamaType)]
pub enum RevokeMode {
    NonVested,
    Full,
}

impl TryFrom<u8> for RevokeMode {
    type Error = ProgramError;

    fn try_from(byte: u8) -> Result<Self, ProgramError> {
        match byte {
            0 => Ok(RevokeMode::NonVested),
            1 => Ok(RevokeMode::Full),
            _ => Err(RewardsProgramError::InvalidRevokeMode.into()),
        }
    }
}

impl RevokeMode {
    pub fn to_byte(&self) -> u8 {
        match self {
            RevokeMode::NonVested => 0,
            RevokeMode::Full => 1,
        }
    }

    pub fn to_bit(&self) -> u8 {
        1 << self.to_byte()
    }

    pub fn is_disabled_by(&self, revocable: u8) -> bool {
        revocable & self.to_bit() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revoke_mode_try_from_non_vested() {
        let mode = RevokeMode::try_from(0).unwrap();
        assert_eq!(mode, RevokeMode::NonVested);
    }

    #[test]
    fn test_revoke_mode_try_from_full() {
        let mode = RevokeMode::try_from(1).unwrap();
        assert_eq!(mode, RevokeMode::Full);
    }

    #[test]
    fn test_revoke_mode_try_from_invalid() {
        let result = RevokeMode::try_from(2);
        assert_eq!(result.err(), Some(ProgramError::Custom(RewardsProgramError::InvalidRevokeMode as u32)));
    }

    #[test]
    fn test_revoke_mode_to_byte() {
        assert_eq!(RevokeMode::NonVested.to_byte(), 0);
        assert_eq!(RevokeMode::Full.to_byte(), 1);
    }

    #[test]
    fn test_revoke_mode_roundtrip() {
        for byte in 0..=1 {
            let mode = RevokeMode::try_from(byte).unwrap();
            assert_eq!(mode.to_byte(), byte);
        }
    }

    #[test]
    fn test_revoke_mode_to_bit() {
        assert_eq!(RevokeMode::NonVested.to_bit(), 0b01);
        assert_eq!(RevokeMode::Full.to_bit(), 0b10);
    }

    #[test]
    fn test_revoke_mode_is_disabled_by() {
        assert!(RevokeMode::NonVested.is_disabled_by(0));
        assert!(RevokeMode::Full.is_disabled_by(0));

        assert!(!RevokeMode::NonVested.is_disabled_by(1));
        assert!(RevokeMode::Full.is_disabled_by(1));

        assert!(RevokeMode::NonVested.is_disabled_by(2));
        assert!(!RevokeMode::Full.is_disabled_by(2));

        assert!(!RevokeMode::NonVested.is_disabled_by(3));
        assert!(!RevokeMode::Full.is_disabled_by(3));
    }

    #[test]
    fn test_revoke_mode_bitmask_combinations() {
        let non_vested_bit = RevokeMode::NonVested.to_bit();
        let full_bit = RevokeMode::Full.to_bit();

        let both = non_vested_bit | full_bit;
        assert_eq!(both, 0b11);
        assert_ne!(both & non_vested_bit, 0);
        assert_ne!(both & full_bit, 0);

        let none: u8 = 0;
        assert_eq!(none & non_vested_bit, 0);
        assert_eq!(none & full_bit, 0);

        let only_full = full_bit;
        assert_eq!(only_full & non_vested_bit, 0);
        assert_ne!(only_full & full_bit, 0);
    }
}
