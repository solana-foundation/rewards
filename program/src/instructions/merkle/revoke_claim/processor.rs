use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};
use pinocchio_token_2022::instructions::TransferChecked;

use crate::{
    errors::RewardsProgramError,
    events::RecipientRevokedEvent,
    state::{MerkleClaim, MerkleClaimSeeds, MerkleDistribution, Revocation, RevocationSeeds},
    traits::{
        AccountParse, AccountSerialize, AccountSize, Distribution, DistributionSigner, EventSerialize, InstructionData,
        PdaSeeds, VestingParams,
    },
    utils::{
        compute_leaf_hash, create_pda_account, emit_event, get_current_timestamp, get_mint_decimals,
        is_pda_uninitialized, verify_proof_or_error, RevokeMode,
    },
    ID,
};

use super::RevokeMerkleClaim;

pub fn process_revoke_merkle_claim(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = RevokeMerkleClaim::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    let current_ts = get_current_timestamp()?;

    // Load distribution and validate authority
    let distribution_data = ix.accounts.distribution.try_borrow()?;
    let mut distribution = MerkleDistribution::from_account(&distribution_data, ix.accounts.distribution, &ID)?;
    drop(distribution_data);

    distribution.validate_authority(ix.accounts.authority.address())?;

    if ix.data.revoke_mode.is_disabled_by(distribution.revocable) {
        return Err(RewardsProgramError::DistributionNotRevocable.into());
    }

    // Verify merkle proof: the authority provides the claimant's leaf data
    let schedule_bytes = ix.data.schedule.to_bytes();
    let leaf = compute_leaf_hash(ix.accounts.claimant.address(), ix.data.total_amount, &schedule_bytes);
    verify_proof_or_error(&ix.data.proof, &distribution.merkle_root, &leaf)?;

    // Validate revocation PDA and derive bump on-chain
    let revocation_seeds =
        RevocationSeeds { parent: *ix.accounts.distribution.address(), user: *ix.accounts.claimant.address() };
    let revocation_bump = revocation_seeds.validate_pda_address(ix.accounts.revocation_marker, &ID)?;

    if !is_pda_uninitialized(ix.accounts.revocation_marker) {
        return Err(RewardsProgramError::ClaimantAlreadyRevoked.into());
    }

    // Validate claim PDA and read claimed_amount (if the claimant already claimed)
    let claim_seeds = MerkleClaimSeeds {
        distribution: *ix.accounts.distribution.address(),
        claimant: *ix.accounts.claimant.address(),
    };
    claim_seeds.validate_pda_address(ix.accounts.claim_account, &ID)?;

    let claimed_amount = if is_pda_uninitialized(ix.accounts.claim_account) {
        0u64
    } else {
        let claim_data = ix.accounts.claim_account.try_borrow()?;
        let claim = MerkleClaim::parse_from_bytes(&claim_data)?;
        drop(claim_data);
        claim.claimed_amount
    };

    // Calculate vesting
    let vested_amount = VestingParams::calculate_unlocked(&ix.data, current_ts)?;
    let vested_unclaimed = vested_amount.checked_sub(claimed_amount).ok_or(RewardsProgramError::MathOverflow)?;
    let unvested = ix.data.total_amount.checked_sub(vested_amount).ok_or(RewardsProgramError::MathOverflow)?;

    // Apply revoke mode
    let decimals = get_mint_decimals(ix.accounts.mint)?;

    let (vested_transferred, total_freed) = match ix.data.revoke_mode {
        RevokeMode::NonVested => {
            if vested_unclaimed > 0 {
                distribution.with_signer(|signers| {
                    TransferChecked {
                        from: ix.accounts.distribution_vault,
                        mint: ix.accounts.mint,
                        to: ix.accounts.claimant_token_account,
                        authority: ix.accounts.distribution,
                        amount: vested_unclaimed,
                        decimals,
                        token_program: ix.accounts.token_program.address(),
                    }
                    .invoke_signed(signers)
                })?;
            }

            Distribution::add_claimed(&mut distribution, vested_unclaimed)?;

            (vested_unclaimed, unvested)
        }
        RevokeMode::Full => {
            let total_freed = unvested.checked_add(vested_unclaimed).ok_or(RewardsProgramError::MathOverflow)?;
            (0, total_freed)
        }
    };

    if total_freed > 0 {
        distribution.with_signer(|signers| {
            TransferChecked {
                from: ix.accounts.distribution_vault,
                mint: ix.accounts.mint,
                to: ix.accounts.authority_token_account,
                authority: ix.accounts.distribution,
                amount: total_freed,
                decimals,
                token_program: ix.accounts.token_program.address(),
            }
            .invoke_signed(signers)
        })?;
    }

    // Write updated distribution
    let mut distribution_data = ix.accounts.distribution.try_borrow_mut()?;
    distribution.write_to_slice(&mut distribution_data)?;
    drop(distribution_data);

    // Create revocation PDA
    let revocation_bump_seed = [revocation_bump];
    let revocation_pda_seeds = revocation_seeds.seeds_with_bump(&revocation_bump_seed);
    let revocation_pda_seeds_array: [_; 4] =
        revocation_pda_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;

    create_pda_account(
        ix.accounts.payer,
        Revocation::LEN,
        &ID,
        ix.accounts.revocation_marker,
        revocation_pda_seeds_array,
    )?;

    let revocation = Revocation::new(revocation_bump);
    let mut revocation_data = ix.accounts.revocation_marker.try_borrow_mut()?;
    revocation.write_to_slice(&mut revocation_data)?;
    drop(revocation_data);

    // Emit event
    let event = RecipientRevokedEvent::new(
        *ix.accounts.distribution.address(),
        *ix.accounts.claimant.address(),
        ix.data.revoke_mode,
        vested_transferred,
        total_freed,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
