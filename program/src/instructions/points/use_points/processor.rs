use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    errors::RewardsProgramError,
    events::PointsUsedEvent,
    state::{PointsConfig, PointsMintSeeds},
    traits::{EventSerialize, InstructionData, PdaSeeds},
    utils::{cpi_burn_points, emit_event, get_token_account_balance, validate_associated_token_account_address},
    ID,
};

use super::UsePoints;

pub fn process_use_points(_program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    let ix = UsePoints::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    // Parse and validate config
    let config_data = ix.accounts.points_config.try_borrow()?;
    let config = PointsConfig::from_account(&config_data, ix.accounts.points_config, &ID)?;
    drop(config_data);

    config.validate_authority(ix.accounts.authority.address())?;

    // Validate points mint PDA
    let mint_seeds = PointsMintSeeds { points_config: *ix.accounts.points_config.address() };
    mint_seeds.validate_pda(ix.accounts.points_mint, &ID, config.mint_bump)?;

    // Validate user token account is the correct ATA
    validate_associated_token_account_address(
        ix.accounts.user_token_account,
        ix.accounts.user.address(),
        ix.accounts.points_mint,
        ix.accounts.token_2022_program,
    )?;

    // Read and validate balance
    let balance = get_token_account_balance(ix.accounts.user_token_account)?;
    if balance < ix.data.quantity {
        return Err(RewardsProgramError::InsufficientPointsBalance.into());
    }

    // Burn points via permanent delegate
    cpi_burn_points(
        &config,
        ix.accounts.user_token_account,
        ix.accounts.points_mint,
        ix.accounts.points_config,
        ix.data.quantity,
        ix.accounts.token_2022_program.address(),
    )?;

    // Read new balance post-burn
    let new_balance = get_token_account_balance(ix.accounts.user_token_account)?;

    let event = PointsUsedEvent::new(
        *ix.accounts.points_config.address(),
        config.authority,
        config.seed,
        config.transferable,
        config.revocable,
        *ix.accounts.user.address(),
        ix.data.quantity,
        new_balance,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
