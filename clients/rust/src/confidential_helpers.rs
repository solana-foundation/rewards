/// Client-side helpers for Token-2022 ConfidentialTransfer extension operations.
///
/// These helpers build the prerequisite instructions that users must submit before
/// interacting with a confidential-rewards pool:
///
/// **Pre-opt-in setup** (call once per reward token ATA):
/// 1. `configure_account` — registers the user's ElGamal public key with Token-2022,
///    enabling confidential credits on their reward token account.
///
/// **Pre-claim flow** (call before each claim or opt-out on a confidential pool):
/// 1. `apply_pending_balance` — moves tokens from `pending_balance` to `available_balance`
///    so they can be withdrawn or transferred again.
///
/// **Authority distribute flow** (for each `DistributeContinuousReward` on a confidential pool):
/// 1. Use `ConfidentialVaultState::new` to create the vault state tracker at pool creation.
/// 2. Call `vault_state.prepare_distribute(amount, aes_key)` to get the
///    `new_decryptable_available_balance` bytes to append to the distribute instruction, and
///    the updated `ElGamalCiphertext` for proof generation on the subsequent claim.
///
/// Both instructions are submitted to the Token-2022 program, not to the rewards program.
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

/// Token-2022 program ID.
pub const TOKEN_2022_PROGRAM_ID: Pubkey = solana_pubkey::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

/// Token-2022 ConfidentialTransfer extension discriminant (outer).
const CT_EXT: u8 = 27;

/// ConfidentialTransferInstruction discriminants (inner).
const CT_CONFIGURE_ACCOUNT: u8 = 2;
const CT_APPLY_PENDING_BALANCE: u8 = 8;

/// Builds a `ConfidentialTransferInstruction::ConfigureAccount` instruction.
///
/// This registers `elgamal_pubkey` with the token account and sets the
/// `decryptable_zero_balance` (the AES-GCM ciphertext of 0) so Token-2022
/// knows the starting confidential balance is empty.
///
/// The instruction must be accompanied in the same transaction by a
/// `VerifyPubkeyValidity` instruction from the ZK ElGamal Proof program,
/// or by a pre-verified context state account address.
///
/// # Arguments
/// * `token_account` — the user's reward token ATA to configure
/// * `mint` — the reward token mint (must have ConfidentialTransfer extension)
/// * `proof_account_or_sysvar` — either the `Instructions` sysvar (if proof is inline)
///   or a pre-verified `ProofContextState` account address
/// * `owner` — owner of `token_account`
/// * `decryptable_zero_balance` — 36-byte AES-GCM ciphertext of 0 (generated client-side)
/// * `maximum_pending_balance_credit_counter` — max pending credits before `ApplyPendingBalance`
///   is required; 65536 is a reasonable default
/// * `proof_instruction_offset` — relative offset to the `VerifyPubkeyValidity` instruction
///   in the transaction; 0 means use `proof_account_or_sysvar` as a context state account
pub fn configure_account(
    token_account: &Pubkey,
    mint: &Pubkey,
    proof_account_or_sysvar: &Pubkey,
    owner: &Pubkey,
    decryptable_zero_balance: [u8; 36],
    maximum_pending_balance_credit_counter: u64,
    proof_instruction_offset: i8,
) -> Instruction {
    // Data layout:
    //   [CT_EXT, CT_CONFIGURE_ACCOUNT,
    //    decryptable_zero_balance(36), maximum_pending_balance_credit_counter(8),
    //    proof_instruction_offset(1)]
    //  = 47 bytes
    let mut data = Vec::with_capacity(47);
    data.push(CT_EXT);
    data.push(CT_CONFIGURE_ACCOUNT);
    data.extend_from_slice(&decryptable_zero_balance);
    data.extend_from_slice(&maximum_pending_balance_credit_counter.to_le_bytes());
    data.push(proof_instruction_offset as u8);

    Instruction {
        program_id: TOKEN_2022_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*token_account, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(*proof_account_or_sysvar, false),
            AccountMeta::new_readonly(*owner, true),
        ],
        data,
    }
}

/// Builds a `ConfidentialTransferInstruction::ApplyPendingBalance` instruction.
///
/// Moves tokens from the account's `pending_balance` (received via confidential
/// deposits or transfers) into `available_balance` so they can be transferred
/// or withdrawn.
///
/// Users must call this before claiming from a confidential-rewards pool when
/// they have a non-zero pending balance.
///
/// # Arguments
/// * `token_account` — the user's reward token ATA
/// * `owner` — owner of `token_account`
/// * `expected_pending_balance_credit_counter` — the current value of
///   `ConfidentialTransferAccount::pending_balance_credit_counter`; used as a
///   consistency check (read from on-chain account data before calling)
/// * `new_decryptable_available_balance` — 36-byte AES-GCM ciphertext of the
///   expected new available balance after applying pending (generated client-side
///   by decrypting and re-encrypting the sum)
pub fn apply_pending_balance(
    token_account: &Pubkey,
    owner: &Pubkey,
    expected_pending_balance_credit_counter: u64,
    new_decryptable_available_balance: [u8; 36],
) -> Instruction {
    // Data layout:
    //   [CT_EXT, CT_APPLY_PENDING_BALANCE,
    //    expected_pending_balance_credit_counter(8),
    //    new_decryptable_available_balance(36)]
    //  = 46 bytes
    let mut data = Vec::with_capacity(46);
    data.push(CT_EXT);
    data.push(CT_APPLY_PENDING_BALANCE);
    data.extend_from_slice(&expected_pending_balance_credit_counter.to_le_bytes());
    data.extend_from_slice(&new_decryptable_available_balance);

    Instruction {
        program_id: TOKEN_2022_PROGRAM_ID,
        accounts: vec![AccountMeta::new(*token_account, false), AccountMeta::new_readonly(*owner, true)],
        data,
    }
}

/// Tracks the vault's confidential available balance across multiple distribute calls.
///
/// The authority creates one of these at pool creation time by calling
/// `ConfidentialVaultState::new(initial_enc)` where `initial_enc` is the `Enc(0, r0)`
/// ciphertext seeded in the vault. Each call to `prepare_distribute` updates the tracked
/// available balance and returns the data needed for the distribute instruction.
///
/// Requires the `confidential` cargo feature.
#[cfg(feature = "confidential")]
pub struct ConfidentialVaultState {
    /// Current vault available balance as an ElGamal ciphertext.
    /// Starts as `Enc(0, r0)` and accumulates: `initial + Σ ElGamal::encode(amount_i)`.
    pub vault_available_ct: solana_zk_sdk::encryption::elgamal::ElGamalCiphertext,
    /// Running total of all distributed amounts (= current decryptable plaintext).
    pub cumulative_available: u64,
}

#[cfg(feature = "confidential")]
impl ConfidentialVaultState {
    /// Creates a new vault state from the `Enc(0, r0)` ciphertext returned by
    /// `create_ct_vault` (or equivalent vault setup).
    pub fn new(initial_enc: solana_zk_sdk::encryption::elgamal::ElGamalCiphertext) -> Self {
        Self { vault_available_ct: initial_enc, cumulative_available: 0 }
    }

    /// Computes the `new_decryptable_available_balance` bytes for a confidential
    /// `CloseContinuousPool` call when unclaimed rewards remain in the vault.
    ///
    /// Returns `[u8; 36]`: AES-GCM ciphertext of 0 (vault's balance after withdrawing all).
    /// Pass to `CloseContinuousPoolBuilder::confidential_close`.
    ///
    /// The caller must also generate `WithdrawProofData` using
    /// `spl_token_confidential_transfer_proof_generation::withdraw::withdraw_proof_data` with:
    /// - `current_available_balance = self.vault_available_ct`
    /// - `current_balance = self.cumulative_available`
    /// - `withdraw_amount = pool.total_distributed - pool.total_claimed`
    /// - `elgamal_keypair = vault_elgamal_keypair`
    pub fn prepare_close(&self, vault_aes_key: &solana_zk_sdk::encryption::auth_encryption::AeKey) -> [u8; 36] {
        vault_aes_key.encrypt(0u64).to_bytes()
    }

    /// Computes the data needed for a confidential `DistributeContinuousReward` call.
    ///
    /// Returns `([u8; 36], ElGamalCiphertext)`:
    /// - `[u8; 36]`: `new_decryptable_available_balance` bytes — pass to
    ///   `DistributeContinuousRewardBuilder::new_decryptable_available_balance`.
    /// - `ElGamalCiphertext`: the vault's expected available balance after this distribute,
    ///   used as `current_available_balance` in `transfer_split_proof_data` for the claim.
    ///
    /// Call this immediately before building the distribute instruction. The internal state
    /// advances so the next call reflects the cumulative balance.
    ///
    /// **`effective_amount` ≠ raw instruction `amount`**: pass the back-computed effective
    /// amount (`delta_rpt * opted_in_supply / REWARD_PRECISION`), not the raw `amount` input.
    /// The program computes this same value and deposits only `effective_amount` into the vault.
    pub fn prepare_distribute(
        &mut self,
        effective_amount: u64,
        vault_aes_key: &solana_zk_sdk::encryption::auth_encryption::AeKey,
    ) -> ([u8; 36], solana_zk_sdk::encryption::elgamal::ElGamalCiphertext) {
        self.vault_available_ct =
            self.vault_available_ct + solana_zk_sdk::encryption::elgamal::ElGamal::encode(effective_amount);
        self.cumulative_available = self.cumulative_available.saturating_add(effective_amount);

        let decryptable = vault_aes_key.encrypt(self.cumulative_available);
        (decryptable.to_bytes(), self.vault_available_ct)
    }
}
