use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    events::PointsIssuedEvent,
    state::PointsConfig,
    traits::{EventSerialize, InstructionData},
    utils::{
        cpi_create_ata_idempotent, cpi_mint_points, emit_event, get_token_account_balance,
        validate_associated_token_account_address,
    },
    ID,
};

use super::IssuePoints;

pub fn process_issue_points(_program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    let ix = IssuePoints::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

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

    // Create ATA idempotently (first issue creates the account)
    cpi_create_ata_idempotent(
        ix.accounts.payer,
        ix.accounts.user,
        ix.accounts.points_mint,
        ix.accounts.user_token_account,
        ix.accounts.system_program,
        ix.accounts.token_2022_program,
    )?;

    // Mint points to the user's token account
    cpi_mint_points(
        &config,
        ix.accounts.points_mint,
        ix.accounts.user_token_account,
        ix.accounts.points_config,
        ix.data.quantity,
        ix.accounts.token_2022_program.address(),
    )?;

    // Read new balance from ATA post-mint
    let new_balance = get_token_account_balance(ix.accounts.user_token_account)?;

    let event = PointsIssuedEvent::new(
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
