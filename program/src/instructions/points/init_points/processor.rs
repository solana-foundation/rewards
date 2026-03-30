use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    events::PointsConfigCreatedEvent,
    state::PointsConfig,
    traits::{AccountSerialize, AccountSize, EventSerialize, PdaSeeds},
    utils::{cpi_initialize_points_mint, create_pda_account, emit_event},
    ID,
};

use super::InitPoints;

pub fn process_init_points(_program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    let ix = InitPoints::try_from((instruction_data, accounts))?;

    let config = PointsConfig::new(
        ix.data.bump,
        ix.data.transferable,
        ix.data.revocable,
        ix.data.mint_bump,
        *ix.accounts.authority.address(),
        *ix.accounts.seed.address(),
    );

    config.validate_pda(ix.accounts.points_config, &ID, ix.data.bump)?;

    let bump_seed = [ix.data.bump];
    let config_seeds = config.seeds_with_bump(&bump_seed);
    let config_seeds_array: [_; 4] =
        config_seeds.try_into().map_err(|_| pinocchio::error::ProgramError::InvalidArgument)?;

    create_pda_account(ix.accounts.payer, PointsConfig::LEN, &ID, ix.accounts.points_config, config_seeds_array)?;

    let mut config_data = ix.accounts.points_config.try_borrow_mut()?;
    config.write_to_slice(&mut config_data)?;
    drop(config_data);

    // Create the Token-2022 points mint with extensions
    cpi_initialize_points_mint(
        ix.accounts.payer,
        ix.accounts.points_mint,
        ix.accounts.points_config,
        ix.accounts.system_program,
        ix.accounts.token_2022_program,
        ix.data.mint_bump,
    )?;

    let event = PointsConfigCreatedEvent::new(
        *ix.accounts.authority.address(),
        *ix.accounts.seed.address(),
        ix.data.transferable,
        ix.data.revocable,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
