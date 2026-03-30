use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use crate::fixtures::{InitPointsSetup, IssuePointsSetup};
use crate::utils::{
    assert_account_closed, assert_user_points_balance, find_event_authority_pda, find_points_config_pda,
    find_points_mint_pda, TestContext, TestInstruction,
};

#[test]
fn test_points_full_lifecycle() {
    let mut ctx = TestContext::new();
    let (event_authority, _) = find_event_authority_pda();

    // ── Step 1: Init config (transferable=1, revocable=1) ──
    let init_setup = InitPointsSetup::builder(&mut ctx).transferable(1).revocable(1).build();
    init_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let config_pda = init_setup.points_config_pda;
    let points_mint = init_setup.points_mint_pda;
    let authority = init_setup.authority;

    // ── Step 2: Issue 500 to user A ─────────────────────────────────────────
    let user_a = ctx.create_funded_keypair();
    let user_a_ata =
        get_associated_token_address_with_program_id(&user_a.pubkey(), &points_mint, &TOKEN_2022_PROGRAM_ID);

    let mut builder = rewards_program_client::instructions::IssuePointsBuilder::new();
    builder
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .points_config(config_pda)
        .points_mint(points_mint)
        .user(user_a.pubkey())
        .user_token_account(user_a_ata)
        .token2022_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority)
        .quantity(500);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone()],
        name: "IssuePoints-A",
    }
    .send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &user_a.pubkey(), &points_mint, 500);

    // ── Step 3: Issue 300 to user B ─────────────────────────────────────────
    let user_b = ctx.create_funded_keypair();
    let user_b_ata =
        get_associated_token_address_with_program_id(&user_b.pubkey(), &points_mint, &TOKEN_2022_PROGRAM_ID);

    let mut builder = rewards_program_client::instructions::IssuePointsBuilder::new();
    builder
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .points_config(config_pda)
        .points_mint(points_mint)
        .user(user_b.pubkey())
        .user_token_account(user_b_ata)
        .token2022_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority)
        .quantity(300);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone()],
        name: "IssuePoints-B",
    }
    .send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &user_b.pubkey(), &points_mint, 300);

    // ── Step 4: Transfer 100 from A to B ────────────────────────────────────
    let mut builder = rewards_program_client::instructions::TransferPointsBuilder::new();
    builder
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .from_user(user_a.pubkey())
        .to_user(user_b.pubkey())
        .points_config(config_pda)
        .points_mint(points_mint)
        .from_token_account(user_a_ata)
        .to_token_account(user_b_ata)
        .token2022_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority)
        .quantity(100);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone(), user_a.insecure_clone()],
        name: "TransferPoints",
    }
    .send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &user_a.pubkey(), &points_mint, 400); // 500 - 100
    assert_user_points_balance(&ctx, &user_b.pubkey(), &points_mint, 400); // 300 + 100

    // ── Step 5: Use 200 from A ──────────────────────────────────────────────
    let mut builder = rewards_program_client::instructions::UsePointsBuilder::new();
    builder
        .authority(authority.pubkey())
        .user(user_a.pubkey())
        .points_config(config_pda)
        .points_mint(points_mint)
        .user_token_account(user_a_ata)
        .token2022_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority)
        .quantity(200);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone(), user_a.insecure_clone()],
        name: "UsePoints-A",
    }
    .send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &user_a.pubkey(), &points_mint, 200); // 400 - 200

    // ── Step 6: Revoke B (burns 400) ────────────────────────────────────────
    let mut builder = rewards_program_client::instructions::RevokePointsBuilder::new();
    builder
        .authority(authority.pubkey())
        .points_config(config_pda)
        .points_mint(points_mint)
        .user(user_b.pubkey())
        .user_token_account(user_b_ata)
        .token2022_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone()],
        name: "RevokePoints-B",
    }
    .send_expect_success(&mut ctx);

    // Tokens burned but account still exists
    assert_user_points_balance(&ctx, &user_b.pubkey(), &points_mint, 0);

    // ── Step 7: Use remaining 200 from A, then close A ──────────────────────
    // Advance slot to get a fresh blockhash — step 5 used the same accounts,
    // signers, and quantity so the tx would be deduplicated otherwise.
    ctx.advance_slot();
    let mut builder = rewards_program_client::instructions::UsePointsBuilder::new();
    builder
        .authority(authority.pubkey())
        .user(user_a.pubkey())
        .points_config(config_pda)
        .points_mint(points_mint)
        .user_token_account(user_a_ata)
        .token2022_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority)
        .quantity(200);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone(), user_a.insecure_clone()],
        name: "UsePoints-A-remaining",
    }
    .send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &user_a.pubkey(), &points_mint, 0);

    // Close A's points account (verifies zero balance, emits event)
    let mut builder = rewards_program_client::instructions::ClosePointsAccountBuilder::new();
    builder
        .authority(authority.pubkey())
        .points_config(config_pda)
        .points_mint(points_mint)
        .user(user_a.pubkey())
        .user_token_account(user_a_ata)
        .token2022_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone(), user_a.insecure_clone()],
        name: "ClosePointsAccount-A",
    }
    .send_expect_success(&mut ctx);

    // Also close B's (already zero from revoke)
    let mut builder = rewards_program_client::instructions::ClosePointsAccountBuilder::new();
    builder
        .authority(authority.pubkey())
        .points_config(config_pda)
        .points_mint(points_mint)
        .user(user_b.pubkey())
        .user_token_account(user_b_ata)
        .token2022_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone(), user_b.insecure_clone()],
        name: "ClosePointsAccount-B",
    }
    .send_expect_success(&mut ctx);

    // ── Step 8: Close config ────────────────────────────────────────────────
    let mut builder = rewards_program_client::instructions::ClosePointsConfigBuilder::new();
    builder
        .authority(authority.pubkey())
        .points_config(config_pda)
        .points_mint(points_mint)
        .destination(ctx.payer.pubkey())
        .token2022_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority);
    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone()],
        name: "ClosePointsConfig",
    }
    .send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &config_pda);
}

#[test]
fn test_points_multiple_configs_same_authority() {
    let mut ctx = TestContext::new();

    let authority = ctx.create_funded_keypair();

    // Create first config with seed1
    let seed1 = Keypair::new();
    let (config_pda1, bump1) = find_points_config_pda(&authority.pubkey(), &seed1.pubkey());
    let (mint_pda1, mint_bump1) = find_points_mint_pda(&config_pda1);

    let setup1 = InitPointsSetup {
        authority: authority.insecure_clone(),
        seed: seed1,
        points_config_pda: config_pda1,
        points_mint_pda: mint_pda1,
        bump: bump1,
        mint_bump: mint_bump1,
        transferable: 0,
        revocable: 0,
    };
    setup1.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Create second config with seed2 (same authority)
    let seed2 = Keypair::new();
    let (config_pda2, bump2) = find_points_config_pda(&authority.pubkey(), &seed2.pubkey());
    let (mint_pda2, mint_bump2) = find_points_mint_pda(&config_pda2);

    let setup2 = InitPointsSetup {
        authority: authority.insecure_clone(),
        seed: seed2,
        points_config_pda: config_pda2,
        points_mint_pda: mint_pda2,
        bump: bump2,
        mint_bump: mint_bump2,
        transferable: 1,
        revocable: 1,
    };
    setup2.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Both configs should exist independently
    assert!(ctx.get_account(&config_pda1).is_some(), "Config 1 should exist");
    assert!(ctx.get_account(&config_pda2).is_some(), "Config 2 should exist");
    assert!(ctx.get_account(&mint_pda1).is_some(), "Mint 1 should exist");
    assert!(ctx.get_account(&mint_pda2).is_some(), "Mint 2 should exist");
    assert_ne!(config_pda1, config_pda2, "Config PDAs should differ");

    // Issue points on both configs to different users
    let user1 = Keypair::new();
    let user1_ata = get_associated_token_address_with_program_id(&user1.pubkey(), &mint_pda1, &TOKEN_2022_PROGRAM_ID);

    let issue1 = IssuePointsSetup {
        authority: authority.insecure_clone(),
        points_config_pda: config_pda1,
        points_mint_pda: mint_pda1,
        user: user1.pubkey(),
        user_ata: user1_ata,
        quantity: 100,
    };
    issue1.build_instruction(&ctx).send_expect_success(&mut ctx);

    let user2 = Keypair::new();
    let user2_ata = get_associated_token_address_with_program_id(&user2.pubkey(), &mint_pda2, &TOKEN_2022_PROGRAM_ID);

    let issue2 = IssuePointsSetup {
        authority: authority.insecure_clone(),
        points_config_pda: config_pda2,
        points_mint_pda: mint_pda2,
        user: user2.pubkey(),
        user_ata: user2_ata,
        quantity: 200,
    };
    issue2.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Verify balances are independent
    assert_user_points_balance(&ctx, &user1.pubkey(), &mint_pda1, 100);
    assert_user_points_balance(&ctx, &user2.pubkey(), &mint_pda2, 200);
}
