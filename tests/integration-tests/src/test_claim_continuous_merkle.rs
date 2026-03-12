use crate::fixtures::{
    build_claim_continuous_instruction, build_claim_continuous_merkle_instruction,
    build_set_continuous_merkle_root_instruction, ClaimContinuousMerkleFixture, ClaimContinuousMerkleSetup,
    ContinuousOptInSetup, CreateContinuousPoolSetup, DistributeContinuousRewardSetup, RevokeContinuousUserSetup,
    DEFAULT_REWARD_AMOUNT,
};
use crate::utils::{
    assert_merkle_claim, assert_rewards_error, find_merkle_claim_pda, get_reward_pool, test_empty_data,
    test_missing_signer, test_not_writable, test_truncated_data, test_wrong_current_program, test_wrong_system_program,
    ContinuousMerkleLeaf, ContinuousMerkleTree, RewardsError, TestContext,
};
use rewards_program_client::types::RevokeMode;
use solana_sdk::signature::Signer;

#[test]
fn test_claim_continuous_merkle_missing_user_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<ClaimContinuousMerkleFixture>(&mut ctx, 1, 0);
}

#[test]
fn test_claim_continuous_merkle_pool_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimContinuousMerkleFixture>(&mut ctx, 2);
}

#[test]
fn test_claim_continuous_merkle_claim_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimContinuousMerkleFixture>(&mut ctx, 3);
}

#[test]
fn test_claim_continuous_merkle_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimContinuousMerkleFixture>(&mut ctx, 6);
}

#[test]
fn test_claim_continuous_merkle_user_reward_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimContinuousMerkleFixture>(&mut ctx, 7);
}

#[test]
fn test_claim_continuous_merkle_wrong_system_program() {
    let mut ctx = TestContext::new();
    test_wrong_system_program::<ClaimContinuousMerkleFixture>(&mut ctx);
}

#[test]
fn test_claim_continuous_merkle_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<ClaimContinuousMerkleFixture>(&mut ctx);
}

#[test]
fn test_claim_continuous_merkle_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<ClaimContinuousMerkleFixture>(&mut ctx);
}

#[test]
fn test_claim_continuous_merkle_truncated_data() {
    let mut ctx = TestContext::new();
    test_truncated_data::<ClaimContinuousMerkleFixture>(&mut ctx);
}

#[test]
fn test_claim_continuous_merkle_success_full() {
    let mut ctx = TestContext::new();
    let setup = ClaimContinuousMerkleSetup::new(&mut ctx);

    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let user_balance = ctx.get_token_balance(&setup.user_reward_token_account);
    assert_eq!(user_balance, setup.cumulative_amount);

    assert_merkle_claim(&ctx, &setup.claim_pda, setup.cumulative_amount, setup.claim_bump);

    let pool = get_reward_pool(&ctx, &setup.distribute_setup.opt_in_setup.pool_setup.reward_pool_pda);
    assert_eq!(pool.total_claimed, setup.cumulative_amount);
}

#[test]
fn test_claim_continuous_merkle_partial_then_full() {
    let mut ctx = TestContext::new();
    let setup = ClaimContinuousMerkleSetup::new(&mut ctx);

    let partial = setup.cumulative_amount / 2;
    setup.build_instruction_with_amount(&ctx, partial).send_expect_success(&mut ctx);

    assert_merkle_claim(&ctx, &setup.claim_pda, partial, setup.claim_bump);

    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let user_balance = ctx.get_token_balance(&setup.user_reward_token_account);
    assert_eq!(user_balance, setup.cumulative_amount);
    assert_merkle_claim(&ctx, &setup.claim_pda, setup.cumulative_amount, setup.claim_bump);
}

#[test]
fn test_claim_continuous_merkle_amount_exceeds_available() {
    let mut ctx = TestContext::new();
    let setup = ClaimContinuousMerkleSetup::new(&mut ctx);

    let too_much = setup.cumulative_amount.checked_add(1).expect("test amount overflow");
    let error = setup.build_instruction_with_amount(&ctx, too_much).send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::ExceedsClaimableAmount);
}

#[test]
fn test_claim_continuous_merkle_reclaim_after_full_fails() {
    let mut ctx = TestContext::new();
    let setup = ClaimContinuousMerkleSetup::new(&mut ctx);

    setup.build_instruction(&ctx).send_expect_success(&mut ctx);
    ctx.advance_slot();

    let error = setup.build_instruction(&ctx).send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::NothingToClaim);
}

#[test]
fn test_claim_continuous_merkle_invalid_proof() {
    let mut ctx = TestContext::new();
    let setup = ClaimContinuousMerkleSetup::new(&mut ctx);

    let pool_setup = &setup.distribute_setup.opt_in_setup.pool_setup;
    let user = &setup.distribute_setup.opt_in_setup.user;

    let ix = build_claim_continuous_merkle_instruction(
        &ctx,
        pool_setup,
        user,
        setup.claim_pda,
        setup.claim_bump,
        setup.user_reward_token_account,
        setup.epoch,
        setup.cumulative_amount,
        0,
        vec![[99u8; 32]],
    );

    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InvalidMerkleProof);
}

#[test]
fn test_claim_continuous_merkle_wrong_epoch() {
    let mut ctx = TestContext::new();
    let setup = ClaimContinuousMerkleSetup::new(&mut ctx);

    let pool_setup = &setup.distribute_setup.opt_in_setup.pool_setup;
    let user = &setup.distribute_setup.opt_in_setup.user;

    let ix = build_claim_continuous_merkle_instruction(
        &ctx,
        pool_setup,
        user,
        setup.claim_pda,
        setup.claim_bump,
        setup.user_reward_token_account,
        setup.epoch + 1,
        setup.cumulative_amount,
        0,
        setup.proof.clone(),
    );

    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::MerkleRootEpochMismatch);
}

#[test]
fn test_claim_continuous_merkle_root_not_set() {
    let mut ctx = TestContext::new();
    let setup = DistributeContinuousRewardSetup::new(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let pool_setup = &setup.opt_in_setup.pool_setup;
    let user = &setup.opt_in_setup.user;
    let (claim_pda, claim_bump) =
        crate::utils::find_merkle_claim_pda(&setup.opt_in_setup.pool_setup.reward_pool_pda, &user.pubkey());
    let user_reward_token_account = ctx.create_token_account(&user.pubkey(), &pool_setup.reward_mint.pubkey());

    let ix = build_claim_continuous_merkle_instruction(
        &ctx,
        pool_setup,
        user,
        claim_pda,
        claim_bump,
        user_reward_token_account,
        1,
        DEFAULT_REWARD_AMOUNT,
        0,
        vec![],
    );

    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::MerkleRootNotSet);
}

#[test]
fn test_claim_continuous_merkle_user_revoked() {
    let mut ctx = TestContext::new();
    let setup = RevokeContinuousUserSetup::new(&mut ctx);

    let pool_setup = &setup.distribute_setup.opt_in_setup.pool_setup;
    let user = &setup.distribute_setup.opt_in_setup.user;
    let other_claimant = ctx.create_funded_keypair();
    let epoch = 1;
    let cumulative_amount = DEFAULT_REWARD_AMOUNT;

    // Revoke before enabling merkle mode; revocation instructions are blocked afterward.
    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::Full);
    revoke_ix.send_expect_success(&mut ctx);

    let merkle_tree = ContinuousMerkleTree::new(vec![
        ContinuousMerkleLeaf::new(pool_setup.reward_pool_pda, user.pubkey(), epoch, cumulative_amount),
        ContinuousMerkleLeaf::new(pool_setup.reward_pool_pda, other_claimant.pubkey(), epoch, cumulative_amount / 2),
    ]);
    build_set_continuous_merkle_root_instruction(pool_setup, merkle_tree.root, epoch).send_expect_success(&mut ctx);

    let (claim_pda, claim_bump) = find_merkle_claim_pda(&pool_setup.reward_pool_pda, &user.pubkey());
    let proof = merkle_tree.get_proof_for_claimant(&user.pubkey()).unwrap_or_default();
    let claim_ix = build_claim_continuous_merkle_instruction(
        &ctx,
        pool_setup,
        user,
        claim_pda,
        claim_bump,
        setup.user_reward_token_account,
        epoch,
        cumulative_amount,
        0,
        proof,
    );

    let error = claim_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UserRevoked);
}

#[test]
fn test_claim_continuous_merkle_rotation_delta_claim() {
    let mut ctx = TestContext::new();
    let setup = ClaimContinuousMerkleSetup::new(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Increase distributed rewards so epoch 2 can claim additional cumulative balance.
    ctx.advance_slot();
    setup.distribute_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let pool_setup = &setup.distribute_setup.opt_in_setup.pool_setup;
    let user = &setup.distribute_setup.opt_in_setup.user;
    let other_claimant = ctx.create_funded_keypair();
    let next_epoch = setup.epoch + 1;
    let next_cumulative = setup.cumulative_amount + (DEFAULT_REWARD_AMOUNT / 2);

    let next_tree = ContinuousMerkleTree::new(vec![
        ContinuousMerkleLeaf::new(pool_setup.reward_pool_pda, user.pubkey(), next_epoch, next_cumulative),
        ContinuousMerkleLeaf::new(
            pool_setup.reward_pool_pda,
            other_claimant.pubkey(),
            next_epoch,
            DEFAULT_REWARD_AMOUNT / 2,
        ),
    ]);

    build_set_continuous_merkle_root_instruction(pool_setup, next_tree.root, next_epoch).send_expect_success(&mut ctx);

    let next_proof = next_tree.get_proof_for_claimant(&user.pubkey()).unwrap_or_default();
    let claim_ix = build_claim_continuous_merkle_instruction(
        &ctx,
        pool_setup,
        user,
        setup.claim_pda,
        setup.claim_bump,
        setup.user_reward_token_account,
        next_epoch,
        next_cumulative,
        0,
        next_proof,
    );
    claim_ix.send_expect_success(&mut ctx);

    let user_balance = ctx.get_token_balance(&setup.user_reward_token_account);
    assert_eq!(user_balance, next_cumulative);
    assert_merkle_claim(&ctx, &setup.claim_pda, next_cumulative, setup.claim_bump);

    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    assert_eq!(pool.total_claimed, next_cumulative);
}

#[test]
fn test_claim_continuous_merkle_rotation_decreased_cumulative_fails() {
    let mut ctx = TestContext::new();
    let setup = ClaimContinuousMerkleSetup::new(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let pool_setup = &setup.distribute_setup.opt_in_setup.pool_setup;
    let user = &setup.distribute_setup.opt_in_setup.user;
    let other_claimant = ctx.create_funded_keypair();
    let next_epoch = setup.epoch + 1;
    let decreased_cumulative =
        setup.cumulative_amount.checked_sub(1).expect("cumulative amount should be greater than zero");

    let next_tree = ContinuousMerkleTree::new(vec![
        ContinuousMerkleLeaf::new(pool_setup.reward_pool_pda, user.pubkey(), next_epoch, decreased_cumulative),
        ContinuousMerkleLeaf::new(pool_setup.reward_pool_pda, other_claimant.pubkey(), next_epoch, 1),
    ]);

    build_set_continuous_merkle_root_instruction(pool_setup, next_tree.root, next_epoch).send_expect_success(&mut ctx);

    let next_proof = next_tree.get_proof_for_claimant(&user.pubkey()).unwrap_or_default();
    let claim_ix = build_claim_continuous_merkle_instruction(
        &ctx,
        pool_setup,
        user,
        setup.claim_pda,
        setup.claim_bump,
        setup.user_reward_token_account,
        next_epoch,
        decreased_cumulative,
        0,
        next_proof,
    );
    let error = claim_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::ClaimedAmountDecreased);
}

#[test]
fn test_claim_continuous_merkle_insufficient_pool_funds() {
    let mut ctx = TestContext::new();
    let setup = ClaimContinuousMerkleSetup::new(&mut ctx);

    let pool_setup = &setup.distribute_setup.opt_in_setup.pool_setup;
    let user = &setup.distribute_setup.opt_in_setup.user;
    let other_claimant = ctx.create_funded_keypair();
    let next_epoch = setup.epoch + 1;
    let too_large_cumulative = setup.distribute_setup.amount.checked_add(1).expect("test amount overflow");

    let next_tree = ContinuousMerkleTree::new(vec![
        ContinuousMerkleLeaf::new(pool_setup.reward_pool_pda, user.pubkey(), next_epoch, too_large_cumulative),
        ContinuousMerkleLeaf::new(pool_setup.reward_pool_pda, other_claimant.pubkey(), next_epoch, 1),
    ]);

    build_set_continuous_merkle_root_instruction(pool_setup, next_tree.root, next_epoch).send_expect_success(&mut ctx);

    let next_proof = next_tree.get_proof_for_claimant(&user.pubkey()).unwrap_or_default();
    let claim_ix = build_claim_continuous_merkle_instruction(
        &ctx,
        pool_setup,
        user,
        setup.claim_pda,
        setup.claim_bump,
        setup.user_reward_token_account,
        next_epoch,
        too_large_cumulative,
        0,
        next_proof,
    );
    let error = claim_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InsufficientFunds);
}

#[test]
fn test_claim_continuous_merkle_success_token_2022() {
    let mut ctx = TestContext::new();
    let pool_setup = CreateContinuousPoolSetup::new_token_2022(&mut ctx);
    pool_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let opt_in_setup = ContinuousOptInSetup::new_from_pool(&mut ctx, pool_setup);
    opt_in_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let authority_token_account = ctx.create_ata_for_program_with_balance(
        &opt_in_setup.pool_setup.authority.pubkey(),
        &opt_in_setup.pool_setup.reward_mint.pubkey(),
        DEFAULT_REWARD_AMOUNT * 10,
        &opt_in_setup.pool_setup.reward_token_program,
    );
    let distribute_setup =
        DistributeContinuousRewardSetup { opt_in_setup, authority_token_account, amount: DEFAULT_REWARD_AMOUNT };
    distribute_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let pool_setup = &distribute_setup.opt_in_setup.pool_setup;
    let user = &distribute_setup.opt_in_setup.user;
    let other_claimant = ctx.create_funded_keypair();
    let epoch = 1;
    let cumulative_amount = DEFAULT_REWARD_AMOUNT;
    let merkle_tree = ContinuousMerkleTree::new(vec![
        ContinuousMerkleLeaf::new(pool_setup.reward_pool_pda, user.pubkey(), epoch, cumulative_amount),
        ContinuousMerkleLeaf::new(pool_setup.reward_pool_pda, other_claimant.pubkey(), epoch, cumulative_amount / 2),
    ]);

    build_set_continuous_merkle_root_instruction(pool_setup, merkle_tree.root, epoch).send_expect_success(&mut ctx);

    let (claim_pda, claim_bump) = find_merkle_claim_pda(&pool_setup.reward_pool_pda, &user.pubkey());
    let proof = merkle_tree.get_proof_for_claimant(&user.pubkey()).unwrap_or_default();
    let user_reward_token_account =
        ctx.create_ata_for_program(&user.pubkey(), &pool_setup.reward_mint.pubkey(), &pool_setup.reward_token_program);

    let claim_ix = build_claim_continuous_merkle_instruction(
        &ctx,
        pool_setup,
        user,
        claim_pda,
        claim_bump,
        user_reward_token_account,
        epoch,
        cumulative_amount,
        0,
        proof,
    );
    claim_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&user_reward_token_account);
    assert_eq!(balance, cumulative_amount);
    assert_merkle_claim(&ctx, &claim_pda, cumulative_amount, claim_bump);
}

#[test]
fn test_claim_continuous_disabled_when_merkle_mode_enabled() {
    let mut ctx = TestContext::new();
    let setup = ClaimContinuousMerkleSetup::new(&mut ctx);

    let pool_setup = &setup.distribute_setup.opt_in_setup.pool_setup;
    let user = &setup.distribute_setup.opt_in_setup.user;
    let user_reward_pda = &setup.distribute_setup.opt_in_setup.user_reward_pda;
    let user_tracked_ta = &setup.distribute_setup.opt_in_setup.user_tracked_token_account;

    let claim_ix = build_claim_continuous_instruction(
        &ctx,
        pool_setup,
        user,
        user_reward_pda,
        user_tracked_ta,
        &setup.user_reward_token_account,
        0,
    );

    let error = claim_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::ContinuousMerkleModeEnabled);
}
