use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    events::PointsRevokedEvent,
    state::PointsConfig,
    traits::EventSerialize,
    utils::{cpi_burn_points, emit_event, get_token_account_balance, validate_associated_token_account_address},
    ID,
};

use super::RevokePoints;

pub fn process_revoke_points(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = RevokePoints::try_from((instruction_data, accounts))?;

    // Parse and validate config
    let config_data = ix.accounts.points_config.try_borrow()?;
    let config = PointsConfig::from_account(&config_data, ix.accounts.points_config, &ID)?;
    drop(config_data);

    config.validate_authority(ix.accounts.authority.address())?;
    config.validate_revocable()?;

    // Validate user token account is the correct ATA
    validate_associated_token_account_address(
        ix.accounts.user_token_account,
        ix.accounts.user.address(),
        ix.accounts.points_mint,
        ix.accounts.token_2022_program,
    )?;

    // Read balance before burning
    let revoked_balance = get_token_account_balance(ix.accounts.user_token_account)?;

    // Burn entire balance via permanent delegate.
    // The token account stays open — the user can close their own ATA
    // via standard Token-2022 CloseAccount since PermanentDelegate only
    // authorizes Burn/Transfer, not CloseAccount.
    if revoked_balance > 0 {
        cpi_burn_points(
            &config,
            ix.accounts.user_token_account,
            ix.accounts.points_mint,
            ix.accounts.points_config,
            revoked_balance,
            ix.accounts.token_2022_program.address(),
        )?;
    }

    let event = PointsRevokedEvent::new(
        *ix.accounts.points_config.address(),
        config.authority,
        config.seed,
        config.transferable,
        config.revocable,
        *ix.accounts.user.address(),
        revoked_balance,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
