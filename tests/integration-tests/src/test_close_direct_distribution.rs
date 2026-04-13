use solana_sdk::signature::Signer;

use crate::fixtures::{
    AddDirectRecipientSetup, CloseDirectDistributionFixture, CloseDirectDistributionSetup,
    CreateDirectDistributionSetup,
};
use crate::utils::{
    assert_account_closed, assert_rewards_error, test_empty_data, test_missing_signer, test_not_writable,
    test_wrong_current_program, test_wrong_system_program, RewardsError, TestContext, PROGRAM_ID,
};

#[test]
fn test_close_direct_distribution_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<CloseDirectDistributionFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_close_direct_distribution_distribution_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CloseDirectDistributionFixture>(&mut ctx, 1);
}

#[test]
fn test_close_direct_distribution_distribution_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CloseDirectDistributionFixture>(&mut ctx, 4);
}

#[test]
fn test_close_direct_distribution_tombstone_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CloseDirectDistributionFixture>(&mut ctx, 2);
}

#[test]
fn test_close_direct_distribution_authority_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CloseDirectDistributionFixture>(&mut ctx, 5);
}

#[test]
fn test_close_direct_distribution_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<CloseDirectDistributionFixture>(&mut ctx);
}

#[test]
fn test_close_direct_distribution_wrong_system_program() {
    let mut ctx = TestContext::new();
    test_wrong_system_program::<CloseDirectDistributionFixture>(&mut ctx);
}

#[test]
fn test_close_direct_distribution_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<CloseDirectDistributionFixture>(&mut ctx);
}

#[test]
fn test_close_direct_distribution_success() {
    let mut ctx = TestContext::new();
    let setup = CloseDirectDistributionSetup::new(&mut ctx);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.distribution_pda);
    let tombstone_account = ctx.get_account(&setup.tombstone_pda).expect("Tombstone should exist after close");
    assert_eq!(tombstone_account.owner, PROGRAM_ID);
}

#[test]
fn test_close_direct_distribution_success_token_2022() {
    let mut ctx = TestContext::new();
    let setup = CloseDirectDistributionSetup::new_token_2022(&mut ctx);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.distribution_pda);
    let tombstone_account = ctx.get_account(&setup.tombstone_pda).expect("Tombstone should exist after close");
    assert_eq!(tombstone_account.owner, PROGRAM_ID);
}

#[test]
fn test_close_direct_distribution_unauthorized() {
    let mut ctx = TestContext::new();
    let setup = CloseDirectDistributionSetup::new(&mut ctx);

    let wrong_authority = ctx.create_funded_keypair();
    let wrong_token_account = ctx.create_token_account(&wrong_authority.pubkey(), &setup.mint);

    let test_ix = setup.build_instruction_with_wrong_authority(&ctx, &wrong_authority, wrong_token_account);
    let error = test_ix.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_close_direct_distribution_returns_tokens() {
    let mut ctx = TestContext::new();
    let distribution_setup = CreateDirectDistributionSetup::new(&mut ctx);
    let recipient_setup = AddDirectRecipientSetup::from_distribution_setup(&mut ctx, &distribution_setup);
    let add_recipient_ix = recipient_setup.build_instruction(&ctx);
    add_recipient_ix.send_expect_success(&mut ctx);

    let distribution_vault_balance = ctx.get_token_balance(&distribution_setup.distribution_vault);
    assert!(distribution_vault_balance > 0, "Vault should have funded tokens");

    let close_setup = CloseDirectDistributionSetup {
        authority: distribution_setup.authority.insecure_clone(),
        distribution_pda: distribution_setup.distribution_pda,
        tombstone_pda: distribution_setup.tombstone_pda,
        mint: distribution_setup.mint.pubkey(),
        distribution_vault: distribution_setup.distribution_vault,
        authority_token_account: recipient_setup.authority_token_account,
        token_program: distribution_setup.token_program,
    };

    let authority_balance_before = ctx.get_token_balance(&close_setup.authority_token_account);

    let test_ix = close_setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let authority_balance_after = ctx.get_token_balance(&close_setup.authority_token_account);
    assert_eq!(
        authority_balance_after,
        authority_balance_before + distribution_vault_balance,
        "Authority should receive vault tokens"
    );
}

// ── Clawback timestamp tests ───────────────────────────────────────

#[test]
fn test_close_direct_distribution_clawback_ts_zero_succeeds() {
    let mut ctx = TestContext::new();
    // clawback_ts=0 means no gate (default)
    let setup = CloseDirectDistributionSetup::new(&mut ctx);
    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);
    assert_account_closed(&ctx, &setup.distribution_pda);
}

#[test]
fn test_close_direct_distribution_clawback_ts_before_timestamp_fails() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let future_ts = current_ts + 86400 * 30; // 30 days in the future

    let distribution_setup = CreateDirectDistributionSetup::builder(&mut ctx).clawback_ts(future_ts).build();
    let create_ix = distribution_setup.build_instruction(&ctx);
    create_ix.send_expect_success(&mut ctx);

    let authority_token_account =
        ctx.create_token_account(&distribution_setup.authority.pubkey(), &distribution_setup.mint.pubkey());

    let close_setup = CloseDirectDistributionSetup {
        authority: distribution_setup.authority.insecure_clone(),
        distribution_pda: distribution_setup.distribution_pda,
        tombstone_pda: distribution_setup.tombstone_pda,
        mint: distribution_setup.mint.pubkey(),
        distribution_vault: distribution_setup.distribution_vault,
        authority_token_account,
        token_program: distribution_setup.token_program,
    };

    let test_ix = close_setup.build_instruction(&ctx);
    let error = test_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::ClawbackNotReached);
}

#[test]
fn test_close_direct_distribution_clawback_ts_after_timestamp_succeeds() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let future_ts = current_ts + 86400 * 30;

    let distribution_setup = CreateDirectDistributionSetup::builder(&mut ctx).clawback_ts(future_ts).build();
    let create_ix = distribution_setup.build_instruction(&ctx);
    create_ix.send_expect_success(&mut ctx);

    // Warp past the clawback timestamp
    ctx.warp_to_timestamp(future_ts + 1);

    let authority_token_account =
        ctx.create_token_account(&distribution_setup.authority.pubkey(), &distribution_setup.mint.pubkey());

    let close_setup = CloseDirectDistributionSetup {
        authority: distribution_setup.authority.insecure_clone(),
        distribution_pda: distribution_setup.distribution_pda,
        tombstone_pda: distribution_setup.tombstone_pda,
        mint: distribution_setup.mint.pubkey(),
        distribution_vault: distribution_setup.distribution_vault,
        authority_token_account,
        token_program: distribution_setup.token_program,
    };

    let test_ix = close_setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);
    assert_account_closed(&ctx, &close_setup.distribution_pda);
}
