use solana_sdk::signature::Signer;

use crate::fixtures::{build_set_balance_instruction, ContinuousOptInSetup, SetContinuousBalanceFixture};
use crate::utils::{
    assert_rewards_error, find_event_authority_pda, test_missing_signer, test_not_writable, RewardsError, TestContext,
    TestInstruction,
};

// ─── Generic validation tests ───

#[test]
fn test_set_balance_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<SetContinuousBalanceFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_set_balance_pool_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<SetContinuousBalanceFixture>(&mut ctx, 1);
}

#[test]
fn test_set_balance_user_reward_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<SetContinuousBalanceFixture>(&mut ctx, 2);
}

// ─── Custom error tests ───

#[test]
fn test_set_balance_wrong_balance_source() {
    let mut ctx = TestContext::new();

    let setup = ContinuousOptInSetup::new(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let set_ix =
        build_set_balance_instruction(&setup.pool_setup, &setup.user.pubkey(), &setup.user_reward_pda, 500_000);

    let error = set_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::BalanceSourceMismatch);
}

#[test]
fn test_set_balance_unauthorized_authority() {
    let mut ctx = TestContext::new();

    let setup = ContinuousOptInSetup::new_authority_set(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let wrong_authority = ctx.create_funded_keypair();
    let (event_authority, _) = find_event_authority_pda();
    let mut builder = rewards_program_client::instructions::SetContinuousBalanceBuilder::new();
    builder
        .authority(wrong_authority.pubkey())
        .reward_pool(setup.pool_setup.reward_pool_pda)
        .user_reward_account(setup.user_reward_pda)
        .user(setup.user.pubkey())
        .event_authority(event_authority)
        .balance(500_000);

    let ix = TestInstruction {
        instruction: builder.instruction(),
        signers: vec![wrong_authority],
        name: "SetContinuousBalance",
    };
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}
