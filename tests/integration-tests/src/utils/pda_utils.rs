use rewards_program_client::REWARDS_PROGRAM_ID;
use solana_sdk::pubkey::Pubkey;

const DIRECT_DISTRIBUTION_SEED: &[u8] = b"direct_distribution";
const DIRECT_RECIPIENT_SEED: &[u8] = b"direct_recipient";
const MERKLE_DISTRIBUTION_SEED: &[u8] = b"merkle_distribution";
const MERKLE_CLAIM_SEED: &[u8] = b"merkle_claim";
const REVOCATION_SEED: &[u8] = b"revocation";
const REWARD_POOL_SEED: &[u8] = b"reward_pool";
const USER_REWARD_SEED: &[u8] = b"user_reward";
const EVENT_AUTHORITY_SEED: &[u8] = b"event_authority";

pub fn find_direct_distribution_pda(mint: &Pubkey, authority: &Pubkey, seeds: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[DIRECT_DISTRIBUTION_SEED, mint.as_ref(), authority.as_ref(), seeds.as_ref()],
        &REWARDS_PROGRAM_ID,
    )
}

pub fn find_direct_recipient_pda(distribution: &Pubkey, recipient: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[DIRECT_RECIPIENT_SEED, distribution.as_ref(), recipient.as_ref()],
        &REWARDS_PROGRAM_ID,
    )
}

pub fn find_event_authority_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[EVENT_AUTHORITY_SEED], &REWARDS_PROGRAM_ID)
}

pub fn find_merkle_distribution_pda(mint: &Pubkey, authority: &Pubkey, seeds: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[MERKLE_DISTRIBUTION_SEED, mint.as_ref(), authority.as_ref(), seeds.as_ref()],
        &REWARDS_PROGRAM_ID,
    )
}

pub fn find_merkle_claim_pda(distribution: &Pubkey, claimant: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MERKLE_CLAIM_SEED, distribution.as_ref(), claimant.as_ref()], &REWARDS_PROGRAM_ID)
}

pub fn find_revocation_pda(parent: &Pubkey, user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[REVOCATION_SEED, parent.as_ref(), user.as_ref()], &REWARDS_PROGRAM_ID)
}

pub fn find_reward_pool_pda(
    reward_mint: &Pubkey,
    tracked_mint: &Pubkey,
    authority: &Pubkey,
    seed: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[REWARD_POOL_SEED, reward_mint.as_ref(), tracked_mint.as_ref(), authority.as_ref(), seed.as_ref()],
        &REWARDS_PROGRAM_ID,
    )
}

pub fn find_user_reward_account_pda(reward_pool: &Pubkey, user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[USER_REWARD_SEED, reward_pool.as_ref(), user.as_ref()], &REWARDS_PROGRAM_ID)
}
