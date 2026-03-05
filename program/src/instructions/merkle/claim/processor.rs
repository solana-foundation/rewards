use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};
use pinocchio_token_2022::instructions::TransferChecked;

use crate::{
    errors::RewardsProgramError,
    events::ClaimedEvent,
    state::{MerkleClaim, MerkleClaimSeeds, MerkleDistribution},
    traits::{
        AccountParse, AccountSerialize, AccountSize, ClaimTracker, Distribution, DistributionSigner, EventSerialize,
        PdaSeeds, VestingParams,
    },
    utils::{
        compute_leaf_hash, create_pda_account_idempotent, emit_event, get_current_timestamp, get_mint_decimals,
        is_pda_uninitialized, resolve_claim_amount, verify_not_revoked, verify_proof_or_error,
    },
    ID,
};

use super::ClaimMerkle;

pub fn process_claim_merkle(_program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    let ix = ClaimMerkle::try_from((instruction_data, accounts))?;

    let current_ts = get_current_timestamp()?;

    let distribution_data = ix.accounts.distribution.try_borrow()?;
    let mut distribution = MerkleDistribution::from_account(&distribution_data, ix.accounts.distribution, &ID)?;
    drop(distribution_data);

    let schedule_bytes = ix.data.schedule.to_bytes();
    let leaf = compute_leaf_hash(ix.accounts.claimant.address(), ix.data.total_amount, &schedule_bytes);
    verify_proof_or_error(&ix.data.proof, &distribution.merkle_root, &leaf)?;

    verify_not_revoked(
        ix.accounts.distribution.address(),
        ix.accounts.claimant.address(),
        ix.accounts.revocation_marker,
        &ID,
        RewardsProgramError::ClaimantAlreadyRevoked,
    )?;

    let claim_seeds = MerkleClaimSeeds {
        distribution: *ix.accounts.distribution.address(),
        claimant: *ix.accounts.claimant.address(),
    };

    claim_seeds.validate_pda(ix.accounts.claim_account, &ID, ix.data.claim_bump)?;

    let claim_bump_seed = [ix.data.claim_bump];
    let claim_pda_seeds = claim_seeds.seeds_with_bump(&claim_bump_seed);
    let claim_pda_seeds_array: [_; 4] = claim_pda_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;

    // Unlike direct distributions where the authority creates recipient accounts upfront,
    // merkle claim accounts are created on the claimant's first claim. Check if this PDA
    // is uninitialized (owned by the system program) to determine if it needs creation.
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

    // Calculate how much the claimant can claim right now:
    // 1. calculate_unlocked: total tokens unlocked by the vesting schedule at current_ts
    // 2. claimable_amount: unlocked minus already claimed (via ClaimTracker)
    // 3. resolve_claim_amount: if amount == 0 claim everything available, else validate request
    let unlocked_amount = VestingParams::calculate_unlocked(&ix.data, current_ts)?;
    let claimable_amount = ClaimTracker::claimable_amount(&claim, unlocked_amount)?;
    let claim_amount = resolve_claim_amount(ix.data.amount, claimable_amount)?;

    ClaimTracker::add_claimed(&mut claim, claim_amount)?;
    Distribution::add_claimed(&mut distribution, claim_amount)?;

    let mut claim_data = ix.accounts.claim_account.try_borrow_mut()?;
    claim.write_to_slice(&mut claim_data)?;
    drop(claim_data);

    let mut distribution_data = ix.accounts.distribution.try_borrow_mut()?;
    distribution.write_to_slice(&mut distribution_data)?;
    drop(distribution_data);

    let decimals = get_mint_decimals(ix.accounts.mint)?;

    distribution.with_signer(|signers| {
        TransferChecked {
            from: ix.accounts.distribution_vault,
            mint: ix.accounts.mint,
            to: ix.accounts.claimant_token_account,
            authority: ix.accounts.distribution,
            amount: claim_amount,
            decimals,
            token_program: ix.accounts.token_program.address(),
        }
        .invoke_signed(signers)
    })?;

    let event = ClaimedEvent::new(*ix.accounts.distribution.address(), *ix.accounts.claimant.address(), claim_amount);
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
