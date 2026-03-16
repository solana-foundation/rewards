use alloc::vec::Vec;
use codama::CodamaInstructions;

use crate::utils::{BalanceSource, RevokeMode, VestingSchedule};

/// Instructions for the Rewards Program.
#[repr(C, u8)]
#[derive(Clone, Debug, PartialEq, CodamaInstructions)]
pub enum RewardsProgramInstruction {
    /// Create a new direct distribution.
    #[codama(account(name = "payer", signer, writable, docs = "Pays for account creation"))]
    #[codama(account(name = "authority", signer, docs = "Distribution authority; stored on-chain"))]
    #[codama(account(name = "seeds", signer, docs = "Arbitrary signer used as PDA seed for uniqueness"))]
    #[codama(account(
        name = "distribution",
        writable,
        docs = "PDA: [b\"direct_distribution\", mint, authority, seeds] (created)"
    ))]
    #[codama(account(name = "mint", docs = "SPL token mint"))]
    #[codama(account(
        name = "distribution_vault",
        writable,
        docs = "ATA of distribution PDA for mint (created via CPI)"
    ))]
    #[codama(account(name = "system_program", docs = "System program"))]
    #[codama(account(name = "token_program", docs = "SPL Token or Token-2022 program"))]
    #[codama(account(name = "associated_token_program", docs = "Associated Token Account program"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID (for event CPI)"))]
    CreateDirectDistribution {
        /// Bump for the distribution PDA
        bump: u8,
        /// Bitmask of allowed revoke modes (0 = not revocable, bit 0 = NonVested, bit 1 = Full)
        revocable: u8,
        /// Timestamp after which authority can close the distribution (0 = no gate)
        clawback_ts: i64,
    } = 0,

    /// Add a recipient to a direct distribution.
    /// Each recipient has their own vesting schedule.
    /// Transfers the recipient's allocation amount into the distribution vault.
    #[codama(account(name = "payer", signer, writable, docs = "Pays for recipient PDA creation"))]
    #[codama(account(name = "authority", signer, docs = "Distribution authority; must match distribution.authority"))]
    #[codama(account(name = "distribution", writable, docs = "PDA: DirectDistribution account"))]
    #[codama(account(
        name = "recipient_account",
        writable,
        docs = "PDA: [b\"direct_recipient\", distribution, recipient] (created)"
    ))]
    #[codama(account(name = "recipient", docs = "Wallet address of the recipient (used as PDA seed)"))]
    #[codama(account(name = "mint", docs = "SPL token mint; must match distribution.mint"))]
    #[codama(account(
        name = "distribution_vault",
        writable,
        docs = "ATA of distribution PDA for mint; receives transferred tokens"
    ))]
    #[codama(account(
        name = "authority_token_account",
        writable,
        docs = "Authority's token account; source of tokens for this recipient's allocation"
    ))]
    #[codama(account(name = "system_program", docs = "System program"))]
    #[codama(account(name = "token_program", docs = "SPL Token or Token-2022 program"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    AddDirectRecipient {
        /// Bump for the recipient PDA
        bump: u8,
        /// Amount allocated to the recipient
        amount: u64,
        /// Vesting schedule
        schedule: VestingSchedule,
    } = 1,

    /// Claim tokens from a direct distribution.
    #[codama(account(
        name = "recipient",
        signer,
        docs = "Wallet address of the claiming recipient; must match recipient_account.recipient"
    ))]
    #[codama(account(name = "distribution", writable, docs = "PDA: DirectDistribution account"))]
    #[codama(account(
        name = "recipient_account",
        writable,
        docs = "PDA: [b\"direct_recipient\", distribution, recipient]"
    ))]
    #[codama(account(name = "mint", docs = "SPL token mint"))]
    #[codama(account(
        name = "distribution_vault",
        writable,
        docs = "ATA of distribution PDA for mint; source of claimed tokens"
    ))]
    #[codama(account(
        name = "recipient_token_account",
        writable,
        docs = "Recipient's token account; destination for claimed tokens"
    ))]
    #[codama(account(name = "token_program", docs = "SPL Token or Token-2022 program"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    ClaimDirect {
        /// Amount to claim. 0 = claim all available.
        amount: u64,
    } = 2,

    /// Close a direct distribution and recover remaining tokens.
    #[codama(account(
        name = "authority",
        signer,
        writable,
        docs = "Distribution authority; receives rent + remaining distribution vault tokens"
    ))]
    #[codama(account(name = "distribution", writable, docs = "PDA: DirectDistribution account (closed)"))]
    #[codama(account(name = "mint", docs = "SPL token mint"))]
    #[codama(account(
        name = "distribution_vault",
        writable,
        docs = "ATA of distribution PDA for mint; remaining tokens returned to authority"
    ))]
    #[codama(account(
        name = "authority_token_account",
        writable,
        docs = "Authority's token account; destination for remaining tokens"
    ))]
    #[codama(account(name = "token_program", docs = "SPL Token or Token-2022 program"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    CloseDirectDistribution {} = 3,

    /// Close a direct recipient account after fully claiming, recovering rent.
    #[codama(account(
        name = "recipient",
        signer,
        docs = "Wallet address of the recipient; must match recipient_account.recipient"
    ))]
    #[codama(account(
        name = "original_payer",
        writable,
        docs = "Original payer of recipient PDA; receives rent refund"
    ))]
    #[codama(account(
        name = "distribution",
        docs = "PDA: DirectDistribution account; must be closed (owner = system program) or fully claimed"
    ))]
    #[codama(account(
        name = "recipient_account",
        writable,
        docs = "PDA: [b\"direct_recipient\", distribution, recipient] (closed)"
    ))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    CloseDirectRecipient {} = 4,

    /// Create a new merkle distribution with initial funding.
    #[codama(account(name = "payer", signer, writable, docs = "Pays for account creation and token transfer"))]
    #[codama(account(name = "authority", signer, docs = "Distribution authority; stored on-chain"))]
    #[codama(account(name = "seeds", signer, docs = "Arbitrary signer used as PDA seed for uniqueness"))]
    #[codama(account(
        name = "distribution",
        writable,
        docs = "PDA: [b\"merkle_distribution\", mint, authority, seeds] (created)"
    ))]
    #[codama(account(name = "mint", docs = "SPL token mint"))]
    #[codama(account(
        name = "distribution_vault",
        writable,
        docs = "ATA of distribution PDA for mint (created via CPI)"
    ))]
    #[codama(account(
        name = "authority_token_account",
        writable,
        docs = "Authority's token account; source of initial funding"
    ))]
    #[codama(account(name = "system_program", docs = "System program"))]
    #[codama(account(name = "token_program", docs = "SPL Token or Token-2022 program"))]
    #[codama(account(name = "associated_token_program", docs = "Associated Token Account program"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    CreateMerkleDistribution {
        /// Bump for the distribution PDA
        bump: u8,
        /// Bitmask of allowed revoke modes (0 = not revocable, bit 0 = NonVested, bit 1 = Full)
        revocable: u8,
        /// Amount of tokens to deposit in distribution vault
        amount: u64,
        /// Merkle root hash
        merkle_root: [u8; 32],
        /// Total amount claimable by all recipients
        total_amount: u64,
        /// Timestamp after which authority can close the distribution
        clawback_ts: i64,
    } = 5,

    /// Claim tokens from a merkle distribution.
    #[codama(account(name = "payer", signer, writable, docs = "Pays for claim PDA creation (if first claim)"))]
    #[codama(account(name = "claimant", signer, docs = "Wallet address of the claimant; proven via merkle proof"))]
    #[codama(account(name = "distribution", writable, docs = "PDA: MerkleDistribution account"))]
    #[codama(account(
        name = "claim_account",
        writable,
        docs = "PDA: [b\"merkle_claim\", distribution, claimant] (created or updated)"
    ))]
    #[codama(account(
        name = "revocation_marker",
        docs = "PDA: [b\"revocation\", distribution, claimant] (checked for existence)"
    ))]
    #[codama(account(name = "mint", docs = "SPL token mint"))]
    #[codama(account(
        name = "distribution_vault",
        writable,
        docs = "ATA of distribution PDA for mint; source of claimed tokens"
    ))]
    #[codama(account(
        name = "claimant_token_account",
        writable,
        docs = "Claimant's token account; destination for claimed tokens"
    ))]
    #[codama(account(name = "system_program", docs = "System program"))]
    #[codama(account(name = "token_program", docs = "SPL Token or Token-2022 program"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    ClaimMerkle {
        /// Bump for the claim PDA
        claim_bump: u8,
        /// Total amount allocated to claimant (from merkle leaf)
        total_amount: u64,
        /// Amount to claim (0 = claim all available)
        amount: u64,
        /// Vesting schedule (from merkle leaf)
        schedule: VestingSchedule,
        /// Merkle proof
        proof: Vec<[u8; 32]>,
    } = 6,

    /// Close a merkle claim after distribution is closed.
    #[codama(account(
        name = "claimant",
        signer,
        writable,
        docs = "Wallet address of the claimant; receives rent refund"
    ))]
    #[codama(account(
        name = "distribution",
        docs = "PDA: MerkleDistribution account; must be closed (owner = system program)"
    ))]
    #[codama(account(
        name = "claim_account",
        writable,
        docs = "PDA: [b\"merkle_claim\", distribution, claimant] (closed)"
    ))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    CloseMerkleClaim {} = 7,

    /// Close a merkle distribution after clawback timestamp.
    #[codama(account(
        name = "authority",
        signer,
        writable,
        docs = "Distribution authority; receives rent + remaining distribution vault tokens"
    ))]
    #[codama(account(name = "distribution", writable, docs = "PDA: MerkleDistribution account (closed)"))]
    #[codama(account(name = "mint", docs = "SPL token mint"))]
    #[codama(account(
        name = "distribution_vault",
        writable,
        docs = "ATA of distribution PDA for mint; remaining tokens returned to authority"
    ))]
    #[codama(account(
        name = "authority_token_account",
        writable,
        docs = "Authority's token account; destination for remaining tokens"
    ))]
    #[codama(account(name = "token_program", docs = "SPL Token or Token-2022 program"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    CloseMerkleDistribution {} = 8,

    /// Revoke a recipient from a revocable direct distribution.
    /// Mode 0 (NonVested): transfers vested-but-unclaimed tokens to recipient, returns unvested tokens to authority.
    /// Mode 1 (Full): returns all unclaimed tokens (unvested + vested-unclaimed) to authority, nothing transferred to recipient.
    #[codama(account(name = "authority", signer, docs = "Distribution authority; must match distribution.authority"))]
    #[codama(account(name = "distribution", writable, docs = "PDA: DirectDistribution account"))]
    #[codama(account(
        name = "recipient_account",
        writable,
        docs = "PDA: [b\"direct_recipient\", distribution, recipient] (closed)"
    ))]
    #[codama(account(name = "recipient", docs = "Wallet address of the recipient"))]
    #[codama(account(
        name = "original_payer",
        writable,
        docs = "Original payer of recipient PDA; receives rent refund"
    ))]
    #[codama(account(name = "mint", docs = "SPL token mint"))]
    #[codama(account(
        name = "distribution_vault",
        writable,
        docs = "ATA of distribution PDA for mint; source of transferred tokens"
    ))]
    #[codama(account(
        name = "recipient_token_account",
        writable,
        docs = "Recipient's token account; destination for vested tokens (NonVested mode)"
    ))]
    #[codama(account(
        name = "authority_token_account",
        writable,
        docs = "Authority's token account; destination for returned tokens"
    ))]
    #[codama(account(name = "token_program", docs = "SPL Token or Token-2022 program"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    RevokeDirectRecipient {
        /// Revoke mode: NonVested (fair) or Full (clawback all)
        revoke_mode: RevokeMode,
    } = 9,

    /// Revoke a claimant from a merkle distribution.
    /// Authority provides the claimant's merkle leaf data for on-chain proof verification.
    /// Mode 0 (NonVested): transfers vested-but-unclaimed tokens to claimant, returns unvested tokens to authority.
    /// Mode 1 (Full): returns all unclaimed tokens (unvested + vested-unclaimed) to authority, nothing transferred to claimant.
    #[codama(account(name = "authority", signer, docs = "Distribution authority; must match distribution.authority"))]
    #[codama(account(name = "payer", signer, writable, docs = "Pays for PDA creation rent"))]
    #[codama(account(name = "distribution", writable, docs = "PDA: MerkleDistribution account"))]
    #[codama(account(
        name = "claim_account",
        docs = "PDA: [b\"merkle_claim\", distribution, claimant] (read-only, may not exist)"
    ))]
    #[codama(account(
        name = "revocation_marker",
        writable,
        docs = "PDA: [b\"revocation\", distribution, claimant] (created)"
    ))]
    #[codama(account(name = "claimant", docs = "Wallet address of the claimant being revoked"))]
    #[codama(account(name = "mint", docs = "SPL token mint"))]
    #[codama(account(
        name = "distribution_vault",
        writable,
        docs = "ATA of distribution PDA for mint; source of transferred tokens"
    ))]
    #[codama(account(
        name = "claimant_token_account",
        writable,
        docs = "Claimant's token account; destination for vested tokens (NonVested mode)"
    ))]
    #[codama(account(
        name = "authority_token_account",
        writable,
        docs = "Authority's token account; destination for returned tokens"
    ))]
    #[codama(account(name = "system_program", docs = "System program"))]
    #[codama(account(name = "token_program", docs = "SPL Token or Token-2022 program"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    RevokeMerkleClaim {
        /// Revoke mode: NonVested (fair) or Full (clawback all)
        revoke_mode: RevokeMode,
        /// Total amount allocated to claimant (from merkle leaf)
        total_amount: u64,
        /// Vesting schedule (from merkle leaf)
        schedule: VestingSchedule,
        /// Merkle proof
        proof: Vec<[u8; 32]>,
    } = 10,

    /// Create a new continuous reward pool.
    #[codama(account(name = "payer", signer, writable, docs = "Pays for account creation"))]
    #[codama(account(name = "authority", signer, docs = "Pool authority; stored on-chain"))]
    #[codama(account(name = "seed", signer, docs = "Arbitrary signer used as PDA seed for uniqueness"))]
    #[codama(account(
        name = "reward_pool",
        writable,
        docs = "PDA: [b\"reward_pool\", reward_mint, tracked_mint, authority, seed] (created)"
    ))]
    #[codama(account(name = "tracked_mint", docs = "SPL token mint tracked for balance-based rewards (e.g. USD1)"))]
    #[codama(account(name = "reward_mint", docs = "SPL token mint distributed as reward"))]
    #[codama(account(
        name = "reward_vault",
        writable,
        docs = "ATA of reward_pool PDA for reward_mint (created via CPI)"
    ))]
    #[codama(account(name = "system_program", docs = "System program"))]
    #[codama(account(name = "reward_token_program", docs = "SPL Token or Token-2022 program for reward mint"))]
    #[codama(account(name = "associated_token_program", docs = "Associated Token Account program"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID (for event CPI)"))]
    CreateContinuousPool {
        /// Bump for the reward pool PDA
        bump: u8,
        /// Balance source mode: OnChain = on-chain token account, AuthoritySet = authority-set
        balance_source: BalanceSource,
        /// Bitmask of allowed revoke modes (0 = not revocable, bit 0 = NonVested, bit 1 = Full)
        revocable: u8,
        /// Timestamp after which authority can close the pool (0 = no gate)
        clawback_ts: i64,
        /// Non-zero to enable confidential reward transfers via Token-2022 ConfidentialTransfer.
        confidential_rewards: u8,
    } = 11,

    /// Opt in to a continuous reward pool. Creates a UserRewardAccount.
    #[codama(account(name = "payer", signer, writable, docs = "Pays for UserRewardAccount PDA creation"))]
    #[codama(account(name = "user", signer, docs = "User opting in; stored on-chain"))]
    #[codama(account(name = "reward_pool", writable, docs = "PDA: RewardPool account"))]
    #[codama(account(
        name = "user_reward_account",
        writable,
        docs = "PDA: [b\"user_reward\", reward_pool, user] (created)"
    ))]
    #[codama(account(
        name = "revocation_marker",
        docs = "PDA: [b\"revocation\", reward_pool, user] (checked for existence; must be uninitialized)"
    ))]
    #[codama(account(
        name = "user_tracked_token_account",
        docs = "User's tracked token account (read for initial balance)"
    ))]
    #[codama(account(name = "tracked_mint", docs = "SPL token mint; must match pool tracked_mint"))]
    #[codama(account(name = "system_program", docs = "System program"))]
    #[codama(account(name = "tracked_token_program", docs = "SPL Token or Token-2022 program for tracked mint"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    ContinuousOptIn {
        /// Bump for the user reward account PDA
        bump: u8,
    } = 12,

    /// Opt out of a continuous reward pool. Settles rewards, closes UserRewardAccount.
    #[codama(account(name = "user", signer, writable, docs = "User opting out; receives rent refund"))]
    #[codama(account(name = "reward_pool", writable, docs = "PDA: RewardPool account"))]
    #[codama(account(
        name = "user_reward_account",
        writable,
        docs = "PDA: [b\"user_reward\", reward_pool, user] (closed)"
    ))]
    #[codama(account(
        name = "user_tracked_token_account",
        docs = "User's tracked token account (read for balance sync)"
    ))]
    #[codama(account(
        name = "reward_vault",
        writable,
        docs = "ATA of reward_pool PDA for reward_mint; source of claimed tokens"
    ))]
    #[codama(account(
        name = "user_reward_token_account",
        writable,
        docs = "User's reward token account; destination for claimed tokens"
    ))]
    #[codama(account(name = "tracked_mint", docs = "SPL token mint; must match pool tracked_mint"))]
    #[codama(account(name = "reward_mint", docs = "SPL token mint; must match reward_pool.reward_mint"))]
    #[codama(account(name = "tracked_token_program", docs = "SPL Token or Token-2022 program for tracked mint"))]
    #[codama(account(name = "reward_token_program", docs = "SPL Token or Token-2022 program for reward mint"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    ContinuousOptOut {} = 13,

    /// Distribute reward tokens to the pool, increasing reward_per_token.
    #[codama(account(name = "authority", signer, docs = "Pool authority; must match reward_pool.authority"))]
    #[codama(account(name = "reward_pool", writable, docs = "PDA: RewardPool account"))]
    #[codama(account(name = "reward_mint", docs = "SPL token mint; must match reward_pool.reward_mint"))]
    #[codama(account(
        name = "reward_vault",
        writable,
        docs = "ATA of reward_pool PDA for reward_mint; receives deposited tokens"
    ))]
    #[codama(account(
        name = "authority_token_account",
        writable,
        docs = "Authority's reward token account; source of deposited tokens"
    ))]
    #[codama(account(name = "reward_token_program", docs = "SPL Token or Token-2022 program for reward mint"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    DistributeContinuousReward {
        /// Amount of reward tokens to distribute
        amount: u64,
        /// Expected pending balance credit counter for confidential pools.
        /// Must be `vault_counter + 1` (one ConfidentialDeposit fires in this tx).
        /// Ignored for non-confidential pools.
        expected_pending_balance_credit_counter: u64,
    } = 14,

    /// Claim accumulated rewards from a continuous reward pool.
    #[codama(account(name = "user", signer, docs = "User claiming rewards"))]
    #[codama(account(name = "reward_pool", writable, docs = "PDA: RewardPool account"))]
    #[codama(account(name = "user_reward_account", writable, docs = "PDA: [b\"user_reward\", reward_pool, user]"))]
    #[codama(account(
        name = "user_tracked_token_account",
        docs = "User's tracked token account (read for balance sync)"
    ))]
    #[codama(account(
        name = "reward_vault",
        writable,
        docs = "ATA of reward_pool PDA for reward_mint; source of claimed tokens"
    ))]
    #[codama(account(
        name = "user_reward_token_account",
        writable,
        docs = "User's reward token account; destination for claimed tokens"
    ))]
    #[codama(account(name = "tracked_mint", docs = "SPL token mint; must match pool tracked_mint"))]
    #[codama(account(name = "reward_mint", docs = "SPL token mint; must match reward_pool.reward_mint"))]
    #[codama(account(name = "tracked_token_program", docs = "SPL Token or Token-2022 program for tracked mint"))]
    #[codama(account(name = "reward_token_program", docs = "SPL Token or Token-2022 program for reward mint"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    ClaimContinuous {
        /// Amount to claim. 0 = claim all available.
        amount: u64,
    } = 15,

    /// Sync a user's tracked balance to their current on-chain token balance. Permissionless.
    #[codama(account(name = "reward_pool", writable, docs = "PDA: RewardPool account (balance_source must be 0)"))]
    #[codama(account(name = "user_reward_account", writable, docs = "PDA: [b\"user_reward\", reward_pool, user]"))]
    #[codama(account(name = "user", docs = "User wallet; used for PDA derivation"))]
    #[codama(account(
        name = "user_tracked_token_account",
        docs = "User's tracked token account (read for current balance)"
    ))]
    #[codama(account(name = "tracked_mint", docs = "SPL token mint; must match pool tracked_mint"))]
    #[codama(account(name = "tracked_token_program", docs = "SPL Token or Token-2022 program for tracked mint"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    SyncContinuousBalance {} = 16,

    /// Authority sets a user's tracked balance directly (for off-chain/cross-chain data).
    #[codama(account(name = "authority", signer, docs = "Pool authority; must match reward_pool.authority"))]
    #[codama(account(name = "reward_pool", writable, docs = "PDA: RewardPool account (balance_source must be 1)"))]
    #[codama(account(name = "user_reward_account", writable, docs = "PDA: [b\"user_reward\", reward_pool, user]"))]
    #[codama(account(name = "user", docs = "User wallet; used for PDA derivation"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    SetContinuousBalance {
        /// New balance to set for the user
        balance: u64,
    } = 17,

    /// Close a continuous reward pool and recover remaining reward tokens.
    #[codama(account(
        name = "authority",
        signer,
        writable,
        docs = "Pool authority; receives rent + remaining reward vault tokens"
    ))]
    #[codama(account(name = "reward_pool", writable, docs = "PDA: RewardPool account (closed)"))]
    #[codama(account(name = "reward_mint", docs = "SPL token mint; must match reward_pool.reward_mint"))]
    #[codama(account(
        name = "reward_vault",
        writable,
        docs = "ATA of reward_pool PDA for reward_mint; remaining tokens returned to authority"
    ))]
    #[codama(account(
        name = "authority_token_account",
        writable,
        docs = "Authority's reward token account; destination for remaining tokens"
    ))]
    #[codama(account(name = "reward_token_program", docs = "SPL Token or Token-2022 program for reward mint"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    CloseContinuousPool {} = 18,

    /// Revoke a user from a continuous reward pool.
    /// Authority force-removes a user and creates a revocation marker PDA to prevent re-opt-in.
    /// Mode 0 (NonVested): transfers accrued rewards to user.
    /// Mode 1 (Full): forfeits all accrued rewards; transfers them to authority.
    #[codama(account(name = "authority", signer, docs = "Pool authority; must match reward_pool.authority"))]
    #[codama(account(name = "payer", signer, writable, docs = "Pays for revocation PDA creation"))]
    #[codama(account(name = "reward_pool", writable, docs = "PDA: RewardPool account"))]
    #[codama(account(
        name = "user_reward_account",
        writable,
        docs = "PDA: [b\"user_reward\", reward_pool, user] (closed)"
    ))]
    #[codama(account(
        name = "revocation_marker",
        writable,
        docs = "PDA: [b\"revocation\", reward_pool, user] (created)"
    ))]
    #[codama(account(name = "user", docs = "User being revoked"))]
    #[codama(account(
        name = "rent_destination",
        writable,
        docs = "Receives rent refund from closed UserRewardAccount; authority decides destination"
    ))]
    #[codama(account(
        name = "user_tracked_token_account",
        docs = "User's tracked token account (read for balance sync)"
    ))]
    #[codama(account(
        name = "reward_vault",
        writable,
        docs = "ATA of reward_pool PDA for reward_mint; source of reward transfer"
    ))]
    #[codama(account(
        name = "user_reward_token_account",
        writable,
        docs = "User's reward token account; destination for rewards (NonVested mode)"
    ))]
    #[codama(account(
        name = "authority_reward_token_account",
        writable,
        docs = "Authority's reward token account; destination for forfeited rewards (Full mode)"
    ))]
    #[codama(account(name = "tracked_mint", docs = "SPL token mint; must match pool tracked_mint"))]
    #[codama(account(name = "reward_mint", docs = "SPL token mint; must match reward_pool.reward_mint"))]
    #[codama(account(name = "system_program", docs = "System program"))]
    #[codama(account(name = "tracked_token_program", docs = "SPL Token or Token-2022 program for tracked mint"))]
    #[codama(account(name = "reward_token_program", docs = "SPL Token or Token-2022 program for reward mint"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    RevokeContinuousUser {
        /// Revoke mode: NonVested (transfer accrued rewards) or Full (forfeit all)
        revoke_mode: RevokeMode,
    } = 19,

    /// Set or rotate the continuous-merkle root for a reward pool.
    #[codama(account(name = "authority", signer, docs = "Pool authority; must match reward_pool.authority"))]
    #[codama(account(name = "reward_pool", writable, docs = "PDA: RewardPool account"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    SetContinuousMerkleRoot {
        /// Merkle root for cumulative continuous claims
        merkle_root: [u8; 32],
        /// Monotonically increasing root version (must be > current root version)
        root_version: u64,
    } = 20,

    /// Claim rewards from a continuous reward pool using a merkle proof over cumulative amounts.
    #[codama(account(name = "payer", signer, writable, docs = "Pays for claim PDA creation (if first claim)"))]
    #[codama(account(name = "user", signer, docs = "User claiming rewards"))]
    #[codama(account(name = "reward_pool", writable, docs = "PDA: RewardPool account"))]
    #[codama(account(
        name = "claim_account",
        writable,
        docs = "PDA: [b\"merkle_claim\", reward_pool, user] (created or updated)"
    ))]
    #[codama(account(
        name = "revocation_marker",
        docs = "PDA: [b\"revocation\", reward_pool, user] (checked for existence)"
    ))]
    #[codama(account(name = "reward_mint", docs = "SPL token mint; must match reward_pool.reward_mint"))]
    #[codama(account(
        name = "reward_vault",
        writable,
        docs = "ATA of reward_pool PDA for reward_mint; source of claimed tokens"
    ))]
    #[codama(account(
        name = "user_reward_token_account",
        writable,
        docs = "User's reward token account; destination for claimed tokens"
    ))]
    #[codama(account(name = "system_program", docs = "System program"))]
    #[codama(account(name = "reward_token_program", docs = "SPL Token or Token-2022 program for reward mint"))]
    #[codama(account(name = "event_authority", docs = "PDA: [b\"__event_authority\"] for event CPI"))]
    #[codama(account(name = "rewards_program", docs = "This program's ID"))]
    ClaimContinuousMerkle {
        /// Bump for the claim PDA
        claim_bump: u8,
        /// Merkle root version this proof targets
        root_version: u64,
        /// Cumulative amount claimable by user at this root_version
        cumulative_amount: u64,
        /// Amount to claim. 0 = claim all available.
        amount: u64,
        /// Merkle proof
        proof: Vec<[u8; 32]>,
    } = 21,

    /// Emit event data via CPI (prevents log truncation).
    #[codama(account(name = "event_authority", signer, docs = "PDA: [b\"__event_authority\"]; validates CPI caller"))]
    EmitEvent {} = 228,
}
