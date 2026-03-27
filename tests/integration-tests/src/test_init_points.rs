use solana_sdk::signer::Signer;

use crate::fixtures::{InitPointsFixture, InitPointsSetup};
use crate::utils::{
    test_empty_data, test_missing_signer, test_not_writable, test_truncated_data, test_wrong_current_program,
    test_wrong_system_program, TestContext,
};

// ── Generic validation tests ────────────────────────────────────────────────

#[test]
fn test_init_points_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<InitPointsFixture>(&mut ctx, 1, 0);
}

#[test]
fn test_init_points_missing_seeds_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<InitPointsFixture>(&mut ctx, 2, 1);
}

#[test]
fn test_init_points_config_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<InitPointsFixture>(&mut ctx, 3);
}

#[test]
fn test_init_points_wrong_system_program() {
    let mut ctx = TestContext::new();
    test_wrong_system_program::<InitPointsFixture>(&mut ctx);
}

#[test]
fn test_init_points_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<InitPointsFixture>(&mut ctx);
}

#[test]
fn test_init_points_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<InitPointsFixture>(&mut ctx);
}

#[test]
fn test_init_points_truncated_data() {
    let mut ctx = TestContext::new();
    test_truncated_data::<InitPointsFixture>(&mut ctx);
}

// ── Success tests ───────────────────────────────────────────────────────────

#[test]
fn test_init_points_success_default() {
    let mut ctx = TestContext::new();
    let setup = InitPointsSetup::new(&mut ctx);
    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    crate::utils::assert_points_config(
        &ctx,
        &setup.points_config_pda,
        &setup.authority.pubkey(),
        &setup.seed.pubkey(),
        setup.bump,
        0,
        0,
        0,
        0,
        0,
    );
}

#[test]
fn test_init_points_success_with_max_supply() {
    let mut ctx = TestContext::new();
    let setup = InitPointsSetup::builder(&mut ctx).max_supply(10_000).build();
    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    crate::utils::assert_points_config(
        &ctx,
        &setup.points_config_pda,
        &setup.authority.pubkey(),
        &setup.seed.pubkey(),
        setup.bump,
        0,
        0,
        10_000,
        0,
        0,
    );
}

#[test]
fn test_init_points_success_all_flags() {
    let mut ctx = TestContext::new();
    let setup = InitPointsSetup::builder(&mut ctx).transferable(1).revocable(1).max_supply(50_000).build();
    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    crate::utils::assert_points_config(
        &ctx,
        &setup.points_config_pda,
        &setup.authority.pubkey(),
        &setup.seed.pubkey(),
        setup.bump,
        1,
        1,
        50_000,
        0,
        0,
    );
}
