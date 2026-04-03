use rewards_program_client::instructions::ContinuousOptInBuilder;
use rewards_program_client::types::RevokeMode;
use solana_sdk::signature::Signer;

use crate::fixtures::{
    build_revoke_user_instruction, ContinuousOptInFixture, ContinuousOptInSetup, CreateContinuousPoolSetup,
    DistributeContinuousRewardSetup, DEFAULT_REWARD_AMOUNT, DEFAULT_TRACKED_BALANCE,
};
use crate::utils::{
    assert_rewards_error, find_event_authority_pda, find_revocation_pda, find_user_reward_account_pda,
    test_missing_signer, RewardsError, TestContext, TestInstruction,
};

// ─── Validation tests (already exist in lifecycle, but isolated here) ───

#[test]
fn test_opt_in_missing_user_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<ContinuousOptInFixture>(&mut ctx, 1, 0);
}

// ─── Custom error tests ───

#[test]
fn test_opt_in_user_revoked() {
    let mut ctx = TestContext::new();

    let mut pool_setup = CreateContinuousPoolSetup::new(&mut ctx);
    pool_setup.revocable = 3;
    pool_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let user = ctx.create_funded_keypair();
    let user_tracked_ta = ctx.create_token_account_with_balance(
        &user.pubkey(),
        &pool_setup.tracked_mint.pubkey(),
        DEFAULT_TRACKED_BALANCE,
    );
    let (user_reward_pda, user_reward_bump) = find_user_reward_account_pda(&pool_setup.reward_pool_pda, &user.pubkey());

    let (event_authority, _) = find_event_authority_pda();
    let (revocation_pda, _) = find_revocation_pda(&pool_setup.reward_pool_pda, &user.pubkey());

    let mut opt_in_builder = ContinuousOptInBuilder::new();
    opt_in_builder
        .payer(ctx.payer.pubkey())
        .user(user.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(user_reward_pda)
        .revocation_marker(revocation_pda)
        .user_tracked_token_account(user_tracked_ta)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .tracked_token_program(spl_token_interface::ID)
        .event_authority(event_authority)
        .bump(user_reward_bump);
    let opt_in_ix = TestInstruction {
        instruction: opt_in_builder.instruction(),
        signers: vec![user.insecure_clone()],
        name: "ContinuousOptIn",
    };
    opt_in_ix.send_expect_success(&mut ctx);

    let authority_token_account = ctx.create_token_account_with_balance(
        &pool_setup.authority.pubkey(),
        &pool_setup.reward_mint.pubkey(),
        DEFAULT_REWARD_AMOUNT * 10,
    );

    let distribute_setup = DistributeContinuousRewardSetup {
        opt_in_setup: ContinuousOptInSetup {
            pool_setup,
            user: user.insecure_clone(),
            user_tracked_token_account: user_tracked_ta,
            user_reward_pda,
            user_reward_bump,
            initial_balance: DEFAULT_TRACKED_BALANCE,
        },
        authority_token_account,
        amount: DEFAULT_REWARD_AMOUNT,
    };
    distribute_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let pool_setup_ref = &distribute_setup.opt_in_setup.pool_setup;
    let user_reward_token_account = ctx.create_token_account(&user.pubkey(), &pool_setup_ref.reward_mint.pubkey());
    let authority_reward_token_account =
        ctx.create_token_account(&pool_setup_ref.authority.pubkey(), &pool_setup_ref.reward_mint.pubkey());

    let revoke_ix = build_revoke_user_instruction(
        &ctx,
        pool_setup_ref,
        &user,
        &user_reward_pda,
        &user_tracked_ta,
        &user_reward_token_account,
        &authority_reward_token_account,
        RevokeMode::NonVested,
    );
    revoke_ix.send_expect_success(&mut ctx);

    ctx.advance_slot();

    let (user_reward_pda_new, user_reward_bump_new) =
        find_user_reward_account_pda(&pool_setup_ref.reward_pool_pda, &user.pubkey());

    let mut opt_in_builder2 = ContinuousOptInBuilder::new();
    opt_in_builder2
        .payer(ctx.payer.pubkey())
        .user(user.pubkey())
        .reward_pool(pool_setup_ref.reward_pool_pda)
        .user_reward_account(user_reward_pda_new)
        .revocation_marker(revocation_pda)
        .user_tracked_token_account(user_tracked_ta)
        .tracked_mint(pool_setup_ref.tracked_mint.pubkey())
        .tracked_token_program(spl_token_interface::ID)
        .event_authority(event_authority)
        .bump(user_reward_bump_new);

    let opt_in_ix2 =
        TestInstruction { instruction: opt_in_builder2.instruction(), signers: vec![user], name: "ContinuousOptIn" };
    let error = opt_in_ix2.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UserRevoked);
}
