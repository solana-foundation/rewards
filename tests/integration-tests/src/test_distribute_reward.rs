use solana_sdk::signature::Signer;

use crate::fixtures::{
    CreateContinuousPoolSetup, DistributeContinuousRewardFixture, DistributeContinuousRewardSetup,
    DEFAULT_REWARD_AMOUNT,
};
use crate::utils::{
    assert_rewards_error, find_event_authority_pda, test_empty_data, test_missing_signer, test_not_writable,
    test_truncated_data, test_wrong_current_program, RewardsError, TestContext, TestInstruction,
};

// ─── Generic validation tests ───

#[test]
fn test_distribute_reward_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<DistributeContinuousRewardFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_distribute_reward_pool_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<DistributeContinuousRewardFixture>(&mut ctx, 1);
}

#[test]
fn test_distribute_reward_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<DistributeContinuousRewardFixture>(&mut ctx, 3);
}

#[test]
fn test_distribute_reward_authority_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<DistributeContinuousRewardFixture>(&mut ctx, 4);
}

#[test]
fn test_distribute_reward_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<DistributeContinuousRewardFixture>(&mut ctx);
}

#[test]
fn test_distribute_reward_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<DistributeContinuousRewardFixture>(&mut ctx);
}

#[test]
fn test_distribute_reward_truncated_data() {
    let mut ctx = TestContext::new();
    test_truncated_data::<DistributeContinuousRewardFixture>(&mut ctx);
}

// ─── Custom error tests ───

#[test]
fn test_distribute_reward_unauthorized_authority() {
    let mut ctx = TestContext::new();
    let setup = DistributeContinuousRewardSetup::new(&mut ctx);

    let wrong_authority = ctx.create_funded_keypair();
    let (event_authority, _) = find_event_authority_pda();
    let pool = &setup.opt_in_setup.pool_setup;

    let mut builder = rewards_program_client::instructions::DistributeContinuousRewardBuilder::new();
    builder
        .authority(wrong_authority.pubkey())
        .reward_pool(pool.reward_pool_pda)
        .reward_mint(pool.reward_mint.pubkey())
        .reward_vault(pool.reward_vault)
        .authority_token_account(setup.authority_token_account)
        .reward_token_program(pool.reward_token_program)
        .event_authority(event_authority)
        .amount(DEFAULT_REWARD_AMOUNT);

    let ix = TestInstruction {
        instruction: builder.instruction(),
        signers: vec![wrong_authority],
        name: "DistributeContinuousReward",
    };
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_distribute_reward_zero_amount() {
    let mut ctx = TestContext::new();
    let setup = DistributeContinuousRewardSetup::new_with_amount(&mut ctx, 0);
    let ix = setup.build_instruction(&ctx);

    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InvalidAmount);
}

#[test]
fn test_distribute_reward_no_opted_in_users() {
    let mut ctx = TestContext::new();

    let pool_setup = CreateContinuousPoolSetup::new(&mut ctx);
    pool_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let authority_token_account = ctx.create_token_account_with_balance(
        &pool_setup.authority.pubkey(),
        &pool_setup.reward_mint.pubkey(),
        1_000_000,
    );
    let (event_authority, _) = find_event_authority_pda();

    let mut builder = rewards_program_client::instructions::DistributeContinuousRewardBuilder::new();
    builder
        .authority(pool_setup.authority.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .reward_mint(pool_setup.reward_mint.pubkey())
        .reward_vault(pool_setup.reward_vault)
        .authority_token_account(authority_token_account)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority)
        .amount(100_000);

    let ix = TestInstruction {
        instruction: builder.instruction(),
        signers: vec![pool_setup.authority.insecure_clone()],
        name: "DistributeContinuousReward",
    };
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::NoOptedInUsers);
}

// Note: DistributionAmountTooSmall requires opted_in_supply > amount * REWARD_PRECISION (1e12).
// With standard 6-decimal tokens, amount=1 needs supply > 1e12 to trigger, making it
// impractical to test without a specialized high-supply fixture.
