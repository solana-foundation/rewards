use pinocchio::error::ProgramError;

use crate::{errors::RewardsProgramError, require_len, traits::InstructionData, utils::BalanceSource};

pub struct CreateContinuousPoolData {
    pub bump: u8,
    pub balance_source: BalanceSource,
    pub revocable: u8,
    pub clawback_ts: i64,
}

impl<'a> TryFrom<&'a [u8]> for CreateContinuousPoolData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let bump = data[0];
        let balance_source = BalanceSource::try_from(data[1])?;
        let revocable = data[2];
        let clawback_ts = i64::from_le_bytes(data[3..11].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Self { bump, balance_source, revocable, clawback_ts })
    }
}

impl<'a> InstructionData<'a> for CreateContinuousPoolData {
    const LEN: usize = 11; // bump(1) + balance_source(1) + revocable(1) + clawback_ts(8)

    fn validate(&self) -> Result<(), ProgramError> {
        if self.clawback_ts < 0 {
            return Err(RewardsProgramError::InvalidTimestamp.into());
        }
        Ok(())
    }
}
