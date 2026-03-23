use solana_sdk::signature::{Keypair, Signer};

use crate::fixtures::InitPointsSetup;
use crate::utils::{
    assert_account_closed, assert_user_points_balance, find_event_authority_pda, find_user_points_pda,
    get_points_config, TestContext, TestInstruction,
};

#[test]
fn test_points_full_lifecycle() {
    let mut ctx = TestContext::new();
    let (event_authority, _) = find_event_authority_pda();

    // ── Step 1: Init config (transferable=1, revocable=1, max_supply=1000) ──
    let init_setup = InitPointsSetup::builder(&mut ctx).transferable(1).revocable(1).max_supply(1_000).build();
    init_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let config_pda = init_setup.points_config_pda;
    let authority = init_setup.authority;

    // ── Step 2: Issue 500 to user A ─────────────────────────────────────────
    let user_a = ctx.create_funded_keypair();
    let (user_a_pda, user_a_bump) = find_user_points_pda(&config_pda, &user_a.pubkey());

    let mut builder = rewards_program_client::instructions::IssuePointsBuilder::new();
    builder
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .points_config(config_pda)
        .user(user_a.pubkey())
        .user_points_account(user_a_pda)
        .event_authority(event_authority)
        .user_points_bump(user_a_bump)
        .quantity(500);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone()],
        name: "IssuePoints-A",
    }
    .send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &user_a_pda, 500);

    // ── Step 3: Issue 300 to user B ─────────────────────────────────────────
    let user_b = Keypair::new();
    let (user_b_pda, user_b_bump) = find_user_points_pda(&config_pda, &user_b.pubkey());

    let mut builder = rewards_program_client::instructions::IssuePointsBuilder::new();
    builder
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .points_config(config_pda)
        .user(user_b.pubkey())
        .user_points_account(user_b_pda)
        .event_authority(event_authority)
        .user_points_bump(user_b_bump)
        .quantity(300);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone()],
        name: "IssuePoints-B",
    }
    .send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &user_b_pda, 300);

    let config = get_points_config(&ctx, &config_pda);
    assert_eq!(config.total_issued, 800);
    assert_eq!(config.total_used, 0);

    // ── Step 4: Transfer 100 from A to B ────────────────────────────────────
    let mut builder = rewards_program_client::instructions::TransferPointsBuilder::new();
    builder
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .from_user(user_a.pubkey())
        .points_config(config_pda)
        .from_user_points(user_a_pda)
        .to_user(user_b.pubkey())
        .to_user_points(user_b_pda)
        .event_authority(event_authority)
        .to_user_points_bump(user_b_bump)
        .quantity(100);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone(), user_a.insecure_clone()],
        name: "TransferPoints",
    }
    .send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &user_a_pda, 400); // 500 - 100
    assert_user_points_balance(&ctx, &user_b_pda, 400); // 300 + 100

    // ── Step 5: Use 200 from A ──────────────────────────────────────────────
    let mut builder = rewards_program_client::instructions::UsePointsBuilder::new();
    builder
        .authority(authority.pubkey())
        .user(user_a.pubkey())
        .points_config(config_pda)
        .user_points_account(user_a_pda)
        .event_authority(event_authority)
        .quantity(200);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone(), user_a.insecure_clone()],
        name: "UsePoints-A",
    }
    .send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &user_a_pda, 200); // 400 - 200

    let config = get_points_config(&ctx, &config_pda);
    assert_eq!(config.total_used, 200);

    // ── Step 6: Revoke B (burns 400, closes account) ────────────────────────
    let mut builder = rewards_program_client::instructions::RevokePointsBuilder::new();
    builder
        .authority(authority.pubkey())
        .points_config(config_pda)
        .user(user_b.pubkey())
        .user_points_account(user_b_pda)
        .destination(ctx.payer.pubkey())
        .event_authority(event_authority);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone()],
        name: "RevokePoints-B",
    }
    .send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &user_b_pda);

    let config = get_points_config(&ctx, &config_pda);
    assert_eq!(config.total_used, 600); // 200 + 400 revoked

    // ── Step 7: Use remaining 200 from A, then close A ──────────────────────
    // Advance slot to get a fresh blockhash — step 5 used the same accounts,
    // signers, and quantity so the tx would be deduplicated otherwise.
    ctx.advance_slot();
    let mut builder = rewards_program_client::instructions::UsePointsBuilder::new();
    builder
        .authority(authority.pubkey())
        .user(user_a.pubkey())
        .points_config(config_pda)
        .user_points_account(user_a_pda)
        .event_authority(event_authority)
        .quantity(200);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone(), user_a.insecure_clone()],
        name: "UsePoints-A-remaining",
    }
    .send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &user_a_pda, 0);

    // Close A's account
    let mut builder = rewards_program_client::instructions::ClosePointsAccountBuilder::new();
    builder
        .authority(authority.pubkey())
        .points_config(config_pda)
        .user(user_a.pubkey())
        .user_points_account(user_a_pda)
        .destination(ctx.payer.pubkey())
        .event_authority(event_authority);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone()],
        name: "ClosePointsAccount-A",
    }
    .send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &user_a_pda);

    // ── Step 8: Close config ────────────────────────────────────────────────
    let config = get_points_config(&ctx, &config_pda);
    assert_eq!(config.total_issued, 800);
    assert_eq!(config.total_used, 800); // 200 + 200 + 400

    let mut builder = rewards_program_client::instructions::ClosePointsConfigBuilder::new();
    builder
        .authority(authority.pubkey())
        .points_config(config_pda)
        .destination(ctx.payer.pubkey())
        .event_authority(event_authority);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone()],
        name: "ClosePointsConfig",
    }
    .send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &config_pda);
}
