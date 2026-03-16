/// End-to-end integration tests for continuous reward pools with `confidential_rewards = 1`.
///
/// Test progression:
///   1. CT account configuration (VerifyPubkeyValidity + ConfigureAccount round-trip)
///   2. Opt-in to a confidential pool with a properly configured reward ATA
///   3. Distribute rewards (TransferChecked → ConfidentialDeposit → ConfidentialApplyPendingBalance)
///   4. Full claim flow with pre-verified proof context state accounts
use bytemuck::bytes_of;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_zk_sdk::{
    encryption::{
        auth_encryption::{AeCiphertext, AeKey},
        elgamal::ElGamalKeypair,
    },
    zk_elgamal_proof_program::{
        instruction::ProofInstruction,
        proof_data::{
            BatchedGroupedCiphertext3HandlesValidityProofContext, BatchedRangeProofContext,
            CiphertextCommitmentEqualityProofContext,
        },
    },
};
use spl_token_confidential_transfer_proof_generation::transfer::transfer_split_proof_data;

use solana_zk_sdk::encryption::pod::elgamal::PodElGamalCiphertext;
use spl_token_2022_interface::pod::PodAccount;

use crate::fixtures::{
    confidential::{create_and_configure_ct_account, create_ct_mint, create_ct_vault, create_proof_context_state},
    CreateContinuousPoolSetup, DEFAULT_REWARD_AMOUNT,
};
use crate::utils::{
    find_event_authority_pda, find_revocation_pda, find_user_reward_account_pda, get_reward_pool, TestContext,
    TOKEN_2022_PROGRAM_ID,
};

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Build a confidential pool setup: CT reward mint, `confidential_rewards = 1`.
///
/// Returns `(pool_setup, vault_elgamal_kp, vault_aes_key, initial_enc)`.
/// `initial_enc` is the `Enc(0, r0)` seeded into the vault at creation — use it to
/// derive the vault's available_balance after distribute:
/// `vault_available_ct = initial_enc + ElGamal::encode(distributed_amount)`.
fn make_confidential_pool(
    ctx: &mut TestContext,
) -> (CreateContinuousPoolSetup, ElGamalKeypair, AeKey, solana_zk_sdk::encryption::elgamal::ElGamalCiphertext) {
    let mut setup = CreateContinuousPoolSetup::new(ctx);
    let ct_reward_mint = Keypair::new();
    create_ct_mint(ctx, &ct_reward_mint, &setup.authority.pubkey());

    let (reward_pool_pda, bump) = crate::utils::find_reward_pool_pda(
        &ct_reward_mint.pubkey(),
        &setup.tracked_mint.pubkey(),
        &setup.authority.pubkey(),
        &setup.seed.pubkey(),
    );
    let (reward_vault, vault_elgamal_kp, vault_aes_key, initial_enc) =
        create_ct_vault(ctx, &reward_pool_pda, &ct_reward_mint.pubkey());

    setup.reward_mint = ct_reward_mint;
    setup.reward_pool_pda = reward_pool_pda;
    setup.bump = bump;
    setup.reward_vault = reward_vault;
    setup.reward_token_program = TOKEN_2022_PROGRAM_ID;
    setup.confidential_rewards = 1;
    (setup, vault_elgamal_kp, vault_aes_key, initial_enc)
}

// ─── Test 1: CT account configuration ────────────────────────────────────────

#[test]
fn test_configure_ct_account_succeeds() {
    let mut ctx = TestContext::new();
    let mint = Keypair::new();
    let payer_pk = ctx.payer.pubkey();
    create_ct_mint(&mut ctx, &mint, &payer_pk);

    let user = ctx.create_funded_keypair();
    let (ata, _elgamal_kp, _aes_key) = create_and_configure_ct_account(&mut ctx, &user, &mint.pubkey());

    // Verify the ATA now has the ConfidentialTransferAccount extension (data > 165 bytes).
    let acc = ctx.svm.get_account(&ata).expect("ATA should exist after configure");
    assert!(acc.data.len() > 165, "CT ATA must be larger than base 165-byte token account");
    assert_eq!(acc.owner, TOKEN_2022_PROGRAM_ID);
}

// ─── Test 2: Opt-in with configured reward ATA ───────────────────────────────

#[test]
fn test_opt_in_confidential_pool_with_configured_ata() {
    let mut ctx = TestContext::new();
    let (pool_setup, _vault_elgamal_kp, _vault_aes_key, _initial_enc) = make_confidential_pool(&mut ctx);
    pool_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let user = ctx.create_funded_keypair();
    let user_tracked_ta =
        ctx.create_token_account_with_balance(&user.pubkey(), &pool_setup.tracked_mint.pubkey(), 1_000_000);
    let (user_reward_pda, user_reward_bump) = find_user_reward_account_pda(&pool_setup.reward_pool_pda, &user.pubkey());

    // Configure the user's reward ATA for confidential transfers.
    let (user_reward_ata, _elgamal_kp, _aes_key) =
        create_and_configure_ct_account(&mut ctx, &user, &pool_setup.reward_mint.pubkey());

    // Build opt-in with the CT ATA appended as account[11].
    let (event_authority, _) = find_event_authority_pda();
    let (revocation_pda, _) = find_revocation_pda(&pool_setup.reward_pool_pda, &user.pubkey());
    let mut builder = rewards_program_client::instructions::ContinuousOptInBuilder::new();
    builder
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
    let mut opt_in_ix = builder.instruction();
    opt_in_ix.accounts.push(solana_sdk::instruction::AccountMeta {
        pubkey: user_reward_ata,
        is_signer: false,
        is_writable: false,
    });

    let blockhash = ctx.svm.latest_blockhash();
    let tx =
        Transaction::new_signed_with_payer(&[opt_in_ix], Some(&ctx.payer.pubkey()), &[&ctx.payer, &user], blockhash);
    ctx.svm.send_transaction(tx).expect("opt-in with configured CT ATA should succeed");

    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, 1_000_000);
    assert_eq!(pool.confidential_rewards, 1);
}

// ─── Test 3: Distribute on confidential pool ─────────────────────────────────

#[test]
fn test_distribute_confidential_pool() {
    let mut ctx = TestContext::new();
    let (pool_setup, _vault_elgamal_kp, vault_aes_key, _initial_enc) = make_confidential_pool(&mut ctx);
    pool_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Opt a user in so opted_in_supply > 0 (distribute rejects zero supply).
    let user = ctx.create_funded_keypair();
    let user_tracked_ta =
        ctx.create_token_account_with_balance(&user.pubkey(), &pool_setup.tracked_mint.pubkey(), 1_000_000);
    let (user_reward_pda, user_reward_bump) = find_user_reward_account_pda(&pool_setup.reward_pool_pda, &user.pubkey());
    let (user_reward_ata, _, _) = create_and_configure_ct_account(&mut ctx, &user, &pool_setup.reward_mint.pubkey());

    let (event_authority, _) = find_event_authority_pda();
    let (revocation_pda, _) = find_revocation_pda(&pool_setup.reward_pool_pda, &user.pubkey());
    let mut builder = rewards_program_client::instructions::ContinuousOptInBuilder::new();
    builder
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
    let mut opt_in_ix = builder.instruction();
    opt_in_ix.accounts.push(solana_sdk::instruction::AccountMeta {
        pubkey: user_reward_ata,
        is_signer: false,
        is_writable: false,
    });
    let bh = ctx.svm.latest_blockhash();
    ctx.svm
        .send_transaction(Transaction::new_signed_with_payer(
            &[opt_in_ix],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer, &user],
            bh,
        ))
        .expect("opt-in");

    // Distribute rewards. For a confidential pool this does TransferChecked then ConfidentialDeposit.
    // The vault must already be configured for CT; if the mint has auto_approve=true AND the vault
    // was created as a Token-2022 ATA for a CT mint, Token-2022 auto-initialises the CT extension.
    let authority_ta = ctx.create_ata_for_program_with_balance(
        &pool_setup.authority.pubkey(),
        &pool_setup.reward_mint.pubkey(),
        DEFAULT_REWARD_AMOUNT * 10,
        &TOKEN_2022_PROGRAM_ID,
    );

    let mut vault_state = rewards_program_client::confidential_helpers::ConfidentialVaultState::new(_initial_enc);
    let (decryptable, _) = vault_state.prepare_distribute(DEFAULT_REWARD_AMOUNT, &vault_aes_key);

    let dist_ix = rewards_program_client::instructions::DistributeContinuousRewardBuilder::new()
        .authority(pool_setup.authority.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .reward_mint(pool_setup.reward_mint.pubkey())
        .reward_vault(pool_setup.reward_vault)
        .authority_token_account(authority_ta)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority)
        .amount(DEFAULT_REWARD_AMOUNT)
        .expected_pending_balance_credit_counter(1)
        .new_decryptable_available_balance(decryptable);

    let bh = ctx.svm.latest_blockhash();
    ctx.svm
        .send_transaction(Transaction::new_signed_with_payer(
            &[dist_ix],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer, &pool_setup.authority],
            bh,
        ))
        .expect("distribute should succeed");

    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    assert!(pool.total_distributed > 0, "pool should have recorded distribution");
}

// ─── Test 4: Full confidential claim flow ────────────────────────────────────

/// Full end-to-end: configure → opt-in → distribute → apply_pending_vault → claim with proofs.
///
/// This test requires:
///  1. The vault to receive a ConfidentialDeposit (distribute step)
///  2. The vault's pending balance to be applied (requires vault ElGamal key knowledge)
///  3. Three pre-verified proof context state accounts for the ConfidentialTransfer claim
///
/// Prerequisites from tests 1–3 must all pass first.
#[test]
fn test_claim_confidential_full_flow() {
    let mut ctx = TestContext::new();
    let (pool_setup, vault_elgamal_kp, vault_aes_key, initial_vault_enc) = make_confidential_pool(&mut ctx);
    pool_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    // ── 1. User opts in with a configured CT reward ATA ──────────────────────
    let user = ctx.create_funded_keypair();
    let user_tracked_ta =
        ctx.create_token_account_with_balance(&user.pubkey(), &pool_setup.tracked_mint.pubkey(), 1_000_000);
    let (user_reward_pda, user_reward_bump) = find_user_reward_account_pda(&pool_setup.reward_pool_pda, &user.pubkey());
    let (user_reward_ata, user_elgamal_kp, _user_aes_key) =
        create_and_configure_ct_account(&mut ctx, &user, &pool_setup.reward_mint.pubkey());

    let (event_authority, _) = find_event_authority_pda();
    let (revocation_pda, _) = find_revocation_pda(&pool_setup.reward_pool_pda, &user.pubkey());
    let mut opt_in_builder = rewards_program_client::instructions::ContinuousOptInBuilder::new();
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
    let mut opt_in_ix = opt_in_builder.instruction();
    opt_in_ix.accounts.push(solana_sdk::instruction::AccountMeta {
        pubkey: user_reward_ata,
        is_signer: false,
        is_writable: false,
    });
    let bh = ctx.svm.latest_blockhash();
    ctx.svm
        .send_transaction(Transaction::new_signed_with_payer(
            &[opt_in_ix],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer, &user],
            bh,
        ))
        .expect("opt-in");

    // ── 2. Distribute rewards ─────────────────────────────────────────────────
    let authority_ta = ctx.create_ata_for_program_with_balance(
        &pool_setup.authority.pubkey(),
        &pool_setup.reward_mint.pubkey(),
        DEFAULT_REWARD_AMOUNT * 10,
        &TOKEN_2022_PROGRAM_ID,
    );

    let mut vault_state = rewards_program_client::confidential_helpers::ConfidentialVaultState::new(initial_vault_enc);
    let (decryptable_bytes, _vault_ct_after) = vault_state.prepare_distribute(DEFAULT_REWARD_AMOUNT, &vault_aes_key);
    let vault_decryptable: AeCiphertext = vault_aes_key.encrypt(DEFAULT_REWARD_AMOUNT);

    let dist_ix = rewards_program_client::instructions::DistributeContinuousRewardBuilder::new()
        .authority(pool_setup.authority.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .reward_mint(pool_setup.reward_mint.pubkey())
        .reward_vault(pool_setup.reward_vault)
        .authority_token_account(authority_ta)
        .reward_token_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority)
        .amount(DEFAULT_REWARD_AMOUNT)
        .expected_pending_balance_credit_counter(1)
        .new_decryptable_available_balance(decryptable_bytes);

    let bh = ctx.svm.latest_blockhash();
    ctx.svm
        .send_transaction(Transaction::new_signed_with_payer(
            &[dist_ix],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer, &pool_setup.authority],
            bh,
        ))
        .expect("distribute");

    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    let claim_amount = pool.total_distributed;
    assert!(claim_amount > 0);

    // ── 3. Get vault's available balance from the vault_state tracker ─────────
    // vault_state.vault_available_ct was advanced by prepare_distribute; it equals
    // initial_enc + ElGamal::encode(distributed_amount) = Enc(amount, r0).
    let vault_available_ct = vault_state.vault_available_ct;

    // Also write this ciphertext to the vault so Token-2022's balance check
    // uses the exact same bytes that proof generation uses.
    {
        use spl_token_2022_interface::extension::{
            confidential_transfer::ConfidentialTransferAccount, BaseStateWithExtensionsMut, PodStateWithExtensionsMut,
        };
        let mut acc = ctx.svm.get_account(&pool_setup.reward_vault).unwrap();
        {
            let mut state = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut acc.data).unwrap();
            let ct_ext = state.get_extension_mut::<ConfidentialTransferAccount>().unwrap();
            ct_ext.available_balance = PodElGamalCiphertext::from(vault_available_ct);
        }
        ctx.svm.set_account(pool_setup.reward_vault, acc).unwrap();
    }

    // ── 4. Generate transfer proofs ───────────────────────────────────────────

    let proof_data = transfer_split_proof_data(
        &vault_available_ct,
        &vault_decryptable,
        claim_amount,
        &vault_elgamal_kp,
        &vault_aes_key,
        user_elgamal_kp.pubkey(),
        None, // no auditor
    )
    .expect("transfer proof generation failed");

    // ── 5. Pre-verify proofs into context state accounts ─────────────────────
    let proof_authority = Keypair::new();

    let eq_ctx_kp = create_proof_context_state(
        &mut ctx,
        ProofInstruction::VerifyCiphertextCommitmentEquality,
        bytes_of(&proof_data.equality_proof_data),
        std::mem::size_of::<CiphertextCommitmentEqualityProofContext>(),
        &proof_authority,
    );

    let cv_ctx_kp = create_proof_context_state(
        &mut ctx,
        ProofInstruction::VerifyBatchedGroupedCiphertext3HandlesValidity,
        bytes_of(&proof_data.ciphertext_validity_proof_data_with_ciphertext.proof_data),
        std::mem::size_of::<BatchedGroupedCiphertext3HandlesValidityProofContext>(),
        &proof_authority,
    );

    let rp_ctx_kp = create_proof_context_state(
        &mut ctx,
        ProofInstruction::VerifyBatchedRangeProofU128,
        bytes_of(&proof_data.range_proof_data),
        std::mem::size_of::<BatchedRangeProofContext>(),
        &proof_authority,
    );

    // ── 6. Build the new_source_decryptable_available_balance ─────────────────
    // After the transfer, vault's new available = 0 (all claimed).
    let new_vault_available_ct: AeCiphertext = vault_aes_key.encrypt(0u64);
    let new_source_decryptable: [u8; 36] = new_vault_available_ct.to_bytes();

    // Auditor ciphertexts: taken from the proof (commitment is non-zero even
    // when no auditor key is set; handle is identity but commitment is not).
    let auditor_lo: [u8; 64] =
        bytemuck::bytes_of(&proof_data.ciphertext_validity_proof_data_with_ciphertext.ciphertext_lo)
            .try_into()
            .unwrap();
    let auditor_hi: [u8; 64] =
        bytemuck::bytes_of(&proof_data.ciphertext_validity_proof_data_with_ciphertext.ciphertext_hi)
            .try_into()
            .unwrap();

    // ── 7. Build the 167-byte confidential transfer data ──────────────────────
    let mut ct_bytes = [0u8; 167];
    ct_bytes[0..36].copy_from_slice(&new_source_decryptable);
    ct_bytes[36..100].copy_from_slice(&auditor_lo);
    ct_bytes[100..164].copy_from_slice(&auditor_hi);
    // offsets 164,165,166 = 0 (use context state accounts)

    // ── 8. Submit ClaimContinuous ─────────────────────────────────────────────
    // Build the instruction with the 167-byte CT data appended after the amount.
    let mut claim_builder = rewards_program_client::instructions::ClaimContinuousBuilder::new();
    claim_builder
        .user(user.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(user_reward_pda)
        .user_tracked_token_account(user_tracked_ta)
        .reward_vault(pool_setup.reward_vault)
        .user_reward_token_account(user_reward_ata)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .reward_mint(pool_setup.reward_mint.pubkey())
        .tracked_token_program(spl_token_interface::ID)
        .reward_token_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority)
        .amount(0); // claim all
    let mut claim_ix = claim_builder.instruction();

    // Append 3 proof context state accounts.
    for ctx_kp in [&eq_ctx_kp, &cv_ctx_kp, &rp_ctx_kp] {
        claim_ix.accounts.push(solana_sdk::instruction::AccountMeta::new_readonly(ctx_kp.pubkey(), false));
    }
    // Append the 167-byte CT data to instruction data.
    claim_ix.data.extend_from_slice(&ct_bytes);

    let bh = ctx.svm.latest_blockhash();
    ctx.svm
        .send_transaction(Transaction::new_signed_with_payer(
            &[claim_ix],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer, &user],
            bh,
        ))
        .expect("confidential claim should succeed");
}

// ─── Test 5: Multi-distribute cumulative claim ────────────────────────────────

/// Distribute twice then claim all accumulated rewards in one claim.
/// Verifies that `ConfidentialVaultState::prepare_distribute` accumulates correctly
/// across multiple rounds and that the total_distributed matches both distributions.
#[test]
fn test_multi_distribute_confidential_claim() {
    let mut ctx = TestContext::new();
    let (pool_setup, vault_elgamal_kp, vault_aes_key, initial_vault_enc) = make_confidential_pool(&mut ctx);
    pool_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    // ── 1. User opts in ───────────────────────────────────────────────────────
    let user = ctx.create_funded_keypair();
    let user_tracked_ta =
        ctx.create_token_account_with_balance(&user.pubkey(), &pool_setup.tracked_mint.pubkey(), 1_000_000);
    let (user_reward_pda, user_reward_bump) = find_user_reward_account_pda(&pool_setup.reward_pool_pda, &user.pubkey());
    let (user_reward_ata, user_elgamal_kp, _user_aes_key) =
        create_and_configure_ct_account(&mut ctx, &user, &pool_setup.reward_mint.pubkey());

    let (event_authority, _) = find_event_authority_pda();
    let (revocation_pda, _) = find_revocation_pda(&pool_setup.reward_pool_pda, &user.pubkey());
    let mut opt_in_builder = rewards_program_client::instructions::ContinuousOptInBuilder::new();
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
    let mut opt_in_ix = opt_in_builder.instruction();
    opt_in_ix.accounts.push(solana_sdk::instruction::AccountMeta {
        pubkey: user_reward_ata,
        is_signer: false,
        is_writable: false,
    });
    let bh = ctx.svm.latest_blockhash();
    ctx.svm
        .send_transaction(Transaction::new_signed_with_payer(
            &[opt_in_ix],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer, &user],
            bh,
        ))
        .expect("opt-in");

    // ── 2. Fund authority TA ──────────────────────────────────────────────────
    let authority_ta = ctx.create_ata_for_program_with_balance(
        &pool_setup.authority.pubkey(),
        &pool_setup.reward_mint.pubkey(),
        DEFAULT_REWARD_AMOUNT * 10,
        &TOKEN_2022_PROGRAM_ID,
    );

    // ── 3. First distribute ───────────────────────────────────────────────────
    let mut vault_state = rewards_program_client::confidential_helpers::ConfidentialVaultState::new(initial_vault_enc);
    let (decryptable_bytes_1, _) = vault_state.prepare_distribute(DEFAULT_REWARD_AMOUNT, &vault_aes_key);

    let dist_ix_1 = rewards_program_client::instructions::DistributeContinuousRewardBuilder::new()
        .authority(pool_setup.authority.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .reward_mint(pool_setup.reward_mint.pubkey())
        .reward_vault(pool_setup.reward_vault)
        .authority_token_account(authority_ta)
        .reward_token_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority)
        .amount(DEFAULT_REWARD_AMOUNT)
        .expected_pending_balance_credit_counter(1)
        .new_decryptable_available_balance(decryptable_bytes_1);

    let bh = ctx.svm.latest_blockhash();
    ctx.svm
        .send_transaction(Transaction::new_signed_with_payer(
            &[dist_ix_1],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer, &pool_setup.authority],
            bh,
        ))
        .expect("first distribute");

    // ── 4. Second distribute ──────────────────────────────────────────────────
    let (decryptable_bytes_2, _) = vault_state.prepare_distribute(DEFAULT_REWARD_AMOUNT, &vault_aes_key);

    let dist_ix_2 = rewards_program_client::instructions::DistributeContinuousRewardBuilder::new()
        .authority(pool_setup.authority.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .reward_mint(pool_setup.reward_mint.pubkey())
        .reward_vault(pool_setup.reward_vault)
        .authority_token_account(authority_ta)
        .reward_token_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority)
        .amount(DEFAULT_REWARD_AMOUNT)
        .expected_pending_balance_credit_counter(1)
        .new_decryptable_available_balance(decryptable_bytes_2);

    let bh = ctx.svm.latest_blockhash();
    ctx.svm
        .send_transaction(Transaction::new_signed_with_payer(
            &[dist_ix_2],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer, &pool_setup.authority],
            bh,
        ))
        .expect("second distribute");

    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    let claim_amount = pool.total_distributed;
    assert_eq!(claim_amount, DEFAULT_REWARD_AMOUNT * 2, "pool should record both distributions");

    // ── 5. Write cumulative vault ciphertext for proof generation ─────────────
    let vault_available_ct = vault_state.vault_available_ct;
    {
        use spl_token_2022_interface::extension::{
            confidential_transfer::ConfidentialTransferAccount, BaseStateWithExtensionsMut, PodStateWithExtensionsMut,
        };
        let mut acc = ctx.svm.get_account(&pool_setup.reward_vault).unwrap();
        {
            let mut state = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut acc.data).unwrap();
            let ct_ext = state.get_extension_mut::<ConfidentialTransferAccount>().unwrap();
            ct_ext.available_balance = PodElGamalCiphertext::from(vault_available_ct);
        }
        ctx.svm.set_account(pool_setup.reward_vault, acc).unwrap();
    }

    // ── 6. Generate transfer proofs for total claim amount ────────────────────
    let vault_decryptable: solana_zk_sdk::encryption::auth_encryption::AeCiphertext =
        vault_aes_key.encrypt(claim_amount);

    let proof_data = transfer_split_proof_data(
        &vault_available_ct,
        &vault_decryptable,
        claim_amount,
        &vault_elgamal_kp,
        &vault_aes_key,
        user_elgamal_kp.pubkey(),
        None,
    )
    .expect("transfer proof generation failed");

    // ── 7. Pre-verify proofs ──────────────────────────────────────────────────
    let proof_authority = Keypair::new();

    let eq_ctx_kp = create_proof_context_state(
        &mut ctx,
        ProofInstruction::VerifyCiphertextCommitmentEquality,
        bytes_of(&proof_data.equality_proof_data),
        std::mem::size_of::<CiphertextCommitmentEqualityProofContext>(),
        &proof_authority,
    );
    let cv_ctx_kp = create_proof_context_state(
        &mut ctx,
        ProofInstruction::VerifyBatchedGroupedCiphertext3HandlesValidity,
        bytes_of(&proof_data.ciphertext_validity_proof_data_with_ciphertext.proof_data),
        std::mem::size_of::<BatchedGroupedCiphertext3HandlesValidityProofContext>(),
        &proof_authority,
    );
    let rp_ctx_kp = create_proof_context_state(
        &mut ctx,
        ProofInstruction::VerifyBatchedRangeProofU128,
        bytes_of(&proof_data.range_proof_data),
        std::mem::size_of::<BatchedRangeProofContext>(),
        &proof_authority,
    );

    // ── 8. Build CT data ──────────────────────────────────────────────────────
    let new_vault_available_ct: solana_zk_sdk::encryption::auth_encryption::AeCiphertext = vault_aes_key.encrypt(0u64);
    let new_source_decryptable: [u8; 36] = new_vault_available_ct.to_bytes();

    let auditor_lo: [u8; 64] =
        bytemuck::bytes_of(&proof_data.ciphertext_validity_proof_data_with_ciphertext.ciphertext_lo)
            .try_into()
            .unwrap();
    let auditor_hi: [u8; 64] =
        bytemuck::bytes_of(&proof_data.ciphertext_validity_proof_data_with_ciphertext.ciphertext_hi)
            .try_into()
            .unwrap();

    let mut ct_bytes = [0u8; 167];
    ct_bytes[0..36].copy_from_slice(&new_source_decryptable);
    ct_bytes[36..100].copy_from_slice(&auditor_lo);
    ct_bytes[100..164].copy_from_slice(&auditor_hi);

    // ── 9. Submit ClaimContinuous ─────────────────────────────────────────────
    let mut claim_builder = rewards_program_client::instructions::ClaimContinuousBuilder::new();
    claim_builder
        .user(user.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(user_reward_pda)
        .user_tracked_token_account(user_tracked_ta)
        .reward_vault(pool_setup.reward_vault)
        .user_reward_token_account(user_reward_ata)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .reward_mint(pool_setup.reward_mint.pubkey())
        .tracked_token_program(spl_token_interface::ID)
        .reward_token_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority)
        .amount(0);
    let mut claim_ix = claim_builder.instruction();
    for ctx_kp in [&eq_ctx_kp, &cv_ctx_kp, &rp_ctx_kp] {
        claim_ix.accounts.push(solana_sdk::instruction::AccountMeta::new_readonly(ctx_kp.pubkey(), false));
    }
    claim_ix.data.extend_from_slice(&ct_bytes);

    let bh = ctx.svm.latest_blockhash();
    ctx.svm
        .send_transaction(Transaction::new_signed_with_payer(
            &[claim_ix],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer, &user],
            bh,
        ))
        .expect("multi-distribute confidential claim should succeed");

    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    assert_eq!(pool.total_claimed, DEFAULT_REWARD_AMOUNT * 2, "total_claimed should equal both distributions");
}
