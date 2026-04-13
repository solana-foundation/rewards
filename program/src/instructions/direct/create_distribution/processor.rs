use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};
use pinocchio_associated_token_account::instructions::CreateIdempotent;

use crate::{
    errors::RewardsProgramError,
    events::DistributionCreatedEvent,
    state::{DirectDistribution, DirectDistributionTombstoneSeeds},
    traits::{AccountSerialize, AccountSize, EventSerialize, InstructionData, PdaSeeds},
    utils::{create_pda_account, emit_event, is_pda_uninitialized},
    ID,
};

use super::CreateDirectDistribution;

pub fn process_create_direct_distribution(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = CreateDirectDistribution::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    let distribution = DirectDistribution::new(
        ix.data.bump,
        ix.data.revocable,
        ix.data.clawback_ts,
        *ix.accounts.authority.address(),
        *ix.accounts.mint.address(),
        *ix.accounts.seed.address(),
    );

    distribution.validate_pda(ix.accounts.distribution, &ID, ix.data.bump)?;
    let tombstone_seeds = DirectDistributionTombstoneSeeds { distribution: *ix.accounts.distribution.address() };
    tombstone_seeds.validate_pda_address(ix.accounts.tombstone, &ID)?;
    if !is_pda_uninitialized(ix.accounts.tombstone) {
        return Err(RewardsProgramError::DistributionPermanentlyClosed.into());
    }

    let bump_seed = [ix.data.bump];
    let distribution_seeds = distribution.seeds_with_bump(&bump_seed);
    let distribution_seeds_array: [_; 5] = distribution_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;

    create_pda_account(
        ix.accounts.payer,
        DirectDistribution::LEN,
        &ID,
        ix.accounts.distribution,
        distribution_seeds_array,
    )?;

    let mut distribution_data = ix.accounts.distribution.try_borrow_mut()?;
    distribution.write_to_slice(&mut distribution_data)?;
    drop(distribution_data);

    CreateIdempotent {
        funding_account: ix.accounts.payer,
        account: ix.accounts.distribution_vault,
        wallet: ix.accounts.distribution,
        mint: ix.accounts.mint,
        system_program: ix.accounts.system_program,
        token_program: ix.accounts.token_program,
    }
    .invoke()?;

    let event = DistributionCreatedEvent::direct(
        *ix.accounts.authority.address(),
        *ix.accounts.mint.address(),
        *ix.accounts.seed.address(),
        ix.data.clawback_ts,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
