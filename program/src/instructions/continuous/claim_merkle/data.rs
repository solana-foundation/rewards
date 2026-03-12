use alloc::vec::Vec;
use pinocchio::error::ProgramError;

use crate::{require_len, traits::InstructionData};

pub struct ClaimContinuousMerkleData {
    pub claim_bump: u8,
    pub root_version: u64,
    pub cumulative_amount: u64,
    pub amount: u64,
    pub proof: Vec<[u8; 32]>,
}

impl<'a> TryFrom<&'a [u8]> for ClaimContinuousMerkleData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        // claim_bump(1) + root_version(8) + cumulative_amount(8) + amount(8) + proof_len(4)
        require_len!(data, Self::LEN);

        let claim_bump = data[0];
        let root_version = u64::from_le_bytes(data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        let cumulative_amount =
            u64::from_le_bytes(data[9..17].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        let amount = u64::from_le_bytes(data[17..25].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        let proof_len =
            u32::from_le_bytes(data[25..29].try_into().map_err(|_| ProgramError::InvalidInstructionData)?) as usize;

        let proof_start = 29;
        let expected_len = proof_start + proof_len * 32;
        require_len!(data, expected_len);

        let mut proof = Vec::with_capacity(proof_len);
        for i in 0..proof_len {
            let start = proof_start + i * 32;
            let end = start + 32;
            let hash: [u8; 32] = data[start..end].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
            proof.push(hash);
        }

        Ok(Self { claim_bump, root_version, cumulative_amount, amount, proof })
    }
}

impl<'a> InstructionData<'a> for ClaimContinuousMerkleData {
    const LEN: usize = 29;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_data(amount: u64, proof: &[[u8; 32]]) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(255); // claim bump
        data.extend_from_slice(&2u64.to_le_bytes()); // root_version
        data.extend_from_slice(&1000u64.to_le_bytes()); // cumulative_amount
        data.extend_from_slice(&amount.to_le_bytes()); // amount
        data.extend_from_slice(&(proof.len() as u32).to_le_bytes()); // proof_len
        for p in proof {
            data.extend_from_slice(p);
        }
        data
    }

    #[test]
    fn test_try_from_no_proof() {
        let data = build_data(0, &[]);
        let parsed = ClaimContinuousMerkleData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.claim_bump, 255);
        assert_eq!(parsed.root_version, 2);
        assert_eq!(parsed.cumulative_amount, 1000);
        assert_eq!(parsed.amount, 0);
        assert!(parsed.proof.is_empty());
    }

    #[test]
    fn test_try_from_with_proof() {
        let proof = [[1u8; 32], [2u8; 32]];
        let data = build_data(500, &proof);
        let parsed = ClaimContinuousMerkleData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.amount, 500);
        assert_eq!(parsed.proof.len(), 2);
        assert_eq!(parsed.proof[0], [1u8; 32]);
        assert_eq!(parsed.proof[1], [2u8; 32]);
    }

    #[test]
    fn test_try_from_too_short() {
        let data = [0u8; 10];
        let result = ClaimContinuousMerkleData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_try_from_invalid_proof_len() {
        let mut data = build_data(0, &[]);
        data.truncate(20);
        let result = ClaimContinuousMerkleData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::InvalidInstructionData));
    }
}
