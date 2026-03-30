use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use crate::fixtures::{ClosePointsConfigFixture, ClosePointsConfigSetup, InitPointsSetup, IssuePointsSetup};
use crate::utils::{
    assert_account_closed, assert_rewards_error, test_empty_data, test_missing_signer, test_not_writable,
    test_wrong_current_program, RewardsError, TestContext,
};

// ── Generic validation tests ────────────────────────────────────────────────

#[test]
fn test_close_points_config_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<ClosePointsConfigFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_close_points_config_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClosePointsConfigFixture>(&mut ctx, 1);
}

#[test]
fn test_close_points_config_destination_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClosePointsConfigFixture>(&mut ctx, 3);
}

#[test]
fn test_close_points_config_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<ClosePointsConfigFixture>(&mut ctx);
}

#[test]
fn test_close_points_config_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<ClosePointsConfigFixture>(&mut ctx);
}

// ── Success tests ───────────────────────────────────────────────────────────

#[test]
fn test_close_points_config_success() {
    let mut ctx = TestContext::new();
    let setup = ClosePointsConfigSetup::new(&mut ctx);
    let ix = setup.build_instruction(&ctx);
    ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.points_config_pda);
}

// ── Error tests ─────────────────────────────────────────────────────────────

#[test]
fn test_close_points_config_wrong_authority() {
    let mut ctx = TestContext::new();
    let setup = ClosePointsConfigSetup::new(&mut ctx);

    let fake_authority = ctx.create_funded_keypair();
    let bad_setup = ClosePointsConfigSetup {
        authority: fake_authority,
        points_config_pda: setup.points_config_pda,
        points_mint_pda: setup.points_mint_pda,
        destination: setup.destination,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_close_points_config_with_outstanding_supply() {
    let mut ctx = TestContext::new();

    // Create config and issue points (creating non-zero supply)
    let init_setup = InitPointsSetup::builder(&mut ctx).build();
    init_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let user = Keypair::new();
    let user_ata = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &init_setup.points_mint_pda,
        &TOKEN_2022_PROGRAM_ID,
    );

    let issue = IssuePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        user: user.pubkey(),
        user_ata,
        quantity: 500,
    };
    issue.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Attempt to close config while supply > 0 — Token-2022 should reject
    let destination = ctx.create_funded_keypair();
    let close_setup = ClosePointsConfigSetup {
        authority: init_setup.authority,
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        destination: destination.pubkey(),
    };
    let ix = close_setup.build_instruction(&ctx);
    let _error = ix.send_expect_error(&mut ctx);
}
