use solana_sdk::signature::Signer;

use crate::fixtures::{CloseContinuousPoolFixture, CloseContinuousPoolSetup, CreateContinuousPoolSetup};
use crate::utils::{
    assert_rewards_error, find_event_authority_pda, test_missing_signer, test_not_writable, test_wrong_current_program,
    RewardsError, TestContext, TestInstruction,
};

// ─── Generic validation tests ───

#[test]
fn test_close_reward_pool_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<CloseContinuousPoolFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_close_reward_pool_pool_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CloseContinuousPoolFixture>(&mut ctx, 1);
}

#[test]
fn test_close_reward_pool_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CloseContinuousPoolFixture>(&mut ctx, 3);
}

#[test]
fn test_close_reward_pool_authority_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CloseContinuousPoolFixture>(&mut ctx, 4);
}

#[test]
fn test_close_reward_pool_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<CloseContinuousPoolFixture>(&mut ctx);
}

// ─── Custom error tests ───

#[test]
fn test_close_reward_pool_unauthorized_authority() {
    let mut ctx = TestContext::new();

    let pool_setup = CreateContinuousPoolSetup::new(&mut ctx);
    pool_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let wrong_authority = ctx.create_funded_keypair();
    let authority_ta = ctx.create_token_account(&wrong_authority.pubkey(), &pool_setup.reward_mint.pubkey());
    let (event_authority, _) = find_event_authority_pda();

    let mut builder = rewards_program_client::instructions::CloseContinuousPoolBuilder::new();
    builder
        .authority(wrong_authority.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .reward_mint(pool_setup.reward_mint.pubkey())
        .reward_vault(pool_setup.reward_vault)
        .authority_token_account(authority_ta)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority);

    let ix = TestInstruction {
        instruction: builder.instruction(),
        signers: vec![wrong_authority],
        name: "CloseContinuousPool",
    };
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_close_reward_pool_clawback_not_reached() {
    let mut ctx = TestContext::new();

    let future_ts = ctx.get_current_timestamp() + 86400;
    let setup = CloseContinuousPoolSetup::new_with_clawback(&mut ctx, future_ts);
    let ix = setup.build_instruction(&ctx);

    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::ClawbackNotReached);
}

#[test]
fn test_close_reward_pool_clawback_after_timestamp_succeeds() {
    let mut ctx = TestContext::new();

    let future_ts = ctx.get_current_timestamp() + 86400;
    let setup = CloseContinuousPoolSetup::new_with_clawback(&mut ctx, future_ts);

    ctx.warp_to_timestamp(future_ts + 1);

    let ix = setup.build_instruction(&ctx);
    ix.send_expect_success(&mut ctx);
}
