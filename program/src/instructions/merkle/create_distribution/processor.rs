use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};
use pinocchio_associated_token_account::instructions::CreateIdempotent;
use pinocchio_token_2022::instructions::TransferChecked;

use crate::{
    errors::RewardsProgramError,
    events::DistributionCreatedEvent,
    state::MerkleDistribution,
    traits::{AccountSerialize, AccountSize, EventSerialize, InstructionData, PdaSeeds},
    utils::{create_pda_account, emit_event, get_mint_decimals, get_token_account_balance},
    ID,
};

use super::CreateMerkleDistribution;

pub fn process_create_merkle_distribution(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = CreateMerkleDistribution::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    // Create distribution with placeholder amount for PDA validation (amount isn't in seeds)
    let mut distribution = MerkleDistribution::new(
        ix.data.bump,
        ix.data.revocable,
        *ix.accounts.authority.address(),
        *ix.accounts.mint.address(),
        *ix.accounts.seed.address(),
        ix.data.merkle_root,
        ix.data.total_amount,
        ix.data.clawback_ts,
    );

    distribution.validate_pda(ix.accounts.distribution, &ID, ix.data.bump)?;

    if MerkleDistribution::is_closed(ix.accounts.distribution, &ID)? {
        return Err(RewardsProgramError::DistributionPermanentlyClosed.into());
    }

    let bump_seed = [ix.data.bump];
    let distribution_seeds = distribution.seeds_with_bump(&bump_seed);
    let distribution_seeds_array: [_; 5] = distribution_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;

    create_pda_account(
        ix.accounts.payer,
        MerkleDistribution::LEN,
        &ID,
        ix.accounts.distribution,
        distribution_seeds_array,
    )?;

    CreateIdempotent {
        funding_account: ix.accounts.payer,
        account: ix.accounts.distribution_vault,
        wallet: ix.accounts.distribution,
        mint: ix.accounts.mint,
        system_program: ix.accounts.system_program,
        token_program: ix.accounts.token_program,
    }
    .invoke()?;

    // Transfer tokens and measure actual received amount (accounts for transfer fees)
    let pre_balance = get_token_account_balance(ix.accounts.distribution_vault)?;
    let decimals = get_mint_decimals(ix.accounts.mint)?;

    TransferChecked {
        from: ix.accounts.authority_token_account,
        mint: ix.accounts.mint,
        to: ix.accounts.distribution_vault,
        authority: ix.accounts.authority,
        amount: ix.data.amount,
        decimals,
        token_program: ix.accounts.token_program.address(),
    }
    .invoke()?;

    let post_balance = get_token_account_balance(ix.accounts.distribution_vault)?;
    let actual_amount = post_balance.checked_sub(pre_balance).ok_or(RewardsProgramError::MathOverflow)?;

    if actual_amount == 0 {
        return Err(RewardsProgramError::InvalidAmount.into());
    }

    // Record actual received amount in distribution state
    distribution.total_amount = actual_amount;

    let mut distribution_data = ix.accounts.distribution.try_borrow_mut()?;
    distribution.write_to_slice(&mut distribution_data)?;
    drop(distribution_data);

    let event = DistributionCreatedEvent::merkle(
        *ix.accounts.authority.address(),
        *ix.accounts.mint.address(),
        *ix.accounts.seed.address(),
        ix.data.merkle_root,
        actual_amount,
        ix.data.clawback_ts,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
