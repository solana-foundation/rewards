use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};
use pinocchio_token_2022::instructions::TransferChecked;

use crate::{
    errors::RewardsProgramError,
    events::ClaimedEvent,
    state::{MerkleClaim, MerkleClaimSeeds, RewardPool},
    traits::{AccountParse, AccountSerialize, AccountSize, ClaimTracker, EventSerialize, PdaSeeds},
    utils::{
        compute_continuous_leaf_hash, create_pda_account_idempotent, emit_event, get_mint_decimals,
        is_pda_uninitialized, resolve_claim_amount, verify_not_revoked, verify_proof_or_error,
    },
    ID,
};

use super::ClaimContinuousMerkle;

pub fn process_claim_continuous_merkle(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = ClaimContinuousMerkle::try_from((instruction_data, accounts))?;

    let pool_data = ix.accounts.reward_pool.try_borrow()?;
    let mut pool = RewardPool::from_account(&pool_data, ix.accounts.reward_pool, &ID)?;
    drop(pool_data);

    pool.validate_reward_mint(ix.accounts.reward_mint.address())?;
    pool.validate_claim_root_version(ix.data.root_version)?;

    let leaf = compute_continuous_leaf_hash(
        ix.accounts.reward_pool.address(),
        ix.accounts.user.address(),
        ix.data.root_version,
        ix.data.cumulative_amount,
    );
    verify_proof_or_error(&ix.data.proof, &pool.merkle_root, &leaf)?;

    verify_not_revoked(
        ix.accounts.reward_pool.address(),
        ix.accounts.user.address(),
        ix.accounts.revocation_marker,
        &ID,
        RewardsProgramError::UserRevoked,
    )?;

    let claim_seeds =
        MerkleClaimSeeds { distribution: *ix.accounts.reward_pool.address(), claimant: *ix.accounts.user.address() };

    claim_seeds.validate_pda(ix.accounts.claim_account, &ID, ix.data.claim_bump)?;

    let claim_bump_seed = [ix.data.claim_bump];
    let claim_pda_seeds = claim_seeds.seeds_with_bump(&claim_bump_seed);
    let claim_pda_seeds_array: [_; 4] = claim_pda_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;

    let is_new_claim = is_pda_uninitialized(ix.accounts.claim_account);

    let mut claim = if is_new_claim {
        create_pda_account_idempotent(
            ix.accounts.payer,
            MerkleClaim::LEN,
            &ID,
            ix.accounts.claim_account,
            claim_pda_seeds_array,
        )?;

        let claim = MerkleClaim::new(ix.data.claim_bump);
        let mut claim_data = ix.accounts.claim_account.try_borrow_mut()?;
        claim.write_to_slice(&mut claim_data)?;
        drop(claim_data);
        claim
    } else {
        let claim_data = ix.accounts.claim_account.try_borrow()?;
        let claim = MerkleClaim::parse_from_bytes(&claim_data)?;
        drop(claim_data);
        claim
    };

    let claimable_amount = ix
        .data
        .cumulative_amount
        .checked_sub(claim.claimed_amount)
        .ok_or(RewardsProgramError::ClaimedAmountDecreased)?;
    let claim_amount = resolve_claim_amount(ix.data.amount, claimable_amount)?;

    ClaimTracker::add_claimed(&mut claim, claim_amount)?;

    pool.total_claimed = pool.validate_total_claim(claim_amount)?;

    let mut claim_data = ix.accounts.claim_account.try_borrow_mut()?;
    claim.write_to_slice(&mut claim_data)?;
    drop(claim_data);

    let mut pool_data = ix.accounts.reward_pool.try_borrow_mut()?;
    pool.write_to_slice(&mut pool_data)?;
    drop(pool_data);

    let decimals = get_mint_decimals(ix.accounts.reward_mint)?;

    pool.with_signer(|signers| {
        TransferChecked {
            from: ix.accounts.reward_vault,
            mint: ix.accounts.reward_mint,
            to: ix.accounts.user_reward_token_account,
            authority: ix.accounts.reward_pool,
            amount: claim_amount,
            decimals,
            token_program: ix.accounts.reward_token_program.address(),
        }
        .invoke_signed(signers)
    })?;

    let event = ClaimedEvent::new(*ix.accounts.reward_pool.address(), *ix.accounts.user.address(), claim_amount);
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
