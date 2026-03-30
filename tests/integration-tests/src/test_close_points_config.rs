use crate::fixtures::{ClosePointsConfigFixture, ClosePointsConfigSetup};
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
