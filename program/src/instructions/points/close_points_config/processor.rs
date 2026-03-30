use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    events::PointsConfigClosedEvent,
    state::{PointsConfig, PointsMintSeeds},
    traits::{EventSerialize, PdaSeeds},
    utils::{close_pda_account, emit_event, Points},
    ID,
};

use super::ClosePointsConfig;

pub fn process_close_points_config(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = ClosePointsConfig::try_from((instruction_data, accounts))?;

    let config_data = ix.accounts.points_config.try_borrow()?;
    let config = PointsConfig::from_account(&config_data, ix.accounts.points_config, &ID)?;
    drop(config_data);

    config.validate_authority(ix.accounts.authority.address())?;

    // Validate points mint PDA
    let mint_seeds = PointsMintSeeds { points_config: *ix.accounts.points_config.address() };
    mint_seeds.validate_pda(ix.accounts.points_mint, &ID, config.mint_bump)?;

    // Close the Token-2022 mint first (requires supply == 0, enforced by Token-2022)
    Points::close_points_mint(
        &config,
        ix.accounts.points_mint,
        ix.accounts.destination,
        ix.accounts.points_config,
        ix.accounts.token_2022_program.address(),
    )?;

    // Close the config PDA, reclaim rent to destination
    close_pda_account(ix.accounts.points_config, ix.accounts.destination)?;

    let event = PointsConfigClosedEvent::new(
        *ix.accounts.points_config.address(),
        config.authority,
        config.seed,
        config.transferable,
        config.revocable,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
