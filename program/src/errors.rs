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

    /// (20) No users opted in to receive rewards
    #[error("No users opted in to receive rewards")]
    NoOptedInUsers,

    /// (21) User is already opted in
    #[error("User is already opted in")]
    UserAlreadyOptedIn,

    /// (22) User is not opted in
    #[error("User is not opted in")]
    UserNotOptedIn,

    /// (23) Distribution amount too small for opted-in supply
    #[error("Distribution amount too small for opted-in supply")]
    DistributionAmountTooSmall,

    /// (24) Tracked mint does not match pool
    #[error("Tracked mint does not match pool")]
    TrackedMintMismatch,

    /// (25) Reward mint does not match pool
    #[error("Reward mint does not match pool")]
    RewardMintMismatch,

    /// (26) Invalid balance source mode
    #[error("Invalid balance source mode")]
    InvalidBalanceSource,

    /// (27) Instruction not allowed for this pool's balance source mode
    #[error("Instruction not allowed for this pool's balance source mode")]
    BalanceSourceMismatch,

    /// (28) User has been revoked from this reward pool
    #[error("User has been revoked from this reward pool")]
    UserRevoked,

    /// (29) User has already been revoked from this reward pool
    #[error("User has already been revoked from this reward pool")]
    UserAlreadyRevoked,

    /// (30) Invalid timestamp value
    #[error("Invalid timestamp value")]
    InvalidTimestamp,
}

impl From<RewardsProgramError> for ProgramError {
    fn from(e: RewardsProgramError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
