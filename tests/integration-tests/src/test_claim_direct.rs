use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
};

use rewards_program_client::types::VestingSchedule;

use crate::fixtures::{ClaimDirectFixture, ClaimDirectSetup};
use crate::utils::{
    assert_direct_recipient, assert_instruction_error, assert_rewards_error, expected_linear_unlock, test_empty_data,
    test_missing_signer, test_not_writable, test_wrong_current_program, RewardsError, TestContext,
};

#[test]
fn test_claim_direct_missing_recipient_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<ClaimDirectFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_claim_direct_distribution_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimDirectFixture>(&mut ctx, 1);
}

#[test]
fn test_claim_direct_recipient_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimDirectFixture>(&mut ctx, 2);
}

#[test]
fn test_claim_direct_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimDirectFixture>(&mut ctx, 4);
}

#[test]
fn test_claim_direct_recipient_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimDirectFixture>(&mut ctx, 5);
}

#[test]
fn test_claim_direct_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<ClaimDirectFixture>(&mut ctx);
}

#[test]
fn test_claim_direct_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<ClaimDirectFixture>(&mut ctx);
}

#[test]
fn test_claim_direct_success_full() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::new(&mut ctx);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.recipient_token_account);
    assert_eq!(balance, setup.amount);

    assert_direct_recipient(
        &ctx,
        &setup.recipient_pda,
        &setup.recipient.pubkey(),
        setup.amount,
        setup.amount,
        setup.recipient_bump,
    );
}

#[test]
fn test_claim_direct_success_token_2022() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::new_token_2022(&mut ctx);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.recipient_token_account);
    assert_eq!(balance, setup.amount);
}

#[test]
fn test_claim_direct_partial_25_percent() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::builder(&mut ctx).warp_to_end(false).build();

    let duration = setup.end_ts - setup.start_ts;
    let timestamp_25_percent = setup.start_ts + (duration / 4);
    ctx.warp_to_timestamp(timestamp_25_percent);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.recipient_token_account);
    let expected = expected_linear_unlock(setup.amount, setup.start_ts, setup.end_ts, timestamp_25_percent);
    assert_eq!(balance, expected, "Balance should match exact linear unlock at 25%");
}

#[test]
fn test_claim_direct_partial_50_percent() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::builder(&mut ctx).warp_to_end(false).build();

    let duration = setup.end_ts - setup.start_ts;
    let timestamp_50_percent = setup.start_ts + (duration / 2);
    ctx.warp_to_timestamp(timestamp_50_percent);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.recipient_token_account);
    let expected = expected_linear_unlock(setup.amount, setup.start_ts, setup.end_ts, timestamp_50_percent);
    assert_eq!(balance, expected, "Balance should match exact linear unlock at 50%");
}

#[test]
fn test_claim_direct_multiple_claims() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::builder(&mut ctx).warp_to_end(false).build();

    let duration = setup.end_ts - setup.start_ts;

    let timestamp_25_percent = setup.start_ts + (duration / 4);
    ctx.warp_to_timestamp(timestamp_25_percent);

    let test_ix1 = setup.build_instruction(&ctx);
    test_ix1.send_expect_success(&mut ctx);

    let balance_after_first = ctx.get_token_balance(&setup.recipient_token_account);
    let expected_first = expected_linear_unlock(setup.amount, setup.start_ts, setup.end_ts, timestamp_25_percent);
    assert_eq!(balance_after_first, expected_first, "Balance should match exact linear unlock at 25%");

    let timestamp_50_percent = setup.start_ts + (duration / 2);
    ctx.warp_to_timestamp(timestamp_50_percent);

    let test_ix2 = setup.build_instruction(&ctx);
    test_ix2.send_expect_success(&mut ctx);

    let balance_after_second = ctx.get_token_balance(&setup.recipient_token_account);
    let expected_total = expected_linear_unlock(setup.amount, setup.start_ts, setup.end_ts, timestamp_50_percent);
    assert_eq!(balance_after_second, expected_total, "Balance should match exact linear unlock at 50%");
}

#[test]
fn test_claim_direct_nothing_before_start() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();

    let setup = ClaimDirectSetup::builder(&mut ctx)
        .schedule(VestingSchedule::Linear { start_ts: current_ts + 1000, end_ts: current_ts + 2000 })
        .warp_to_end(false)
        .build();

    let test_ix = setup.build_instruction(&ctx);
    let error = test_ix.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::NothingToClaim);
}

#[test]
fn test_claim_direct_nothing_already_claimed() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::new(&mut ctx);

    let test_ix1 = setup.build_instruction(&ctx);
    test_ix1.send_expect_success(&mut ctx);

    ctx.advance_slot();

    let test_ix2 = setup.build_instruction(&ctx);
    let error = test_ix2.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::NothingToClaim);
}

#[test]
fn test_claim_direct_unauthorized() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::new(&mut ctx);

    let wrong_signer = ctx.create_funded_keypair();
    let wrong_signer_token_account = ctx.create_token_account(&wrong_signer.pubkey(), &setup.mint);

    let test_ix = setup.build_instruction_with_wrong_signer(&ctx, &wrong_signer, wrong_signer_token_account);
    let error = test_ix.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::UnauthorizedRecipient);
}

#[test]
fn test_claim_direct_immediate_vesting() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::builder(&mut ctx).schedule(VestingSchedule::Immediate).warp_to_end(false).build();

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.recipient_token_account);
    assert_eq!(balance, setup.amount);

    assert_direct_recipient(
        &ctx,
        &setup.recipient_pda,
        &setup.recipient.pubkey(),
        setup.amount,
        setup.amount,
        setup.recipient_bump,
    );
}

#[test]
fn test_claim_direct_specific_amount() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::new(&mut ctx);

    let claim_amount = setup.amount / 3;
    let test_ix = setup.build_instruction_with_amount(claim_amount);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.recipient_token_account);
    assert_eq!(balance, claim_amount);

    assert_direct_recipient(
        &ctx,
        &setup.recipient_pda,
        &setup.recipient.pubkey(),
        setup.amount,
        claim_amount,
        setup.recipient_bump,
    );
}

#[test]
fn test_claim_direct_exceeds_claimable_amount() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::builder(&mut ctx).warp_to_end(false).build();

    // Warp to 50%
    let duration = setup.end_ts - setup.start_ts;
    let mid_point = setup.start_ts + (duration / 2);
    ctx.warp_to_timestamp(mid_point);

    // Request more than the 50% that's vested
    let test_ix = setup.build_instruction_with_amount(setup.amount);
    let error = test_ix.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::ExceedsClaimableAmount);
}

#[test]
fn test_claim_direct_rejects_mint_mismatch() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::new(&mut ctx);

    let wrong_mint = Keypair::new();
    ctx.create_mint_for_program(&wrong_mint, &ctx.payer.pubkey(), 6, &setup.token_program);

    let wrong_distribution_vault = ctx.create_ata_for_program_with_balance(
        &setup.distribution_pda,
        &wrong_mint.pubkey(),
        setup.amount,
        &setup.token_program,
    );
    let wrong_recipient_token_account =
        ctx.create_ata_for_program(&setup.recipient.pubkey(), &wrong_mint.pubkey(), &setup.token_program);

    let test_ix = setup
        .build_instruction(&ctx)
        .with_account_at(3, wrong_mint.pubkey())
        .with_account_at(4, wrong_distribution_vault)
        .with_account_at(5, wrong_recipient_token_account);

    let error = test_ix.send_expect_error(&mut ctx);
    assert_instruction_error(error, InstructionError::InvalidAccountData);
}
