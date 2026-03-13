/// Test helpers for setting up Token-2022 ConfidentialTransfer accounts and pools.
use bytemuck;
use rewards_program_client::confidential_helpers::{configure_account, TOKEN_2022_PROGRAM_ID};
use solana_sdk::{
    account::Account,
    instruction::AccountMeta,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_zk_sdk::{
    encryption::{
        auth_encryption::AeKey,
        elgamal::{ElGamalCiphertext, ElGamalKeypair},
        pod::elgamal::{PodElGamalCiphertext, PodElGamalPubkey},
    },
    zk_elgamal_proof_program::{instruction::ProofInstruction, proof_data::pubkey_validity::PubkeyValidityProofData},
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022_interface::{
    extension::{
        confidential_transfer::{ConfidentialTransferAccount, ConfidentialTransferMint},
        BaseStateWithExtensionsMut, ExtensionType, PodStateWithExtensionsMut,
    },
    pod::{PodAccount, PodMint},
};

use crate::utils::TestContext;

/// Minimal size for a Token-2022 account with the `ConfidentialTransferAccount` extension.
pub fn ct_token_account_len() -> usize {
    ExtensionType::try_calculate_account_len::<PodAccount>(&[ExtensionType::ConfidentialTransferAccount]).unwrap()
}

/// Minimal size for a Token-2022 mint with the `ConfidentialTransferMint` extension.
pub fn ct_mint_len() -> usize {
    ExtensionType::try_calculate_account_len::<PodMint>(&[ExtensionType::ConfidentialTransferMint]).unwrap()
}

/// Creates a Token-2022 mint with the `ConfidentialTransferMint` extension.
///
/// - `auto_approve_new_accounts = true` — new ATAs are automatically approved
///   without needing explicit authority approval.
/// - No auditor key (all zeros).
pub fn create_ct_mint(ctx: &mut TestContext, mint: &Keypair, mint_authority: &Pubkey) {
    let space = ct_mint_len();
    let mut data = vec![0u8; space];

    {
        let mut mint_state = PodStateWithExtensionsMut::<PodMint>::unpack_uninitialized(&mut data).unwrap();
        mint_state.base.mint_authority = solana_program::program_option::COption::Some(*mint_authority).into();
        mint_state.base.decimals = 6;
        mint_state.base.is_initialized = true.into();

        let ct_mint_ext = mint_state.init_extension::<ConfidentialTransferMint>(true).unwrap();
        ct_mint_ext.authority = *bytemuck::from_bytes(mint_authority.as_ref());
        ct_mint_ext.auto_approve_new_accounts = true.into();
        // No auditor: auditor_elgamal_pubkey stays zeroed (OptionalNonZeroElGamalPubkey default)

        mint_state.init_account_type().unwrap();
    }

    ctx.svm
        .set_account(
            mint.pubkey(),
            Account {
                lamports: ctx.svm.minimum_balance_for_rent_exemption(space),
                data,
                owner: TOKEN_2022_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();
}

/// Creates a Token-2022 ATA sized to hold the `ConfidentialTransferAccount` extension,
/// then submits a `VerifyPubkeyValidity` + `ConfigureAccount` transaction to enable
/// confidential transfers on it.
///
/// Returns `(ata_pubkey, elgamal_keypair, aes_key)` so callers can generate proofs later.
pub fn create_and_configure_ct_account(
    ctx: &mut TestContext,
    owner: &Keypair,
    mint: &Pubkey,
) -> (Pubkey, ElGamalKeypair, AeKey) {
    let ata = get_associated_token_address_with_program_id(&owner.pubkey(), mint, &TOKEN_2022_PROGRAM_ID);
    let space = ct_token_account_len();

    // Pre-allocate the ATA with the correct size for the CT extension.
    // We seed `mint` and `owner` in the base account data so Token-2022 recognises it.
    let mut data = vec![0u8; space];
    {
        use solana_program::program_option::COption;
        let mut acc_state = PodStateWithExtensionsMut::<PodAccount>::unpack_uninitialized(&mut data).unwrap();
        acc_state.base.mint = *mint;
        acc_state.base.owner = owner.pubkey();
        acc_state.base.amount = 0u64.into();
        acc_state.base.delegate = COption::None.into();
        acc_state.base.state = spl_token_2022_interface::state::AccountState::Initialized as u8;
        acc_state.base.is_native = COption::None.into();
        acc_state.base.delegated_amount = 0u64.into();
        acc_state.base.close_authority = COption::None.into();
        acc_state.init_account_type().unwrap();
    }

    ctx.svm
        .set_account(
            ata,
            Account {
                lamports: ctx.svm.minimum_balance_for_rent_exemption(space),
                data,
                owner: TOKEN_2022_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();

    // Generate the ElGamal keypair and AES key for this account.
    let elgamal_kp = ElGamalKeypair::new_rand();
    let aes_key = AeKey::new_rand();

    // Encrypt 0 to get the initial decryptable_zero_balance.
    let zero_ct = aes_key.encrypt(0);
    let decryptable_zero_balance: [u8; 36] = zero_ct.to_bytes();

    // Build VerifyPubkeyValidity proof (inline, no context state).
    let proof_data = PubkeyValidityProofData::new(&elgamal_kp).expect("pubkey validity proof");
    let verify_ix = ProofInstruction::VerifyPubkeyValidity.encode_verify_proof(None, &proof_data);

    // Build ConfigureAccount instruction.
    // proof_instruction_offset = -1 → look one instruction back (VerifyPubkeyValidity at ix[0]).
    // proof_account_or_sysvar = Instructions sysvar (for inline proof lookup).
    let instructions_sysvar = solana_sdk::sysvar::instructions::id();
    let configure_ix = configure_account(
        &ata,
        mint,
        &instructions_sysvar,
        &owner.pubkey(),
        decryptable_zero_balance,
        u64::MAX, // maximum_pending_balance_credit_counter
        -1,       // proof is one instruction before ConfigureAccount
    );

    let blockhash = ctx.svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(
        &[verify_ix, configure_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, owner],
        blockhash,
    );
    ctx.svm.send_transaction(tx).expect("ConfigureAccount should succeed");

    (ata, elgamal_kp, aes_key)
}

/// Creates a proof context state account for a given proof type and pre-verifies it.
///
/// Returns the pubkey of the context state account.
///
/// `proof_data_bytes`: serialized proof data (after the 1-byte discriminant).
/// `proof_instruction`: the `ProofInstruction` variant to verify.
/// `context_state_len`: byte size of the context state (proof-type-specific).
pub fn create_proof_context_state(
    ctx: &mut TestContext,
    proof_instruction: ProofInstruction,
    proof_data_bytes: &[u8],
    context_state_len: usize,
    authority: &Keypair,
) -> Keypair {
    use solana_zk_sdk::zk_elgamal_proof_program::id as zk_id;
    use solana_zk_sdk::zk_elgamal_proof_program::state::ProofContextStateMeta;

    let context_state_kp = Keypair::new();
    let context_state_pubkey = context_state_kp.pubkey();

    // Pre-allocate the context state account.
    let full_len = std::mem::size_of::<ProofContextStateMeta>() + context_state_len;
    ctx.svm
        .set_account(
            context_state_pubkey,
            Account {
                lamports: ctx.svm.minimum_balance_for_rent_exemption(full_len),
                data: vec![0u8; full_len],
                owner: zk_id(),
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();

    // Build the verify instruction that writes context to the state account.
    let mut ix_data = vec![proof_instruction as u8];
    ix_data.extend_from_slice(proof_data_bytes);

    let verify_and_store_ix = solana_sdk::instruction::Instruction {
        program_id: zk_id(),
        accounts: vec![
            AccountMeta::new(context_state_pubkey, false),
            AccountMeta::new_readonly(authority.pubkey(), false),
        ],
        data: ix_data,
    };

    let blockhash = ctx.svm.latest_blockhash();
    let tx =
        Transaction::new_signed_with_payer(&[verify_and_store_ix], Some(&ctx.payer.pubkey()), &[&ctx.payer], blockhash);
    ctx.svm.send_transaction(tx).expect("proof context state creation should succeed");

    context_state_kp
}

/// Creates a Token-2022 ATA for a PDA-owned vault with the
/// `ConfidentialTransferAccount` extension pre-configured.
///
/// Since the vault owner (pool PDA) cannot sign, this bypasses the
/// `ConfigureAccount` instruction and writes the account state directly.
///
/// The `available_balance` is seeded with a proper encryption of 0 (non-zero
/// randomness) so that BPF `add_with_lo_hi` propagates that randomness through
/// every distribute, producing ciphertexts suitable for ZK proof generation.
///
/// Returns `(vault_pubkey, vault_elgamal_keypair, vault_aes_key, initial_available_enc)`.
/// The caller should keep `initial_available_enc` and use it to derive the post-distribute
/// available balance via `initial_available_enc + ElGamal::encode(distributed_amount)`.
pub fn create_ct_vault(
    ctx: &mut TestContext,
    vault_owner: &Pubkey,
    mint: &Pubkey,
) -> (Pubkey, ElGamalKeypair, AeKey, ElGamalCiphertext) {
    use solana_program::program_option::COption;

    let ata = get_associated_token_address_with_program_id(vault_owner, mint, &TOKEN_2022_PROGRAM_ID);
    let space = ct_token_account_len();
    let mut data = vec![0u8; space];

    let vault_elgamal_kp = ElGamalKeypair::new_rand();
    let vault_aes_key = AeKey::new_rand();
    // Seed with Enc(0, r0): non-zero randomness so BPF arithmetic propagates it.
    let initial_enc = vault_elgamal_kp.pubkey().encrypt(0u64);

    {
        let mut acc_state = PodStateWithExtensionsMut::<PodAccount>::unpack_uninitialized(&mut data).unwrap();
        acc_state.base.mint = *mint;
        acc_state.base.owner = *vault_owner;
        acc_state.base.amount = 0u64.into();
        acc_state.base.delegate = COption::None.into();
        acc_state.base.state = spl_token_2022_interface::state::AccountState::Initialized as u8;
        acc_state.base.is_native = COption::None.into();
        acc_state.base.delegated_amount = 0u64.into();
        acc_state.base.close_authority = COption::None.into();

        let ct_ext = acc_state.init_extension::<ConfidentialTransferAccount>(true).unwrap();
        ct_ext.approved = true.into();
        ct_ext.elgamal_pubkey = PodElGamalPubkey::from(*vault_elgamal_kp.pubkey());
        ct_ext.allow_confidential_credits = true.into();
        ct_ext.allow_non_confidential_credits = true.into();
        ct_ext.maximum_pending_balance_credit_counter = u64::MAX.into();
        ct_ext.available_balance = PodElGamalCiphertext::from(initial_enc);

        acc_state.init_account_type().unwrap();
    }

    ctx.svm
        .set_account(
            ata,
            Account {
                lamports: ctx.svm.minimum_balance_for_rent_exemption(space),
                data,
                owner: TOKEN_2022_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();

    (ata, vault_elgamal_kp, vault_aes_key, initial_enc)
}
