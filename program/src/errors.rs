use codama::CodamaErrors;
use pinocchio::error::ProgramError;
use thiserror::Error;

/// Errors that may be returned by the Rewards Program.
#[derive(Clone, Debug, Eq, PartialEq, Error, CodamaErrors)]
pub enum RewardsProgramError {
    /// (0) Invalid amount specified
    #[error("Invalid amount specified")]
    InvalidAmount,

    /// (1) Invalid time window configuration
    #[error("Invalid time window configuration")]
    InvalidTimeWindow,

    /// (2) Invalid schedule type
    #[error("Invalid schedule type")]
    InvalidScheduleType,

    /// (3) Unauthorized authority
    #[error("Unauthorized authority")]
    UnauthorizedAuthority,

    /// (4) Unauthorized recipient
    #[error("Unauthorized recipient")]
    UnauthorizedRecipient,

    /// (5) Insufficient funds in distribution
    #[error("Insufficient funds in distribution")]
    InsufficientFunds,

    /// (6) Nothing available to claim
    #[error("Nothing available to claim")]
    NothingToClaim,

    /// (7) Math overflow occurred
    #[error("Math overflow occurred")]
    MathOverflow,

    /// (8) Invalid account data
    #[error("Invalid account data")]
    InvalidAccountData,

    /// (9) Event authority PDA is invalid
    #[error("Event authority PDA is invalid")]
    InvalidEventAuthority,

    /// (10) Rent calculation failed
    #[error("Rent calculation failed")]
    RentCalculationFailed,

    /// (11) Requested claim amount exceeds available balance
    #[error("Requested claim amount exceeds available balance")]
    ExceedsClaimableAmount,

    /// (12) Invalid merkle proof
    #[error("Invalid merkle proof")]
    InvalidMerkleProof,

    /// (13) Clawback timestamp not yet reached
    #[error("Clawback timestamp not yet reached")]
    ClawbackNotReached,

    /// (14) Claim has not been fully vested
    #[error("Claim has not been fully vested")]
    ClaimNotFullyVested,

    /// (15) Invalid cliff timestamp
    #[error("Invalid cliff timestamp")]
    InvalidCliffTimestamp,

    /// (16) Claimed amount cannot decrease
    #[error("Claimed amount cannot decrease")]
    ClaimedAmountDecreased,

    /// (17) Distribution is not revocable
    #[error("Distribution is not revocable")]
    DistributionNotRevocable,

    /// (18) Invalid revoke mode
    #[error("Invalid revoke mode")]
    InvalidRevokeMode,

    /// (19) Claimant has already been revoked
    #[error("Claimant has already been revoked")]
    ClaimantAlreadyRevoked,

    /// (20) Transfer hook mints are not supported
    #[error("Transfer hook mints are not supported")]
    TransferHookMintUnsupported,

    /// (21) Distribution has been permanently closed
    #[error("Distribution has been permanently closed")]
    DistributionPermanentlyClosed,
}

impl From<RewardsProgramError> for ProgramError {
    fn from(e: RewardsProgramError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_are_contiguous() {
        let codes = [
            RewardsProgramError::InvalidAmount as u32,
            RewardsProgramError::InvalidTimeWindow as u32,
            RewardsProgramError::InvalidScheduleType as u32,
            RewardsProgramError::UnauthorizedAuthority as u32,
            RewardsProgramError::UnauthorizedRecipient as u32,
            RewardsProgramError::InsufficientFunds as u32,
            RewardsProgramError::NothingToClaim as u32,
            RewardsProgramError::MathOverflow as u32,
            RewardsProgramError::InvalidAccountData as u32,
            RewardsProgramError::InvalidEventAuthority as u32,
            RewardsProgramError::RentCalculationFailed as u32,
            RewardsProgramError::ExceedsClaimableAmount as u32,
            RewardsProgramError::InvalidMerkleProof as u32,
            RewardsProgramError::ClawbackNotReached as u32,
            RewardsProgramError::ClaimNotFullyVested as u32,
            RewardsProgramError::InvalidCliffTimestamp as u32,
            RewardsProgramError::ClaimedAmountDecreased as u32,
            RewardsProgramError::DistributionNotRevocable as u32,
            RewardsProgramError::InvalidRevokeMode as u32,
            RewardsProgramError::ClaimantAlreadyRevoked as u32,
            RewardsProgramError::TransferHookMintUnsupported as u32,
            RewardsProgramError::DistributionPermanentlyClosed as u32,
        ];

        for (expected, actual) in codes.iter().copied().enumerate() {
            assert_eq!(actual, expected as u32);
        }
    }
}
