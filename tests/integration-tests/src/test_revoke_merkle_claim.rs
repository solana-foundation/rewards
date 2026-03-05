use rewards_program_client::accounts::MerkleDistribution;
use rewards_program_client::types::{RevokeMode, VestingSchedule};

use crate::fixtures::{RevokeMerkleClaimFixture, RevokeMerkleClaimSetup};
use crate::utils::{
    assert_rewards_error, expected_linear_unlock, test_empty_data, test_missing_signer, test_not_writable,
    test_wrong_current_program, RewardsError, TestContext, PROGRAM_ID,
};

// ── Generic fixture tests ──────────────────────────────────────────

#[test]
fn test_revoke_merkle_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<RevokeMerkleClaimFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_revoke_merkle_missing_payer_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<RevokeMerkleClaimFixture>(&mut ctx, 1, 1);
}

#[test]
fn test_revoke_merkle_distribution_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokeMerkleClaimFixture>(&mut ctx, 2);
}

#[test]
fn test_revoke_merkle_revocation_marker_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokeMerkleClaimFixture>(&mut ctx, 4);
}

#[test]
fn test_revoke_merkle_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokeMerkleClaimFixture>(&mut ctx, 7);
}

#[test]
fn test_revoke_merkle_claimant_token_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokeMerkleClaimFixture>(&mut ctx, 8);
}

#[test]
fn test_revoke_merkle_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<RevokeMerkleClaimFixture>(&mut ctx);
}

#[test]
fn test_revoke_merkle_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<RevokeMerkleClaimFixture>(&mut ctx);
}

// ── Error paths ────────────────────────────────────────────────────

#[test]
fn test_revoke_merkle_wrong_authority() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::new(&mut ctx);

    let wrong_authority = ctx.create_funded_keypair();
    let revoke_ix = setup.build_instruction_with_wrong_authority(&ctx, &wrong_authority, RevokeMode::NonVested);
    let error = revoke_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_revoke_merkle_invalid_proof() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::new(&mut ctx);

    let mut revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    // Corrupt the proof by modifying bytes in the instruction data
    // The proof is at the end of the data, so corrupt the last 32 bytes
    let data_len = revoke_ix.instruction.data.len();
    if data_len >= 32 {
        revoke_ix.instruction.data[data_len - 1] ^= 0xFF;
    }
    let error = revoke_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InvalidMerkleProof);
}

#[test]
fn test_revoke_merkle_invalid_mode() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::new(&mut ctx);

    let mut revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    // Patch revoke_mode byte to invalid value 2
    // Data layout: [discriminator(1), revoke_mode(1), ...]
    revoke_ix.instruction.data[1] = 2;
    let error = revoke_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InvalidRevokeMode);
}

#[test]
fn test_revoke_merkle_double_revoke() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::new(&mut ctx);

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);

    // Advance clock so LiteSVM doesn't reject as duplicate transaction
    ctx.warp_to_timestamp(setup.start_ts + 1);

    // Second revoke should fail
    let revoke_ix2 = setup.build_instruction(&ctx, RevokeMode::NonVested);
    let error = revoke_ix2.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::ClaimantAlreadyRevoked);
}

// ── Happy paths — claimant never claimed ──────────────────────────

#[test]
fn test_revoke_merkle_non_vested_before_vesting_start() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::new(&mut ctx);
    // Don't warp — we're at start_ts, so nothing is vested yet

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);

    let claimant_balance = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(claimant_balance, 0, "Claimant should receive nothing when nothing is vested");

    // Revocation PDA should exist
    let revocation_marker = ctx.get_account(&setup.revocation_pda).expect("Revocation PDA should exist");
    assert_eq!(revocation_marker.owner, PROGRAM_ID);
}

#[test]
fn test_revoke_merkle_non_vested_at_midpoint() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::new(&mut ctx);

    let midpoint = setup.start_ts + (setup.end_ts - setup.start_ts) / 2;
    ctx.warp_to_timestamp(midpoint);

    let vault_balance_before = ctx.get_token_balance(&setup.distribution_vault);
    let claimant_balance_before = ctx.get_token_balance(&setup.claimant_token_account);
    let authority_balance_before = ctx.get_token_balance(&setup.authority_token_account);

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);

    let expected_vested = expected_linear_unlock(setup.total_amount, setup.start_ts, setup.end_ts, midpoint);
    let expected_unvested = setup.total_amount - expected_vested;

    let claimant_balance_after = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(
        claimant_balance_after,
        claimant_balance_before + expected_vested,
        "Claimant should receive vested tokens"
    );

    let authority_balance_after = ctx.get_token_balance(&setup.authority_token_account);
    assert_eq!(
        authority_balance_after,
        authority_balance_before + expected_unvested,
        "Authority should receive unvested tokens"
    );

    let vault_balance_after = ctx.get_token_balance(&setup.distribution_vault);
    assert_eq!(
        vault_balance_after,
        vault_balance_before - expected_vested - expected_unvested,
        "Vault should decrease by vested + unvested"
    );

    // Distribution total_claimed should reflect vested transfer
    let dist_account = ctx.get_account(&setup.distribution_pda).expect("Distribution should exist");
    let dist = MerkleDistribution::from_bytes(&dist_account.data).expect("Should deserialize");
    assert_eq!(dist.total_claimed, expected_vested, "total_claimed should include vested transfer");
}

#[test]
fn test_revoke_merkle_non_vested_after_full_vest() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::new(&mut ctx);

    ctx.warp_to_timestamp(setup.end_ts + 1);

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);

    let claimant_balance = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(claimant_balance, setup.total_amount, "Claimant should receive full amount when fully vested");

    let dist_account = ctx.get_account(&setup.distribution_pda).expect("Distribution should exist");
    let dist = MerkleDistribution::from_bytes(&dist_account.data).expect("Should deserialize");
    assert_eq!(dist.total_claimed, setup.total_amount, "All marked as claimed");
}

#[test]
fn test_revoke_merkle_full_at_midpoint() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::new(&mut ctx);

    let midpoint = setup.start_ts + (setup.end_ts - setup.start_ts) / 2;
    ctx.warp_to_timestamp(midpoint);

    let vault_balance_before = ctx.get_token_balance(&setup.distribution_vault);
    let claimant_balance_before = ctx.get_token_balance(&setup.claimant_token_account);
    let authority_balance_before = ctx.get_token_balance(&setup.authority_token_account);

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::Full);
    revoke_ix.send_expect_success(&mut ctx);

    let expected_vested = expected_linear_unlock(setup.total_amount, setup.start_ts, setup.end_ts, midpoint);
    let expected_unvested = setup.total_amount - expected_vested;
    let total_freed = expected_unvested + expected_vested;

    let claimant_balance_after = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(claimant_balance_after, claimant_balance_before, "Claimant should receive nothing in Full mode");

    let authority_balance_after = ctx.get_token_balance(&setup.authority_token_account);
    assert_eq!(
        authority_balance_after,
        authority_balance_before + total_freed,
        "Authority should receive all unclaimed tokens in Full mode"
    );

    let vault_balance_after = ctx.get_token_balance(&setup.distribution_vault);
    assert_eq!(
        vault_balance_after,
        vault_balance_before - total_freed,
        "Vault should decrease by total freed in Full mode"
    );

    let dist_account = ctx.get_account(&setup.distribution_pda).expect("Distribution should exist");
    let dist = MerkleDistribution::from_bytes(&dist_account.data).expect("Should deserialize");
    assert_eq!(dist.total_claimed, 0, "total_claimed should not change in Full mode");
}

#[test]
fn test_revoke_merkle_with_immediate_schedule() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::builder(&mut ctx).schedule(VestingSchedule::Immediate).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);

    let claimant_balance = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(claimant_balance, setup.total_amount, "All should transfer with Immediate schedule");
}

// ── Happy paths — claimant partially claimed ──────────────────────

#[test]
fn test_revoke_merkle_non_vested_after_partial_claim() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::new(&mut ctx);

    // Warp to 25% and claim
    let quarter = setup.start_ts + (setup.end_ts - setup.start_ts) / 4;
    ctx.warp_to_timestamp(quarter);

    let claim_ix = setup.build_claim_instruction(&ctx);
    claim_ix.send_expect_success(&mut ctx);

    let claimed_at_quarter = expected_linear_unlock(setup.total_amount, setup.start_ts, setup.end_ts, quarter);
    let claimant_balance_after_claim = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(claimant_balance_after_claim, claimed_at_quarter);

    // Warp to 50% and revoke NonVested
    let midpoint = setup.start_ts + (setup.end_ts - setup.start_ts) / 2;
    ctx.warp_to_timestamp(midpoint);

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);

    let vested_at_midpoint = expected_linear_unlock(setup.total_amount, setup.start_ts, setup.end_ts, midpoint);
    let vested_unclaimed = vested_at_midpoint - claimed_at_quarter;

    let claimant_balance_after_revoke = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(
        claimant_balance_after_revoke,
        claimant_balance_after_claim + vested_unclaimed,
        "Should transfer only the unclaimed vested portion"
    );

    let dist_account = ctx.get_account(&setup.distribution_pda).expect("Distribution should exist");
    let dist = MerkleDistribution::from_bytes(&dist_account.data).expect("Should deserialize");
    assert_eq!(
        dist.total_claimed,
        claimed_at_quarter + vested_unclaimed,
        "total_claimed should reflect both claim and revoke transfer"
    );
}

#[test]
fn test_revoke_merkle_full_after_partial_claim() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::new(&mut ctx);

    // Warp to 25% and claim
    let quarter = setup.start_ts + (setup.end_ts - setup.start_ts) / 4;
    ctx.warp_to_timestamp(quarter);

    let claim_ix = setup.build_claim_instruction(&ctx);
    claim_ix.send_expect_success(&mut ctx);

    let claimed_at_quarter = expected_linear_unlock(setup.total_amount, setup.start_ts, setup.end_ts, quarter);
    let claimant_balance_after_claim = ctx.get_token_balance(&setup.claimant_token_account);

    // Warp to 50% and revoke Full
    let midpoint = setup.start_ts + (setup.end_ts - setup.start_ts) / 2;
    ctx.warp_to_timestamp(midpoint);

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::Full);
    revoke_ix.send_expect_success(&mut ctx);

    let claimant_balance_after_revoke = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(claimant_balance_after_revoke, claimant_balance_after_claim, "Full mode should not transfer anything");

    let dist_account = ctx.get_account(&setup.distribution_pda).expect("Distribution should exist");
    let dist = MerkleDistribution::from_bytes(&dist_account.data).expect("Should deserialize");
    assert_eq!(dist.total_claimed, claimed_at_quarter, "total_claimed should only reflect the original claim");
}

// ── Post-revocation behavior ──────────────────────────────────────

#[test]
fn test_claim_after_revocation_fails() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::new(&mut ctx);

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);

    ctx.warp_to_timestamp(setup.end_ts + 1);

    let claim_ix = setup.build_claim_instruction(&ctx);
    let error = claim_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::ClaimantAlreadyRevoked);
}

// ── Token-2022 support ────────────────────────────────────────────

#[test]
fn test_revoke_merkle_with_token_2022() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::new_token_2022(&mut ctx);

    let midpoint = setup.start_ts + (setup.end_ts - setup.start_ts) / 2;
    ctx.warp_to_timestamp(midpoint);

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);

    let expected_vested = expected_linear_unlock(setup.total_amount, setup.start_ts, setup.end_ts, midpoint);
    let claimant_balance = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(claimant_balance, expected_vested, "Token-2022 revoke should work");
}

// ── Bitmask permission tests ──────────────────────────────────────

#[test]
fn test_revoke_merkle_non_vested_rejected_when_only_full_bit_set() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::builder(&mut ctx).revocable(2).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    let error = revoke_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::DistributionNotRevocable);
}

#[test]
fn test_revoke_merkle_full_rejected_when_only_non_vested_bit_set() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::builder(&mut ctx).revocable(1).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::Full);
    let error = revoke_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::DistributionNotRevocable);
}

#[test]
fn test_revoke_merkle_non_vested_succeeds_when_revocable_3() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::builder(&mut ctx).revocable(3).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);
}

#[test]
fn test_revoke_merkle_full_succeeds_when_revocable_3() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::builder(&mut ctx).revocable(3).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::Full);
    revoke_ix.send_expect_success(&mut ctx);
}

#[test]
fn test_revoke_merkle_non_vested_succeeds_when_only_non_vested_bit_set() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::builder(&mut ctx).revocable(1).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);
}

#[test]
fn test_revoke_merkle_full_succeeds_when_only_full_bit_set() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::builder(&mut ctx).revocable(2).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::Full);
    revoke_ix.send_expect_success(&mut ctx);
}

#[test]
fn test_revoke_merkle_all_modes_rejected_when_revocable_0() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::builder(&mut ctx).revocable(0).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    let error = revoke_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::DistributionNotRevocable);
}

#[test]
fn test_revoke_merkle_full_rejected_when_revocable_0() {
    let mut ctx = TestContext::new();
    let setup = RevokeMerkleClaimSetup::builder(&mut ctx).revocable(0).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::Full);
    let error = revoke_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::DistributionNotRevocable);
}
