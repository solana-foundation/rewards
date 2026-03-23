use solana_sdk::instruction::InstructionError;
use solana_sdk::signature::Signer;

use rewards_program_client::types::VestingSchedule;

use crate::fixtures::{ClaimMerkleFixture, ClaimMerkleSetup};
use crate::utils::{
    assert_merkle_claim, assert_rewards_error, expected_linear_unlock, test_missing_signer, test_not_writable,
    test_wrong_account, test_wrong_current_program, test_wrong_system_program, RewardsError, TestContext,
};

#[test]
fn test_claim_merkle_missing_claimant_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<ClaimMerkleFixture>(&mut ctx, 1, 0);
}

#[test]
fn test_claim_merkle_distribution_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimMerkleFixture>(&mut ctx, 2);
}

#[test]
fn test_claim_merkle_claim_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimMerkleFixture>(&mut ctx, 3);
}

#[test]
fn test_claim_merkle_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimMerkleFixture>(&mut ctx, 6);
}

#[test]
fn test_claim_merkle_claimant_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimMerkleFixture>(&mut ctx, 7);
}

#[test]
fn test_claim_merkle_wrong_system_program() {
    let mut ctx = TestContext::new();
    test_wrong_system_program::<ClaimMerkleFixture>(&mut ctx);
}

#[test]
fn test_claim_merkle_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<ClaimMerkleFixture>(&mut ctx);
}

#[test]
fn test_claim_merkle_wrong_revocation_pda() {
    let mut ctx = TestContext::new();
    test_wrong_account::<ClaimMerkleFixture>(&mut ctx, 4, InstructionError::InvalidSeeds);
}

#[test]
fn test_claim_merkle_success() {
    let mut ctx = TestContext::new();
    let setup = ClaimMerkleSetup::new(&mut ctx);

    let balance_before = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(balance_before, 0);

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    let balance_after = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(balance_after, setup.total_amount);

    // Claim account persists after full claim (rent recovered via CloseMerkleClaim)
    assert_merkle_claim(&ctx, &setup.claim_pda, setup.total_amount, setup.claim_bump);
}

#[test]
fn test_claim_merkle_success_token_2022() {
    let mut ctx = TestContext::new();
    let setup = ClaimMerkleSetup::new_token_2022(&mut ctx);

    let balance_before = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(balance_before, 0);

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    let balance_after = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(balance_after, setup.total_amount);
}

#[test]
fn test_claim_merkle_invalid_proof() {
    let mut ctx = TestContext::new();
    let setup = ClaimMerkleSetup::new(&mut ctx);

    let wrong_proof = vec![[99u8; 32]];
    let instruction = setup.build_instruction_with_wrong_proof(&ctx, wrong_proof);
    let error = instruction.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::InvalidMerkleProof);
}

#[test]
fn test_claim_merkle_wrong_amount_in_proof() {
    let mut ctx = TestContext::new();
    let setup = ClaimMerkleSetup::new(&mut ctx);

    let instruction = setup.build_instruction_with_wrong_amount(&ctx, setup.total_amount + 1000);
    let error = instruction.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::InvalidMerkleProof);
}

#[test]
fn test_claim_merkle_wrong_claimant() {
    let mut ctx = TestContext::new();
    let setup = ClaimMerkleSetup::new(&mut ctx);

    let wrong_claimant = ctx.create_funded_keypair();
    let wrong_token_account = ctx.create_token_account(&wrong_claimant.pubkey(), &setup.mint);

    let instruction = setup.build_instruction_with_wrong_claimant(&ctx, &wrong_claimant, wrong_token_account);
    let error = instruction.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::InvalidMerkleProof);
}

#[test]
fn test_claim_merkle_partial_claim_linear() {
    let mut ctx = TestContext::new();
    let setup = ClaimMerkleSetup::builder(&mut ctx).linear().warp_to_end(false).build();

    // Warp to 50% through the vesting period
    let mid_point = setup.start_ts() + (setup.end_ts() - setup.start_ts()) / 2;
    ctx.warp_to_timestamp(mid_point);

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.claimant_token_account);
    let expected = expected_linear_unlock(setup.total_amount, setup.start_ts(), setup.end_ts(), mid_point);
    assert_eq!(balance, expected, "Balance should match exact linear unlock at 50%");

    // Claim account should still exist since not fully claimed
    assert_merkle_claim(&ctx, &setup.claim_pda, balance, setup.claim_bump);
}

#[test]
fn test_claim_merkle_partial_claim_then_full() {
    let mut ctx = TestContext::new();
    let setup = ClaimMerkleSetup::builder(&mut ctx).linear().warp_to_end(false).build();

    // First claim at 50%
    let mid_point = setup.start_ts() + (setup.end_ts() - setup.start_ts()) / 2;
    ctx.warp_to_timestamp(mid_point);

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    let first_balance = ctx.get_token_balance(&setup.claimant_token_account);
    let expected_first = expected_linear_unlock(setup.total_amount, setup.start_ts(), setup.end_ts(), mid_point);
    assert_eq!(first_balance, expected_first, "Balance should match exact linear unlock at 50%");

    // Now warp to end and claim rest
    ctx.warp_to_timestamp(setup.end_ts());

    let instruction2 = setup.build_instruction(&ctx);
    instruction2.send_expect_success(&mut ctx);

    let final_balance = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(final_balance, setup.total_amount);

    // Claim account persists (rent recovered via CloseMerkleClaim after distribution closes)
    assert_merkle_claim(&ctx, &setup.claim_pda, setup.total_amount, setup.claim_bump);
}

#[test]
fn test_claim_merkle_immediate_schedule() {
    let mut ctx = TestContext::new();
    let setup = ClaimMerkleSetup::builder(&mut ctx).immediate().warp_to_end(false).build();

    // Should be able to claim full amount immediately
    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(balance, setup.total_amount);
}

#[test]
fn test_claim_merkle_before_start() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();

    let setup = ClaimMerkleSetup::builder(&mut ctx)
        .schedule(VestingSchedule::Linear { start_ts: current_ts + 86400, end_ts: current_ts + 86400 * 2 })
        .warp_to_end(false)
        .build();

    // Claim should fail because nothing is unlocked yet
    let instruction = setup.build_instruction(&ctx);
    let error = instruction.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::NothingToClaim);
}

#[test]
fn test_claim_merkle_specific_amount() {
    let mut ctx = TestContext::new();
    let setup = ClaimMerkleSetup::new(&mut ctx);

    let claim_amount = setup.total_amount / 2;
    let instruction = setup.build_instruction_with_amount(&ctx, claim_amount);
    instruction.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(balance, claim_amount);

    // Claim account should still exist
    assert_merkle_claim(&ctx, &setup.claim_pda, claim_amount, setup.claim_bump);
}

#[test]
fn test_claim_merkle_amount_exceeds_available() {
    let mut ctx = TestContext::new();
    let setup = ClaimMerkleSetup::builder(&mut ctx).linear().warp_to_end(false).build();

    // Warp to 25% through vesting
    let quarter_point = setup.start_ts() + (setup.end_ts() - setup.start_ts()) / 4;
    ctx.warp_to_timestamp(quarter_point);

    // Try to claim full amount when only ~25% is unlocked
    let instruction = setup.build_instruction_with_amount(&ctx, setup.total_amount);
    let error = instruction.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::ExceedsClaimableAmount);
}

#[test]
fn test_claim_merkle_multiple_claimants_tree() {
    let mut ctx = TestContext::new();
    let setup = ClaimMerkleSetup::builder(&mut ctx).num_claimants(4).build();

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(balance, setup.total_amount);
}

#[test]
fn test_claim_merkle_idempotent_claim_creation() {
    let mut ctx = TestContext::new();
    let setup = ClaimMerkleSetup::builder(&mut ctx).linear().warp_to_end(false).build();

    // First claim at 50%
    let mid_point = setup.start_ts() + (setup.end_ts() - setup.start_ts()) / 2;
    ctx.warp_to_timestamp(mid_point);

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    let first_balance = ctx.get_token_balance(&setup.claimant_token_account);
    let expected = expected_linear_unlock(setup.total_amount, setup.start_ts(), setup.end_ts(), mid_point);
    assert_eq!(first_balance, expected, "Balance should match exact linear unlock at 50%");

    // Advance slot to get a fresh blockhash for the second transaction
    ctx.advance_slot();

    // Claim again at same timestamp - should fail since nothing new is unlocked
    let instruction2 = setup.build_instruction(&ctx);
    let error = instruction2.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::NothingToClaim);

    // Balance should remain unchanged
    let second_balance = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(second_balance, first_balance);
}

#[test]
fn test_claim_merkle_reclaim_after_full_claim_fails() {
    let mut ctx = TestContext::new();
    let setup = ClaimMerkleSetup::new(&mut ctx);

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(balance, setup.total_amount);

    ctx.advance_slot();

    // Second claim must fail — everything is already claimed
    let instruction2 = setup.build_instruction(&ctx);
    let error = instruction2.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::NothingToClaim);
}
