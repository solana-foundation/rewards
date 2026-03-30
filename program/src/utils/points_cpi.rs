use pinocchio::{
    account::AccountView,
    cpi::{invoke, Seed},
    instruction::{InstructionAccount, InstructionView},
    Address, ProgramResult,
};
use pinocchio_associated_token_account::instructions::CreateIdempotent;
use pinocchio_token_2022::instructions::{Burn, CloseAccount, InitializeMint2, MintTo};

use crate::state::{PointsConfig, PointsMintSeeds, POINTS_MINT_SPACE};
use crate::traits::PdaSeeds;
use crate::utils::create_pda_account;

// Token-2022 instruction discriminators for extension init (from spl-token-2022)
const IX_INIT_MINT_CLOSE_AUTHORITY: u8 = 25;
const IX_INIT_NON_TRANSFERABLE_MINT: u8 = 32;
const IX_INIT_PERMANENT_DELEGATE: u8 = 35;

/// Initialize a Token-2022 points mint with NonTransferable + PermanentDelegate +
/// MintCloseAuthority extensions. The PointsConfig PDA is set as mint authority,
/// permanent delegate, and close authority.
///
/// Extension init instructions must be called before InitializeMint2 — this is
/// enforced by the Token-2022 program.
pub fn cpi_initialize_points_mint<'a>(
    payer: &'a AccountView,
    points_mint: &'a AccountView,
    config_account: &'a AccountView,
    _system_program: &'a AccountView,
    token_2022_program: &'a AccountView,
    mint_bump: u8,
) -> ProgramResult {
    let bump_seed = [mint_bump];
    let pda_seeds: [Seed; 3] = [
        Seed::from(PointsMintSeeds::PREFIX),
        Seed::from(config_account.address().as_ref()),
        Seed::from(bump_seed.as_slice()),
    ];

    // 1. Create the mint account with enough space for extensions
    create_pda_account(payer, POINTS_MINT_SPACE, token_2022_program.address(), points_mint, pda_seeds)?;

    let token_program = token_2022_program.address();
    let config_addr = config_account.address();

    // 2. InitializeNonTransferableMint — discriminator 32, no data
    {
        let data = [IX_INIT_NON_TRANSFERABLE_MINT];
        let accounts = [InstructionAccount::writable(points_mint.address())];
        let ix = InstructionView { program_id: token_program, accounts: &accounts, data: &data };
        invoke::<1>(&ix, &[points_mint])?;
    }

    // 3. InitializePermanentDelegate — discriminator 35, data = 32-byte pubkey
    {
        let mut data = [0u8; 33];
        data[0] = IX_INIT_PERMANENT_DELEGATE;
        data[1..33].copy_from_slice(config_addr.as_ref());
        let accounts = [InstructionAccount::writable(points_mint.address())];
        let ix = InstructionView { program_id: token_program, accounts: &accounts, data: &data };
        invoke::<1>(&ix, &[points_mint])?;
    }

    // 4. InitializeMintCloseAuthority — discriminator 25, data = COption<Pubkey>
    //    COption::Some = 1 byte (0x01) + 32 bytes pubkey
    {
        let mut data = [0u8; 34];
        data[0] = IX_INIT_MINT_CLOSE_AUTHORITY;
        data[1] = 1; // COption::Some
        data[2..34].copy_from_slice(config_addr.as_ref());
        let accounts = [InstructionAccount::writable(points_mint.address())];
        let ix = InstructionView { program_id: token_program, accounts: &accounts, data: &data };
        invoke::<1>(&ix, &[points_mint])?;
    }

    // 5. Initialize the mint itself (decimals=0 for whole-number points)
    InitializeMint2 {
        mint: points_mint,
        decimals: 0,
        mint_authority: config_addr,
        freeze_authority: None,
        token_program,
    }
    .invoke()?;

    Ok(())
}

/// Mint points to a user's token account. The PointsConfig PDA signs as mint authority.
#[inline(always)]
pub fn cpi_mint_points<'a>(
    config: &PointsConfig,
    mint: &'a AccountView,
    destination: &'a AccountView,
    config_account: &'a AccountView,
    amount: u64,
    token_program: &'a Address,
) -> ProgramResult {
    config.with_signer(|signers| {
        MintTo { mint, account: destination, mint_authority: config_account, amount, token_program }
            .invoke_signed(signers)
    })
}

/// Burn points from a user's token account. The PointsConfig PDA signs as permanent delegate.
#[inline(always)]
pub fn cpi_burn_points<'a>(
    config: &PointsConfig,
    token_account: &'a AccountView,
    mint: &'a AccountView,
    config_account: &'a AccountView,
    amount: u64,
    token_program: &'a Address,
) -> ProgramResult {
    config.with_signer(|signers| {
        Burn { account: token_account, mint, authority: config_account, amount, token_program }.invoke_signed(signers)
    })
}

/// Close a user's token account. The PointsConfig PDA signs as permanent delegate.
/// Rent lamports are returned to the destination account.
#[inline(always)]
pub fn cpi_close_token_account<'a>(
    config: &PointsConfig,
    token_account: &'a AccountView,
    destination: &'a AccountView,
    config_account: &'a AccountView,
    token_program: &'a Address,
) -> ProgramResult {
    config.with_signer(|signers| {
        CloseAccount { account: token_account, destination, authority: config_account, token_program }
            .invoke_signed(signers)
    })
}

/// Close the points mint account via MintCloseAuthority. The PointsConfig PDA signs
/// as close authority. Token-2022 enforces that mint supply must be 0.
#[inline(always)]
pub fn cpi_close_points_mint<'a>(
    config: &PointsConfig,
    mint: &'a AccountView,
    destination: &'a AccountView,
    config_account: &'a AccountView,
    token_program: &'a Address,
) -> ProgramResult {
    config.with_signer(|signers| {
        CloseAccount { account: mint, destination, authority: config_account, token_program }.invoke_signed(signers)
    })
}

/// Create an Associated Token Account idempotently for a user's points.
#[inline(always)]
pub fn cpi_create_ata_idempotent<'a>(
    payer: &'a AccountView,
    user: &'a AccountView,
    mint: &'a AccountView,
    ata: &'a AccountView,
    system_program: &'a AccountView,
    token_program: &'a AccountView,
) -> ProgramResult {
    CreateIdempotent { funding_account: payer, account: ata, wallet: user, mint, system_program, token_program }
        .invoke()
}
