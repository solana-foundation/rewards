use rewards_program_client::types::BalanceSource;
use solana_sdk::signer::Signer;

use crate::fixtures::{
    build_claim_continuous_instruction, build_close_reward_pool_instruction, build_opt_out_instruction,
    build_set_balance_instruction, build_sync_balance_instruction, ContinuousOptInFixture, ContinuousOptInSetup,
    CreateContinuousPoolFixture, CreateContinuousPoolSetup, DistributeContinuousRewardSetup, DEFAULT_REWARD_AMOUNT,
    DEFAULT_TRACKED_BALANCE,
};
use crate::utils::{
    assert_account_closed, assert_reward_pool, find_revocation_pda, find_user_reward_account_pda, get_reward_pool,
    get_user_reward_account, test_empty_data, test_missing_signer, test_not_writable, test_truncated_data,
    test_wrong_current_program, test_wrong_system_program, TestContext,
};

// ─── CreateContinuousPool generic tests ───

#[test]
fn test_create_reward_pool_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<CreateContinuousPoolFixture>(&mut ctx, 1, 0);
}

#[test]
fn test_create_reward_pool_missing_seeds_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<CreateContinuousPoolFixture>(&mut ctx, 2, 1);
}

#[test]
fn test_create_reward_pool_pool_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CreateContinuousPoolFixture>(&mut ctx, 3);
}

#[test]
fn test_create_reward_pool_wrong_system_program() {
    let mut ctx = TestContext::new();
    test_wrong_system_program::<CreateContinuousPoolFixture>(&mut ctx);
}

#[test]
fn test_create_reward_pool_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<CreateContinuousPoolFixture>(&mut ctx);
}

#[test]
fn test_create_reward_pool_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<CreateContinuousPoolFixture>(&mut ctx);
}

#[test]
fn test_create_reward_pool_truncated_data() {
    let mut ctx = TestContext::new();
    test_truncated_data::<CreateContinuousPoolFixture>(&mut ctx);
}

#[test]
fn test_create_reward_pool_success() {
    let mut ctx = TestContext::new();
    let setup = CreateContinuousPoolSetup::new(&mut ctx);
    let instruction = setup.build_instruction(&ctx);

    instruction.send_expect_success(&mut ctx);

    assert_reward_pool(
        &ctx,
        &setup.reward_pool_pda,
        &setup.authority.pubkey(),
        &setup.tracked_mint.pubkey(),
        &setup.reward_mint.pubkey(),
        setup.bump,
    );

    let pool = get_reward_pool(&ctx, &setup.reward_pool_pda);
    assert_eq!(pool.balance_source, BalanceSource::OnChain);
    assert_eq!(pool.opted_in_supply, 0);
    assert_eq!(pool.total_distributed, 0);
    assert_eq!(pool.total_claimed, 0);
    assert_eq!(pool.reward_per_token, 0);
}

#[test]
fn test_create_reward_pool_authority_set_mode() {
    let mut ctx = TestContext::new();
    let setup = CreateContinuousPoolSetup::new_authority_set(&mut ctx);
    let instruction = setup.build_instruction(&ctx);

    instruction.send_expect_success(&mut ctx);

    let pool = get_reward_pool(&ctx, &setup.reward_pool_pda);
    assert_eq!(pool.balance_source, BalanceSource::AuthoritySet);
}

// ─── ContinuousOptIn generic tests ───

#[test]
fn test_opt_in_missing_user_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<ContinuousOptInFixture>(&mut ctx, 1, 0);
}

#[test]
fn test_opt_in_pool_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ContinuousOptInFixture>(&mut ctx, 2);
}

#[test]
fn test_opt_in_user_reward_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ContinuousOptInFixture>(&mut ctx, 3);
}

#[test]
fn test_opt_in_wrong_system_program() {
    let mut ctx = TestContext::new();
    test_wrong_system_program::<ContinuousOptInFixture>(&mut ctx);
}

#[test]
fn test_opt_in_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<ContinuousOptInFixture>(&mut ctx);
}

#[test]
fn test_opt_in_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<ContinuousOptInFixture>(&mut ctx);
}

#[test]
fn test_opt_in_success() {
    let mut ctx = TestContext::new();
    let setup = ContinuousOptInSetup::new(&mut ctx);
    let instruction = setup.build_instruction(&ctx);

    instruction.send_expect_success(&mut ctx);

    let user_account = get_user_reward_account(&ctx, &setup.user_reward_pda);
    assert_eq!(user_account.bump, setup.user_reward_bump);
    assert_eq!(user_account.last_known_balance, DEFAULT_TRACKED_BALANCE);
    assert_eq!(user_account.accrued_rewards, 0);
    assert_eq!(user_account.reward_per_token_paid, 0);

    let pool = get_reward_pool(&ctx, &setup.pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, DEFAULT_TRACKED_BALANCE);
}

#[test]
fn test_opt_in_authority_set_mode_zero_balance() {
    let mut ctx = TestContext::new();
    let setup = ContinuousOptInSetup::new_authority_set(&mut ctx);
    let instruction = setup.build_instruction(&ctx);

    instruction.send_expect_success(&mut ctx);

    let user_account = get_user_reward_account(&ctx, &setup.user_reward_pda);
    assert_eq!(user_account.last_known_balance, 0);

    let pool = get_reward_pool(&ctx, &setup.pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, 0);
}

// ─── DistributeContinuousReward tests ───

#[test]
fn test_distribute_reward_success() {
    let mut ctx = TestContext::new();
    let setup = DistributeContinuousRewardSetup::new(&mut ctx);
    let instruction = setup.build_instruction(&ctx);

    instruction.send_expect_success(&mut ctx);

    let pool = get_reward_pool(&ctx, &setup.opt_in_setup.pool_setup.reward_pool_pda);
    assert_eq!(pool.total_distributed, DEFAULT_REWARD_AMOUNT);

    let expected_rpt = (DEFAULT_REWARD_AMOUNT as u128 * 1_000_000_000_000) / DEFAULT_TRACKED_BALANCE as u128;
    assert_eq!(pool.reward_per_token, expected_rpt);
}

#[test]
fn test_distribute_reward_updates_vault_balance() {
    let mut ctx = TestContext::new();
    let setup = DistributeContinuousRewardSetup::new(&mut ctx);
    let vault = setup.opt_in_setup.pool_setup.reward_vault;

    let before = ctx.get_token_balance(&vault);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);
    let after = ctx.get_token_balance(&vault);

    assert_eq!(after - before, DEFAULT_REWARD_AMOUNT);
}

// ─── ClaimContinuous tests ───

#[test]
fn test_claim_continuous_full() {
    let mut ctx = TestContext::new();
    let setup = DistributeContinuousRewardSetup::new(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let pool_setup = &setup.opt_in_setup.pool_setup;
    let user = &setup.opt_in_setup.user;
    let user_reward_pda = &setup.opt_in_setup.user_reward_pda;
    let user_tracked_ta = &setup.opt_in_setup.user_tracked_token_account;

    let user_reward_ta = ctx.create_token_account(&user.pubkey(), &pool_setup.reward_mint.pubkey());

    let claim_ix = build_claim_continuous_instruction(
        &ctx,
        pool_setup,
        user,
        user_reward_pda,
        user_tracked_ta,
        &user_reward_ta,
        0,
    );
    claim_ix.send_expect_success(&mut ctx);

    let user_account = get_user_reward_account(&ctx, user_reward_pda);
    assert_eq!(user_account.accrued_rewards, 0);

    let balance = ctx.get_token_balance(&user_reward_ta);
    assert_eq!(balance, DEFAULT_REWARD_AMOUNT);

    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    assert_eq!(pool.total_claimed, DEFAULT_REWARD_AMOUNT);
}

#[test]
fn test_claim_continuous_partial() {
    let mut ctx = TestContext::new();
    let setup = DistributeContinuousRewardSetup::new(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let pool_setup = &setup.opt_in_setup.pool_setup;
    let user = &setup.opt_in_setup.user;
    let user_reward_pda = &setup.opt_in_setup.user_reward_pda;
    let user_tracked_ta = &setup.opt_in_setup.user_tracked_token_account;

    let user_reward_ta = ctx.create_token_account(&user.pubkey(), &pool_setup.reward_mint.pubkey());

    let partial_amount = DEFAULT_REWARD_AMOUNT / 2;
    let claim_ix = build_claim_continuous_instruction(
        &ctx,
        pool_setup,
        user,
        user_reward_pda,
        user_tracked_ta,
        &user_reward_ta,
        partial_amount,
    );
    claim_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&user_reward_ta);
    assert_eq!(balance, partial_amount);

    let user_account = get_user_reward_account(&ctx, user_reward_pda);
    assert_eq!(user_account.accrued_rewards, DEFAULT_REWARD_AMOUNT - partial_amount);
}

// ─── SyncContinuousBalance tests ───

#[test]
fn test_sync_balance_increases_supply() {
    let mut ctx = TestContext::new();
    let setup = ContinuousOptInSetup::new(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let new_balance = DEFAULT_TRACKED_BALANCE * 2;
    ctx.set_token_balance(&setup.user_tracked_token_account, new_balance);

    let sync_ix = build_sync_balance_instruction(
        &setup.pool_setup,
        &setup.user.pubkey(),
        &setup.user_reward_pda,
        &setup.user_tracked_token_account,
    );
    sync_ix.send_expect_success(&mut ctx);

    let user_account = get_user_reward_account(&ctx, &setup.user_reward_pda);
    assert_eq!(user_account.last_known_balance, new_balance);

    let pool = get_reward_pool(&ctx, &setup.pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, new_balance);
}

#[test]
fn test_sync_balance_decreases_supply() {
    let mut ctx = TestContext::new();
    let setup = ContinuousOptInSetup::new(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let new_balance = DEFAULT_TRACKED_BALANCE / 2;
    ctx.set_token_balance(&setup.user_tracked_token_account, new_balance);

    let sync_ix = build_sync_balance_instruction(
        &setup.pool_setup,
        &setup.user.pubkey(),
        &setup.user_reward_pda,
        &setup.user_tracked_token_account,
    );
    sync_ix.send_expect_success(&mut ctx);

    let user_account = get_user_reward_account(&ctx, &setup.user_reward_pda);
    assert_eq!(user_account.last_known_balance, new_balance);

    let pool = get_reward_pool(&ctx, &setup.pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, new_balance);
}

// ─── SetContinuousBalance tests ───

#[test]
fn test_set_balance_success() {
    let mut ctx = TestContext::new();
    let setup = ContinuousOptInSetup::new_authority_set(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let new_balance = 500_000;
    let set_ix =
        build_set_balance_instruction(&setup.pool_setup, &setup.user.pubkey(), &setup.user_reward_pda, new_balance);
    set_ix.send_expect_success(&mut ctx);

    let user_account = get_user_reward_account(&ctx, &setup.user_reward_pda);
    assert_eq!(user_account.last_known_balance, new_balance);

    let pool = get_reward_pool(&ctx, &setup.pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, new_balance);
}

// ─── ContinuousOptOut tests ───

#[test]
fn test_opt_out_with_rewards() {
    let mut ctx = TestContext::new();
    let setup = DistributeContinuousRewardSetup::new(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let pool_setup = &setup.opt_in_setup.pool_setup;
    let user = &setup.opt_in_setup.user;
    let user_reward_pda = &setup.opt_in_setup.user_reward_pda;
    let user_tracked_ta = &setup.opt_in_setup.user_tracked_token_account;

    let user_reward_ta = ctx.create_token_account(&user.pubkey(), &pool_setup.reward_mint.pubkey());

    let opt_out_ix =
        build_opt_out_instruction(&ctx, pool_setup, user, user_reward_pda, user_tracked_ta, &user_reward_ta);
    opt_out_ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, user_reward_pda);

    let balance = ctx.get_token_balance(&user_reward_ta);
    assert_eq!(balance, DEFAULT_REWARD_AMOUNT);

    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, 0);
    assert_eq!(pool.total_claimed, DEFAULT_REWARD_AMOUNT);
}

#[test]
fn test_opt_out_no_rewards() {
    let mut ctx = TestContext::new();
    let setup = ContinuousOptInSetup::new(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let pool_setup = &setup.pool_setup;
    let user_reward_ta = ctx.create_token_account(&setup.user.pubkey(), &pool_setup.reward_mint.pubkey());

    let opt_out_ix = build_opt_out_instruction(
        &ctx,
        pool_setup,
        &setup.user,
        &setup.user_reward_pda,
        &setup.user_tracked_token_account,
        &user_reward_ta,
    );
    opt_out_ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.user_reward_pda);

    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, 0);
}

// ─── CloseContinuousPool tests ───

#[test]
fn test_close_reward_pool_success() {
    let mut ctx = TestContext::new();
    let setup = CreateContinuousPoolSetup::new(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let authority_ta = ctx.create_token_account(&setup.authority.pubkey(), &setup.reward_mint.pubkey());

    let close_ix = build_close_reward_pool_instruction(&ctx, &setup, &authority_ta);
    close_ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.reward_pool_pda);
}

#[test]
fn test_close_reward_pool_with_remaining_tokens() {
    let mut ctx = TestContext::new();
    let setup = DistributeContinuousRewardSetup::new(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let pool_setup = &setup.opt_in_setup.pool_setup;
    let authority_ta = ctx.create_token_account(&pool_setup.authority.pubkey(), &pool_setup.reward_mint.pubkey());

    let close_ix = build_close_reward_pool_instruction(&ctx, pool_setup, &authority_ta);
    close_ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &pool_setup.reward_pool_pda);

    let authority_balance = ctx.get_token_balance(&authority_ta);
    assert_eq!(authority_balance, DEFAULT_REWARD_AMOUNT);
}

// ─── Full lifecycle test ───

#[test]
fn test_full_lifecycle_on_chain_balance() {
    let mut ctx = TestContext::new();

    // 1. Create pool
    let pool_setup = CreateContinuousPoolSetup::new(&mut ctx);
    pool_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    // 2. User A opts in with 1_000_000
    let user_a = ctx.create_funded_keypair();
    let user_a_tracked_ta =
        ctx.create_token_account_with_balance(&user_a.pubkey(), &pool_setup.tracked_mint.pubkey(), 1_000_000);
    let (user_a_reward_pda, user_a_bump) = find_user_reward_account_pda(&pool_setup.reward_pool_pda, &user_a.pubkey());

    let (event_authority, _) = crate::utils::find_event_authority_pda();
    let (user_a_revocation_pda, _) = find_revocation_pda(&pool_setup.reward_pool_pda, &user_a.pubkey());
    let mut opt_in_builder = rewards_program_client::instructions::ContinuousOptInBuilder::new();
    opt_in_builder
        .payer(ctx.payer.pubkey())
        .user(user_a.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(user_a_reward_pda)
        .revocation_marker(user_a_revocation_pda)
        .user_tracked_token_account(user_a_tracked_ta)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .tracked_token_program(spl_token_interface::ID)
        .event_authority(event_authority)
        .bump(user_a_bump);
    let opt_in_ix = crate::utils::TestInstruction {
        instruction: opt_in_builder.instruction(),
        signers: vec![user_a.insecure_clone()],
        name: "ContinuousOptIn",
    };
    opt_in_ix.send_expect_success(&mut ctx);

    // 3. User B opts in with 500_000
    let user_b = ctx.create_funded_keypair();
    let user_b_tracked_ta =
        ctx.create_token_account_with_balance(&user_b.pubkey(), &pool_setup.tracked_mint.pubkey(), 500_000);
    let (user_b_reward_pda, user_b_bump) = find_user_reward_account_pda(&pool_setup.reward_pool_pda, &user_b.pubkey());

    let (user_b_revocation_pda, _) = find_revocation_pda(&pool_setup.reward_pool_pda, &user_b.pubkey());
    let mut opt_in_builder_b = rewards_program_client::instructions::ContinuousOptInBuilder::new();
    opt_in_builder_b
        .payer(ctx.payer.pubkey())
        .user(user_b.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(user_b_reward_pda)
        .revocation_marker(user_b_revocation_pda)
        .user_tracked_token_account(user_b_tracked_ta)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .tracked_token_program(spl_token_interface::ID)
        .event_authority(event_authority)
        .bump(user_b_bump);
    let opt_in_ix_b = crate::utils::TestInstruction {
        instruction: opt_in_builder_b.instruction(),
        signers: vec![user_b.insecure_clone()],
        name: "ContinuousOptIn",
    };
    opt_in_ix_b.send_expect_success(&mut ctx);

    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, 1_500_000);

    // 4. Distribute 150_000 reward tokens
    let authority_ta = ctx.create_token_account_with_balance(
        &pool_setup.authority.pubkey(),
        &pool_setup.reward_mint.pubkey(),
        1_000_000,
    );

    let mut dist_builder = rewards_program_client::instructions::DistributeContinuousRewardBuilder::new();
    dist_builder
        .authority(pool_setup.authority.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .reward_mint(pool_setup.reward_mint.pubkey())
        .reward_vault(pool_setup.reward_vault)
        .authority_token_account(authority_ta)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority)
        .amount(150_000);
    let dist_ix = crate::utils::TestInstruction {
        instruction: dist_builder.instruction(),
        signers: vec![pool_setup.authority.insecure_clone()],
        name: "DistributeContinuousReward",
    };
    dist_ix.send_expect_success(&mut ctx);

    // 5. User A claims (should get 100_000 = 1_000_000 / 1_500_000 * 150_000)
    let user_a_reward_ta = ctx.create_token_account(&user_a.pubkey(), &pool_setup.reward_mint.pubkey());

    let claim_a = build_claim_continuous_instruction(
        &ctx,
        &pool_setup,
        &user_a,
        &user_a_reward_pda,
        &user_a_tracked_ta,
        &user_a_reward_ta,
        0,
    );
    claim_a.send_expect_success(&mut ctx);

    let user_a_balance = ctx.get_token_balance(&user_a_reward_ta);
    assert_eq!(user_a_balance, 100_000);

    // 6. User B claims (should get 50_000 = 500_000 / 1_500_000 * 150_000)
    let user_b_reward_ta = ctx.create_token_account(&user_b.pubkey(), &pool_setup.reward_mint.pubkey());

    let claim_b = build_claim_continuous_instruction(
        &ctx,
        &pool_setup,
        &user_b,
        &user_b_reward_pda,
        &user_b_tracked_ta,
        &user_b_reward_ta,
        0,
    );
    claim_b.send_expect_success(&mut ctx);

    let user_b_balance = ctx.get_token_balance(&user_b_reward_ta);
    assert_eq!(user_b_balance, 50_000);

    // 7. User A opts out
    let opt_out_a = build_opt_out_instruction(
        &ctx,
        &pool_setup,
        &user_a,
        &user_a_reward_pda,
        &user_a_tracked_ta,
        &user_a_reward_ta,
    );
    opt_out_a.send_expect_success(&mut ctx);
    assert_account_closed(&ctx, &user_a_reward_pda);

    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, 500_000);

    // 8. User B opts out
    let opt_out_b = build_opt_out_instruction(
        &ctx,
        &pool_setup,
        &user_b,
        &user_b_reward_pda,
        &user_b_tracked_ta,
        &user_b_reward_ta,
    );
    opt_out_b.send_expect_success(&mut ctx);
    assert_account_closed(&ctx, &user_b_reward_pda);

    // 9. Close the pool
    let close_ix = build_close_reward_pool_instruction(&ctx, &pool_setup, &authority_ta);
    close_ix.send_expect_success(&mut ctx);
    assert_account_closed(&ctx, &pool_setup.reward_pool_pda);
}

#[test]
fn test_lifecycle_authority_set_balance() {
    let mut ctx = TestContext::new();

    // 1. Create pool in authority-set mode
    let pool_setup = CreateContinuousPoolSetup::new_authority_set(&mut ctx);
    pool_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    // 2. User opts in (initial balance = 0 in authority-set mode)
    let user = ctx.create_funded_keypair();
    let user_tracked_ta = ctx.create_token_account(&user.pubkey(), &pool_setup.tracked_mint.pubkey());
    let (user_reward_pda, user_bump) = find_user_reward_account_pda(&pool_setup.reward_pool_pda, &user.pubkey());

    let (event_authority, _) = crate::utils::find_event_authority_pda();
    let (user_revocation_pda, _) = find_revocation_pda(&pool_setup.reward_pool_pda, &user.pubkey());
    let mut opt_in_builder = rewards_program_client::instructions::ContinuousOptInBuilder::new();
    opt_in_builder
        .payer(ctx.payer.pubkey())
        .user(user.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(user_reward_pda)
        .revocation_marker(user_revocation_pda)
        .user_tracked_token_account(user_tracked_ta)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .tracked_token_program(spl_token_interface::ID)
        .event_authority(event_authority)
        .bump(user_bump);
    let opt_in_ix = crate::utils::TestInstruction {
        instruction: opt_in_builder.instruction(),
        signers: vec![user.insecure_clone()],
        name: "ContinuousOptIn",
    };
    opt_in_ix.send_expect_success(&mut ctx);

    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, 0);

    // 3. Authority sets balance to 1_000_000
    let set_ix = build_set_balance_instruction(&pool_setup, &user.pubkey(), &user_reward_pda, 1_000_000);
    set_ix.send_expect_success(&mut ctx);

    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, 1_000_000);

    let user_account = get_user_reward_account(&ctx, &user_reward_pda);
    assert_eq!(user_account.last_known_balance, 1_000_000);

    // 4. Distribute 100_000 rewards
    let authority_ta = ctx.create_token_account_with_balance(
        &pool_setup.authority.pubkey(),
        &pool_setup.reward_mint.pubkey(),
        1_000_000,
    );

    let mut dist_builder = rewards_program_client::instructions::DistributeContinuousRewardBuilder::new();
    dist_builder
        .authority(pool_setup.authority.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .reward_mint(pool_setup.reward_mint.pubkey())
        .reward_vault(pool_setup.reward_vault)
        .authority_token_account(authority_ta)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority)
        .amount(100_000);
    let dist_ix = crate::utils::TestInstruction {
        instruction: dist_builder.instruction(),
        signers: vec![pool_setup.authority.insecure_clone()],
        name: "DistributeContinuousReward",
    };
    dist_ix.send_expect_success(&mut ctx);

    // 5. User claims all
    let user_reward_ta = ctx.create_token_account(&user.pubkey(), &pool_setup.reward_mint.pubkey());

    let claim_ix = build_claim_continuous_instruction(
        &ctx,
        &pool_setup,
        &user,
        &user_reward_pda,
        &user_tracked_ta,
        &user_reward_ta,
        0,
    );
    claim_ix.send_expect_success(&mut ctx);

    let claimed = ctx.get_token_balance(&user_reward_ta);
    assert_eq!(claimed, 100_000);
}

#[test]
fn test_sync_balance_after_distribution() {
    let mut ctx = TestContext::new();
    let setup = DistributeContinuousRewardSetup::new(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    // User sells half their tokens
    let new_balance = DEFAULT_TRACKED_BALANCE / 2;
    ctx.set_token_balance(&setup.opt_in_setup.user_tracked_token_account, new_balance);

    let sync_ix = build_sync_balance_instruction(
        &setup.opt_in_setup.pool_setup,
        &setup.opt_in_setup.user.pubkey(),
        &setup.opt_in_setup.user_reward_pda,
        &setup.opt_in_setup.user_tracked_token_account,
    );
    sync_ix.send_expect_success(&mut ctx);

    // Rewards should have been settled before balance update
    let user_account = get_user_reward_account(&ctx, &setup.opt_in_setup.user_reward_pda);
    assert_eq!(user_account.last_known_balance, new_balance);
    assert_eq!(user_account.accrued_rewards, DEFAULT_REWARD_AMOUNT);

    let pool = get_reward_pool(&ctx, &setup.opt_in_setup.pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, new_balance);
}
