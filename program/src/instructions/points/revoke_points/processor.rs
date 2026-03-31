use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    errors::RewardsProgramError,
    events::PointsRevokedEvent,
    state::{PointsConfig, PointsMintSeeds},
    traits::{EventSerialize, PdaSeeds},
    utils::{emit_event, get_token_account_balance, Points},
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

    // Validate points mint PDA
    let mint_seeds = PointsMintSeeds { points_config: *ix.accounts.points_config.address() };
    mint_seeds.validate_pda(ix.accounts.points_mint, &ID, config.mint_bump)?;

    // Read balance — error if nothing to revoke
    let revoked_balance = get_token_account_balance(ix.accounts.user_token_account)?;
    if revoked_balance == 0 {
        return Err(RewardsProgramError::PointsNothingToRevoke.into());
    }

    // Burn entire balance via permanent delegate.
    // The token account stays open — the user can close their own ATA
    // via standard Token-2022 CloseAccount since PermanentDelegate only
    // authorizes Burn/Transfer, not CloseAccount.
    Points::burn(
        &config,
        ix.accounts.user_token_account,
        ix.accounts.points_mint,
        ix.accounts.points_config,
        revoked_balance,
        ix.accounts.token_2022_program.address(),
    )?;

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
