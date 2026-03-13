use pinocchio::error::ProgramError;

use crate::{require_len, traits::InstructionData, utils::CONFIDENTIAL_TRANSFER_DATA_LEN};

pub struct ClaimContinuousData {
    pub amount: u64,
    /// Present when the pool has `confidential_rewards` enabled.
    /// Packed 167-byte buffer; interpreted by the processor via `ConfidentialTransferData::try_from_bytes`.
    pub confidential_transfer_bytes: Option<[u8; CONFIDENTIAL_TRANSFER_DATA_LEN]>,
}

impl<'a> TryFrom<&'a [u8]> for ClaimContinuousData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let amount = u64::from_le_bytes(data[0..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        let confidential_transfer_bytes = if data.len() >= Self::LEN + CONFIDENTIAL_TRANSFER_DATA_LEN {
            Some(
                data[Self::LEN..Self::LEN + CONFIDENTIAL_TRANSFER_DATA_LEN]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            )
        } else {
            None
        };

        Ok(Self { amount, confidential_transfer_bytes })
    }
}

impl<'a> InstructionData<'a> for ClaimContinuousData {
    const LEN: usize = 8;
}
