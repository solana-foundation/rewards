use pinocchio::{
    account::AccountView,
    cpi::Signer,
    instruction::{InstructionAccount, InstructionView},
    Address, ProgramResult,
};
use pinocchio_token_2022::{instructions::TransferChecked, ID as TOKEN_2022_PROGRAM_ID};

use crate::{errors::RewardsProgramError, state::RewardPool};

// Token-2022 instruction discriminants:
//   byte 0 = 27  (TokenInstruction::ConfidentialTransferExtension)
//   byte 1 = sub-discriminant from ConfidentialTransferInstruction enum

const CT_EXT: u8 = 27;
const CT_EMPTY_ACCOUNT: u8 = 4;
const CT_DEPOSIT: u8 = 5;
const CT_WITHDRAW: u8 = 6;
const CT_TRANSFER: u8 = 7;

// `ZkE1Gama1Proof11111111111111111111111111111` in base58
pub const ZK_ELGAMAL_PROOF_PROGRAM_ID: Address = Address::new_from_array([
    0x08, 0x63, 0x75, 0xac, 0xe2, 0xae, 0xea, 0x28, 0x1a, 0x6b, 0x37, 0x4d, 0x68, 0x1b, 0xa7, 0x6a, 0x53, 0xcc, 0xf6,
    0x38, 0xc0, 0x74, 0x55, 0x93, 0x6c, 0x05, 0xd0, 0x65, 0x40, 0x00, 0x00, 0x00,
]);

/// Raw `TransferInstructionData` fields passed by the client.
///
/// Layout (167 bytes):
/// - `new_source_decryptable_available_balance`: 36 bytes (AES-GCM ciphertext of the vault's new available balance)
/// - `transfer_amount_auditor_ciphertext_lo`:    64 bytes (ElGamal ciphertext, low bits; must be the actual ciphertext from the validity proof even if no auditor key is configured)
/// - `transfer_amount_auditor_ciphertext_hi`:    64 bytes (ElGamal ciphertext, high bits; same requirement — Token-2022 validates these against the proof's Pedersen commitment regardless)
/// - `equality_proof_instruction_offset`:         1 byte  (0 = use context state account)
/// - `ciphertext_validity_proof_instruction_offset`: 1 byte
/// - `range_proof_instruction_offset`:            1 byte
pub const CONFIDENTIAL_TRANSFER_DATA_LEN: usize = 167;

pub struct ConfidentialTransferData<'a> {
    pub new_source_decryptable_available_balance: &'a [u8; 36],
    pub transfer_amount_auditor_ciphertext_lo: &'a [u8; 64],
    pub transfer_amount_auditor_ciphertext_hi: &'a [u8; 64],
    pub equality_proof_instruction_offset: i8,
    pub ciphertext_validity_proof_instruction_offset: i8,
    pub range_proof_instruction_offset: i8,
}

impl<'a> ConfidentialTransferData<'a> {
    pub fn try_from_bytes(bytes: &'a [u8]) -> Option<Self> {
        if bytes.len() < CONFIDENTIAL_TRANSFER_DATA_LEN {
            return None;
        }
        let new_source_decryptable_available_balance = bytes[0..36].try_into().ok()?;
        let transfer_amount_auditor_ciphertext_lo = bytes[36..100].try_into().ok()?;
        let transfer_amount_auditor_ciphertext_hi = bytes[100..164].try_into().ok()?;
        let equality_proof_instruction_offset = bytes[164] as i8;
        let ciphertext_validity_proof_instruction_offset = bytes[165] as i8;
        let range_proof_instruction_offset = bytes[166] as i8;

        Some(Self {
            new_source_decryptable_available_balance,
            transfer_amount_auditor_ciphertext_lo,
            transfer_amount_auditor_ciphertext_hi,
            equality_proof_instruction_offset,
            ciphertext_validity_proof_instruction_offset,
            range_proof_instruction_offset,
        })
    }
}

/// CPI: ConfidentialTransfer::Deposit
///
/// Converts tokens in the token account's own plaintext balance into
/// confidential pending balance. Used after a standard `TransferChecked`
/// fills the vault, so the vault's incoming rewards are private at rest.
///
/// Accounts:
///   0. [writable] token_account  — the account whose public balance is converted
///   1. []         mint
///   2. [signer]   authority      — owner/delegate of `token_account`
pub struct ConfidentialDeposit<'a, 'b> {
    pub token_account: &'a AccountView,
    pub mint: &'a AccountView,
    pub authority: &'a AccountView,
    pub amount: u64,
    pub decimals: u8,
    pub signers: &'b [Signer<'b, 'b>],
}

impl ConfidentialDeposit<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Instruction data: [CT_EXT, CT_DEPOSIT, amount(8 LE), decimals(1)] = 11 bytes
        let mut data = [0u8; 11];
        data[0] = CT_EXT;
        data[1] = CT_DEPOSIT;
        let amount_bytes = self.amount.to_le_bytes();
        data[2..10].copy_from_slice(&amount_bytes);
        data[10] = self.decimals;

        let instruction_accounts = [
            InstructionAccount::writable(self.token_account.address()),
            InstructionAccount::readonly(self.mint.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ];

        let instruction =
            InstructionView { program_id: &TOKEN_2022_PROGRAM_ID, accounts: &instruction_accounts, data: &data };

        pinocchio::cpi::invoke_signed(&instruction, &[self.token_account, self.mint, self.authority], self.signers)
    }
}

/// CPI: ConfidentialTransfer::EmptyAccount
///
/// Proves the vault's available balance encodes 0, then zeros out the ciphertext
/// bytes so that `CloseAccount` can succeed. Must be called after `Withdraw` has
/// converted all tokens to plaintext.
///
/// Accounts:
///   0. [writable] token_account  — the vault to empty
///   1. []         zero_ciphertext_proof_context
///   2. [signer]   authority      — owner of `token_account` (pool PDA)
pub struct ConfidentialEmptyAccount<'a, 'b> {
    pub token_account: &'a AccountView,
    pub zero_ciphertext_proof_context: &'a AccountView,
    pub authority: &'a AccountView,
    pub signers: &'b [Signer<'b, 'b>],
}

impl ConfidentialEmptyAccount<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // [CT_EXT, CT_EMPTY_ACCOUNT, proof_instruction_offset(1)] = 3 bytes
        let data = [CT_EXT, CT_EMPTY_ACCOUNT, 0u8];

        let instruction_accounts = [
            InstructionAccount::writable(self.token_account.address()),
            InstructionAccount::readonly(self.zero_ciphertext_proof_context.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ];

        let instruction =
            InstructionView { program_id: &TOKEN_2022_PROGRAM_ID, accounts: &instruction_accounts, data: &data };

        pinocchio::cpi::invoke_signed(
            &instruction,
            &[self.token_account, self.zero_ciphertext_proof_context, self.authority],
            self.signers,
        )
    }
}

/// CPI: ConfidentialTransfer::Withdraw
///
/// Converts the vault's confidential available balance back to plaintext.
/// After this CPI the vault's `base.amount` (plaintext) equals `amount` and
/// its CT available balance is zero, allowing a subsequent `TransferChecked`
/// to sweep the tokens and `CloseAccount` to succeed.
///
/// Accounts:
///   0. [writable] token_account  — the vault to withdraw from
///   1. []         mint
///   2. []         equality_proof_context
///   3. []         range_proof_context
///   4. [signer]   authority      — owner of `token_account` (pool PDA)
pub struct ConfidentialWithdraw<'a, 'b> {
    pub token_account: &'a AccountView,
    pub mint: &'a AccountView,
    pub equality_proof_context: &'a AccountView,
    pub range_proof_context: &'a AccountView,
    pub authority: &'a AccountView,
    pub amount: u64,
    pub decimals: u8,
    pub new_decryptable_available_balance: &'b [u8; 36],
    pub signers: &'b [Signer<'b, 'b>],
}

impl ConfidentialWithdraw<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // [CT_EXT, CT_WITHDRAW, amount(8), decimals(1), new_decryptable(36), eq_offset(1), range_offset(1)] = 49 bytes
        let mut data = [0u8; 49];
        data[0] = CT_EXT;
        data[1] = CT_WITHDRAW;
        data[2..10].copy_from_slice(&self.amount.to_le_bytes());
        data[10] = self.decimals;
        data[11..47].copy_from_slice(self.new_decryptable_available_balance);
        // offsets = 0 (use context state accounts), bytes 47-48 already zeroed

        let instruction_accounts = [
            InstructionAccount::writable(self.token_account.address()),
            InstructionAccount::readonly(self.mint.address()),
            InstructionAccount::readonly(self.equality_proof_context.address()),
            InstructionAccount::readonly(self.range_proof_context.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ];

        let instruction =
            InstructionView { program_id: &TOKEN_2022_PROGRAM_ID, accounts: &instruction_accounts, data: &data };

        pinocchio::cpi::invoke_signed(
            &instruction,
            &[self.token_account, self.mint, self.equality_proof_context, self.range_proof_context, self.authority],
            self.signers,
        )
    }
}

/// CPI: ConfidentialTransfer::ApplyPendingBalance
///
/// Moves tokens from the vault's confidential pending balance to its available
/// balance. Called immediately after `ConfidentialDeposit` so that distribute
/// leaves the vault in a claimable state in a single transaction.
///
/// Accounts:
///   0. [writable] token_account  — the vault whose pending balance is applied
///   1. [signer]   authority      — owner of `token_account` (pool PDA)
pub struct ConfidentialApplyPendingBalance<'a, 'b> {
    pub token_account: &'a AccountView,
    pub authority: &'a AccountView,
    /// The vault's `pending_balance_credit_counter` value before this distribute.
    /// One `ConfidentialDeposit` happens in the same tx, so Token-2022 will see
    /// counter+1 when it processes `ApplyPendingBalance`. Pass the value read
    /// off-chain before building the transaction (`counter + 1`).
    pub expected_pending_balance_credit_counter: u64,
    /// AES-GCM ciphertext of the vault's new available balance (deposit amount
    /// + previous available), computed by the authority off-chain.
    pub new_decryptable_available_balance: &'b [u8; 36],
    pub signers: &'b [Signer<'b, 'b>],
}

const CT_APPLY_PENDING: u8 = 8;

impl ConfidentialApplyPendingBalance<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // [CT_EXT, CT_APPLY_PENDING, counter(8 LE), new_decryptable(36)] = 46 bytes
        let mut data = [0u8; 46];
        data[0] = CT_EXT;
        data[1] = CT_APPLY_PENDING;
        data[2..10].copy_from_slice(&self.expected_pending_balance_credit_counter.to_le_bytes());
        data[10..46].copy_from_slice(self.new_decryptable_available_balance);

        let instruction_accounts = [
            InstructionAccount::writable(self.token_account.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ];

        let instruction =
            InstructionView { program_id: &TOKEN_2022_PROGRAM_ID, accounts: &instruction_accounts, data: &data };

        pinocchio::cpi::invoke_signed(&instruction, &[self.token_account, self.authority], self.signers)
    }
}

/// CPI: ConfidentialTransfer::Transfer
///
/// Transfers tokens from vault to user using confidential transfer.
/// All three proof context accounts must reference pre-verified
/// `ProofContextState` accounts (i.e., all instruction offsets = 0).
///
/// Accounts (in CPI order):
///   0. [writable] source            — reward vault
///   1. []         mint
///   2. [writable] destination       — user's reward token account
///   3. []         equality_proof_context
///   4. []         ciphertext_validity_proof_context
///   5. []         range_proof_context
///   6. [signer]   authority         — reward pool PDA
pub struct ConfidentialTransferCpi<'a, 'b> {
    pub source: &'a AccountView,
    pub mint: &'a AccountView,
    pub destination: &'a AccountView,
    pub equality_proof_context: &'a AccountView,
    pub ciphertext_validity_proof_context: &'a AccountView,
    pub range_proof_context: &'a AccountView,
    pub authority: &'a AccountView,
    pub transfer_data: &'b ConfidentialTransferData<'b>,
    pub signers: &'b [Signer<'b, 'b>],
}

impl ConfidentialTransferCpi<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Instruction data: [CT_EXT, CT_TRANSFER, ...TransferInstructionData(167)] = 169 bytes
        let mut data = [0u8; 169];
        data[0] = CT_EXT;
        data[1] = CT_TRANSFER;

        // new_source_decryptable_available_balance (36 bytes)
        data[2..38].copy_from_slice(self.transfer_data.new_source_decryptable_available_balance);
        // transfer_amount_auditor_ciphertext_lo (64 bytes)
        data[38..102].copy_from_slice(self.transfer_data.transfer_amount_auditor_ciphertext_lo);
        // transfer_amount_auditor_ciphertext_hi (64 bytes)
        data[102..166].copy_from_slice(self.transfer_data.transfer_amount_auditor_ciphertext_hi);
        // proof instruction offsets
        data[166] = self.transfer_data.equality_proof_instruction_offset as u8;
        data[167] = self.transfer_data.ciphertext_validity_proof_instruction_offset as u8;
        data[168] = self.transfer_data.range_proof_instruction_offset as u8;

        let instruction_accounts = [
            InstructionAccount::writable(self.source.address()),
            InstructionAccount::readonly(self.mint.address()),
            InstructionAccount::writable(self.destination.address()),
            InstructionAccount::readonly(self.equality_proof_context.address()),
            InstructionAccount::readonly(self.ciphertext_validity_proof_context.address()),
            InstructionAccount::readonly(self.range_proof_context.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ];

        let instruction =
            InstructionView { program_id: &TOKEN_2022_PROGRAM_ID, accounts: &instruction_accounts, data: &data };

        pinocchio::cpi::invoke_signed(
            &instruction,
            &[
                self.source,
                self.mint,
                self.destination,
                self.equality_proof_context,
                self.ciphertext_validity_proof_context,
                self.range_proof_context,
                self.authority,
            ],
            self.signers,
        )
    }
}

/// Transfer reward tokens from vault to destination, dispatching to either
/// `TransferChecked` (plain pool) or `ConfidentialTransfer` (CT pool).
#[allow(clippy::too_many_arguments)]
pub fn transfer_reward_tokens<'a>(
    pool: &RewardPool,
    vault: &'a AccountView,
    destination: &'a AccountView,
    reward_pool_account: &'a AccountView,
    mint: &'a AccountView,
    reward_token_program: &Address,
    amount: u64,
    decimals: u8,
    ct_data: Option<&[u8]>,
    proof_contexts: Option<[&'a AccountView; 3]>,
) -> ProgramResult {
    if pool.confidential_rewards != 0 {
        let ct_bytes = ct_data.ok_or(RewardsProgramError::InvalidAccountData)?;
        let ct = ConfidentialTransferData::try_from_bytes(ct_bytes).ok_or(RewardsProgramError::InvalidAccountData)?;
        let [eq_ctx, cv_ctx, rp_ctx] = proof_contexts.ok_or(RewardsProgramError::InvalidAccountData)?;
        pool.with_signer(|signers| {
            ConfidentialTransferCpi {
                source: vault,
                mint,
                destination,
                equality_proof_context: eq_ctx,
                ciphertext_validity_proof_context: cv_ctx,
                range_proof_context: rp_ctx,
                authority: reward_pool_account,
                transfer_data: &ct,
                signers,
            }
            .invoke()
        })
    } else {
        pool.with_signer(|signers| {
            TransferChecked {
                from: vault,
                mint,
                to: destination,
                authority: reward_pool_account,
                amount,
                decimals,
                token_program: reward_token_program,
            }
            .invoke_signed(signers)
        })
    }
}

/// After a `TransferChecked` into the vault, convert the plaintext deposit into
/// confidential pending balance and immediately apply it to available balance.
/// No-op for non-confidential pools.
#[allow(clippy::too_many_arguments)]
pub fn maybe_confidential_deposit<'a>(
    pool: &RewardPool,
    vault: &'a AccountView,
    mint: &'a AccountView,
    reward_pool_account: &'a AccountView,
    amount: u64,
    decimals: u8,
    expected_pending_balance_credit_counter: u64,
    new_decryptable: Option<&[u8; 36]>,
) -> ProgramResult {
    if pool.confidential_rewards == 0 {
        return Ok(());
    }
    let new_decryptable = new_decryptable.ok_or(RewardsProgramError::InvalidAccountData)?;
    pool.with_signer(|signers| {
        ConfidentialDeposit { token_account: vault, mint, authority: reward_pool_account, amount, decimals, signers }
            .invoke()?;
        ConfidentialApplyPendingBalance {
            token_account: vault,
            authority: reward_pool_account,
            expected_pending_balance_credit_counter,
            new_decryptable_available_balance: new_decryptable,
            signers,
        }
        .invoke()
    })
}
