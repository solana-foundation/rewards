use codama::CodamaType;
use pinocchio::error::ProgramError;

use crate::errors::RewardsProgramError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, CodamaType)]
pub enum BalanceSource {
    OnChain,
    AuthoritySet,
}

impl TryFrom<u8> for BalanceSource {
    type Error = ProgramError;

    fn try_from(byte: u8) -> Result<Self, ProgramError> {
        match byte {
            0 => Ok(BalanceSource::OnChain),
            1 => Ok(BalanceSource::AuthoritySet),
            _ => Err(RewardsProgramError::InvalidBalanceSource.into()),
        }
    }
}

impl BalanceSource {
    pub fn to_byte(&self) -> u8 {
        match self {
            BalanceSource::OnChain => 0,
            BalanceSource::AuthoritySet => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_source_try_from_on_chain() {
        let source = BalanceSource::try_from(0).unwrap();
        assert_eq!(source, BalanceSource::OnChain);
    }

    #[test]
    fn test_balance_source_try_from_authority_set() {
        let source = BalanceSource::try_from(1).unwrap();
        assert_eq!(source, BalanceSource::AuthoritySet);
    }

    #[test]
    fn test_balance_source_try_from_invalid() {
        let result = BalanceSource::try_from(2);
        assert_eq!(result.err(), Some(ProgramError::Custom(RewardsProgramError::InvalidBalanceSource as u32)));
    }

    #[test]
    fn test_balance_source_to_byte() {
        assert_eq!(BalanceSource::OnChain.to_byte(), 0);
        assert_eq!(BalanceSource::AuthoritySet.to_byte(), 1);
    }

    #[test]
    fn test_balance_source_roundtrip() {
        for byte in 0..=1 {
            let source = BalanceSource::try_from(byte).unwrap();
            assert_eq!(source.to_byte(), byte);
        }
    }
}
