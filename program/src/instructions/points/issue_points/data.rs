use pinocchio::error::ProgramError;

use crate::{errors::RewardsProgramError, require_len, traits::InstructionData};

pub struct IssuePointsData {
    pub user_points_bump: u8,
    pub quantity: u64,
}

impl<'a> TryFrom<&'a [u8]> for IssuePointsData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let user_points_bump = data[0];
        let quantity = u64::from_le_bytes(data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Self { user_points_bump, quantity })
    }
}

impl<'a> InstructionData<'a> for IssuePointsData {
    const LEN: usize = 9; // user_points_bump(1) + quantity(8)

    fn validate(&self) -> Result<(), ProgramError> {
        if self.quantity == 0 {
            return Err(RewardsProgramError::InvalidAmount.into());
        }
        Ok(())
    }
}
