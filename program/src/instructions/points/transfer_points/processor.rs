use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    errors::RewardsProgramError,
    events::PointsTransferredEvent,
    state::PointsConfig,
    traits::{EventSerialize, InstructionData},
    utils::{
        cpi_burn_points, cpi_create_ata_idempotent, cpi_mint_points, emit_event, get_token_account_balance,
        validate_associated_token_account_address,
    },
    ID,
};

use super::TransferPoints;

pub fn process_transfer_points(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = TransferPoints::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    // Parse and validate config
    let config_data = ix.accounts.points_config.try_borrow()?;
    let config = PointsConfig::from_account(&config_data, ix.accounts.points_config, &ID)?;
    drop(config_data);

    config.validate_authority(ix.accounts.authority.address())?;
    config.validate_transferable()?;

    // Prevent self-transfers
    if ix.accounts.from_user.address() == ix.accounts.to_user.address() {
        return Err(RewardsProgramError::PointsSelfTransferNotAllowed.into());
    }

    // Validate from token account is the correct ATA
    validate_associated_token_account_address(
        ix.accounts.from_token_account,
        ix.accounts.from_user.address(),
        ix.accounts.points_mint,
        ix.accounts.token_2022_program,
    )?;

    // Validate to token account is the correct ATA
    validate_associated_token_account_address(
        ix.accounts.to_token_account,
        ix.accounts.to_user.address(),
        ix.accounts.points_mint,
        ix.accounts.token_2022_program,
    )?;

    // Validate from balance
    let from_balance = get_token_account_balance(ix.accounts.from_token_account)?;
    if from_balance < ix.data.quantity {
        return Err(RewardsProgramError::InsufficientPointsBalance.into());
    }

    // Create destination ATA if needed
    cpi_create_ata_idempotent(
        ix.accounts.payer,
        ix.accounts.to_user,
        ix.accounts.points_mint,
        ix.accounts.to_token_account,
        ix.accounts.system_program,
        ix.accounts.token_2022_program,
    )?;

    // Transfer via burn + mint (NonTransferable blocks standard transfers)
    cpi_burn_points(
        &config,
        ix.accounts.from_token_account,
        ix.accounts.points_mint,
        ix.accounts.points_config,
        ix.data.quantity,
        ix.accounts.token_2022_program.address(),
    )?;

    cpi_mint_points(
        &config,
        ix.accounts.points_mint,
        ix.accounts.to_token_account,
        ix.accounts.points_config,
        ix.data.quantity,
        ix.accounts.token_2022_program.address(),
    )?;

    let event = PointsTransferredEvent::new(
        *ix.accounts.points_config.address(),
        config.authority,
        config.seed,
        config.transferable,
        config.revocable,
        *ix.accounts.from_user.address(),
        *ix.accounts.to_user.address(),
        ix.data.quantity,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
