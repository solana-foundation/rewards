use solana_sdk::instruction::InstructionError;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

use crate::fixtures::{UsePointsFixture, UsePointsSetup};
use crate::utils::{
    assert_instruction_error, assert_rewards_error, assert_user_points_balance, test_empty_data, test_missing_signer,
    test_not_writable, test_truncated_data, test_wrong_current_program, RewardsError, TestContext,
};

// ── Generic validation tests ────────────────────────────────────────────────

#[test]
fn test_use_points_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<UsePointsFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_use_points_missing_user_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<UsePointsFixture>(&mut ctx, 1, 1);
}

#[test]
fn test_use_points_points_mint_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<UsePointsFixture>(&mut ctx, 3);
}

#[test]
fn test_use_points_user_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<UsePointsFixture>(&mut ctx, 4);
}

#[test]
fn test_use_points_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<UsePointsFixture>(&mut ctx);
}

#[test]
fn test_use_points_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<UsePointsFixture>(&mut ctx);
}

#[test]
fn test_use_points_truncated_data() {
    let mut ctx = TestContext::new();
    test_truncated_data::<UsePointsFixture>(&mut ctx);
}

// ── Success tests ───────────────────────────────────────────────────────────

#[test]
fn test_use_points_success_partial() {
    let mut ctx = TestContext::new();
    let setup = UsePointsSetup::builder(&mut ctx).issue_quantity(1_000).quantity(400).build();
    let ix = setup.build_instruction(&ctx);
    ix.send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &setup.user.pubkey(), &setup.points_mint_pda, 600);
}

#[test]
fn test_use_points_success_full_balance() {
    let mut ctx = TestContext::new();
    let setup = UsePointsSetup::builder(&mut ctx).issue_quantity(500).quantity(500).build();
    let ix = setup.build_instruction(&ctx);
    ix.send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &setup.user.pubkey(), &setup.points_mint_pda, 0);
}

// ── Error tests ─────────────────────────────────────────────────────────────

#[test]
fn test_use_points_zero_quantity() {
    let mut ctx = TestContext::new();
    let mut setup = UsePointsSetup::builder(&mut ctx).issue_quantity(1_000).quantity(100).build();
    setup.quantity = 0;
    let ix = setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InvalidAmount);
}

#[test]
fn test_use_points_insufficient_balance() {
    let mut ctx = TestContext::new();
    let setup = UsePointsSetup::builder(&mut ctx).issue_quantity(100).quantity(200).build();
    let ix = setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InsufficientPointsBalance);
}

#[test]
fn test_use_points_wrong_authority() {
    let mut ctx = TestContext::new();
    let setup = UsePointsSetup::new(&mut ctx);

    let fake_authority = ctx.create_funded_keypair();
    let bad_setup = UsePointsSetup {
        authority: fake_authority,
        user: setup.user,
        points_config_pda: setup.points_config_pda,
        points_mint_pda: setup.points_mint_pda,
        user_ata: setup.user_ata,
        quantity: setup.quantity,
        issued_quantity: setup.issued_quantity,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_use_points_wrong_ata() {
    let mut ctx = TestContext::new();
    let setup = UsePointsSetup::new(&mut ctx);

    // Use a fabricated ATA address
    let wrong_ata = Keypair::new().pubkey();
    let bad_setup = UsePointsSetup {
        authority: setup.authority,
        user: setup.user,
        points_config_pda: setup.points_config_pda,
        points_mint_pda: setup.points_mint_pda,
        user_ata: wrong_ata,
        quantity: setup.quantity,
        issued_quantity: setup.issued_quantity,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_instruction_error(error, InstructionError::InvalidAccountData);
}
