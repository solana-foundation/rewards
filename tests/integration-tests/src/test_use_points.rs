use solana_sdk::{instruction::InstructionError, signer::Signer};

use crate::fixtures::{UsePointsFixture, UsePointsSetup};
use crate::utils::{
    assert_instruction_error, assert_rewards_error, assert_user_points_balance, find_user_points_pda,
    get_points_config, test_empty_data, test_missing_signer, test_not_writable, test_truncated_data,
    test_wrong_current_program, RewardsError, TestContext,
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
fn test_use_points_config_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<UsePointsFixture>(&mut ctx, 2);
}

#[test]
fn test_use_points_user_points_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<UsePointsFixture>(&mut ctx, 3);
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

    assert_user_points_balance(&ctx, &setup.user_points_pda, 600);

    let config = get_points_config(&ctx, &setup.points_config_pda);
    assert_eq!(config.total_used, 400);
    assert_eq!(config.total_issued, 1_000);
}

#[test]
fn test_use_points_success_full_balance() {
    let mut ctx = TestContext::new();
    let setup = UsePointsSetup::builder(&mut ctx).issue_quantity(500).quantity(500).build();
    let ix = setup.build_instruction(&ctx);
    ix.send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &setup.user_points_pda, 0);

    let config = get_points_config(&ctx, &setup.points_config_pda);
    assert_eq!(config.total_used, 500);
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
        user_points_pda: setup.user_points_pda,
        quantity: setup.quantity,
        issued_quantity: setup.issued_quantity,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_use_points_wrong_user_pda_mismatch() {
    let mut ctx = TestContext::new();
    // Setup creates config + issues to user
    let setup = UsePointsSetup::new(&mut ctx);

    // Create a different user and derive their PDA
    let other_user = ctx.create_funded_keypair();
    let (other_user_pda, _) = find_user_points_pda(&setup.points_config_pda, &other_user.pubkey());

    // Use the real user as signer but pass the other user's PDA
    let bad_setup = UsePointsSetup {
        authority: setup.authority,
        user: setup.user, // correct signer
        points_config_pda: setup.points_config_pda,
        user_points_pda: other_user_pda, // wrong PDA — doesn't derive from this user
        quantity: setup.quantity,
        issued_quantity: setup.issued_quantity,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    // PDA doesn't exist on-chain, so ownership check fires before PDA derivation
    assert_instruction_error(error, InstructionError::InvalidAccountOwner);
}
