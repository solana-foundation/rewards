use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    errors::RewardsProgramError,
    events::PointsAccountClosedEvent,
    state::PointsConfig,
    traits::EventSerialize,
    utils::{emit_event, get_token_account_balance, validate_associated_token_account_address},
    ID,
};

use super::ClosePointsAccount;

pub fn process_close_points_account(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = ClosePointsAccount::try_from((instruction_data, accounts))?;

    // Parse and validate config
    let config_data = ix.accounts.points_config.try_borrow()?;
    let config = PointsConfig::from_account(&config_data, ix.accounts.points_config, &ID)?;
    drop(config_data);

    config.validate_authority(ix.accounts.authority.address())?;

    // Validate user token account is the correct ATA
    validate_associated_token_account_address(
        ix.accounts.user_token_account,
        ix.accounts.user.address(),
        ix.accounts.points_mint,
        ix.accounts.token_2022_program,
    )?;

    // Require zero balance — the user can close their own ATA via
    // standard Token-2022 CloseAccount after this verification.
    let balance = get_token_account_balance(ix.accounts.user_token_account)?;
    if balance != 0 {
        return Err(RewardsProgramError::PointsBalanceNotZero.into());
    }

    let event = PointsAccountClosedEvent::new(
        *ix.accounts.points_config.address(),
        config.authority,
        config.seed,
        config.transferable,
        config.revocable,
        *ix.accounts.user.address(),
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
