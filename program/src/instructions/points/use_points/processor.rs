use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    events::PointsUsedEvent,
    state::{PointsConfig, UserPointsAccount},
    traits::{AccountSerialize, EventSerialize, InstructionData},
    utils::emit_event,
    ID,
};

use super::UsePoints;

pub fn process_use_points(_program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    let ix = UsePoints::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    // Parse and validate config
    let config_data = ix.accounts.points_config.try_borrow()?;
    let mut config = PointsConfig::from_account(&config_data, ix.accounts.points_config, &ID)?;
    drop(config_data);

    config.validate_authority(ix.accounts.authority.address())?;

    // Parse and validate user points account
    let user_data = ix.accounts.user_points_account.try_borrow()?;
    let mut user_account = UserPointsAccount::from_account(
        &user_data,
        ix.accounts.user_points_account,
        &ID,
        ix.accounts.points_config.address(),
        ix.accounts.user.address(),
    )?;
    drop(user_data);

    // Validate balance
    user_account.validate_balance(ix.data.quantity)?;

    // Update balances
    user_account.sub_balance(ix.data.quantity)?;
    config.add_used(ix.data.quantity)?;

    // Write updated state
    let mut user_account_data = ix.accounts.user_points_account.try_borrow_mut()?;
    user_account.write_to_slice(&mut user_account_data)?;
    drop(user_account_data);

    let mut config_account_data = ix.accounts.points_config.try_borrow_mut()?;
    config.write_to_slice(&mut config_account_data)?;
    drop(config_account_data);

    let event = PointsUsedEvent::new(
        *ix.accounts.points_config.address(),
        config.authority,
        config.seed,
        config.max_supply,
        config.transferable,
        config.revocable,
        config.total_issued,
        config.total_used,
        *ix.accounts.user.address(),
        ix.data.quantity,
        user_account.balance,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
