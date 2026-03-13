use core::mem::MaybeUninit;
use core::slice::from_raw_parts;

use pinocchio::{
    account::AccountView,
    cpi::Signer,
    instruction::{InstructionAccount, InstructionView},
    ProgramResult,
};
use pinocchio_token_2022::ID as TOKEN_2022_PROGRAM_ID;

// Token-2022 instruction discriminants:
//   byte 0 = 27  (TokenInstruction::ConfidentialTransferExtension)
//   byte 1 = sub-discriminant from ConfidentialTransferInstruction enum

const CT_EXT: u8 = 27;
const CT_DEPOSIT: u8 = 5;
const CT_TRANSFER: u8 = 7;

/// Raw `TransferInstructionData` fields passed by the client.
///
/// Layout (167 bytes):
/// - `new_source_decryptable_available_balance`: 36 bytes (AES-GCM ciphertext of the vault's new available balance)
/// - `transfer_amount_auditor_ciphertext_lo`:    64 bytes (ElGamal ciphertext, low bits; all-zeros if no auditor)
/// - `transfer_amount_auditor_ciphertext_hi`:    64 bytes (ElGamal ciphertext, high bits; all-zeros if no auditor)
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
        let mut data = [MaybeUninit::<u8>::uninit(); 11];
        data[0].write(CT_EXT);
        data[1].write(CT_DEPOSIT);
        let amount_bytes = self.amount.to_le_bytes();
        for (i, b) in amount_bytes.iter().enumerate() {
            data[2 + i].write(*b);
        }
        data[10].write(self.decimals);

        let instruction_accounts = [
            InstructionAccount::writable(self.token_account.address()),
            InstructionAccount::readonly(self.mint.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ];

        let instruction = InstructionView {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &instruction_accounts,
            data: unsafe { from_raw_parts(data.as_ptr() as *const u8, 11) },
        };

        pinocchio::cpi::invoke_signed(&instruction, &[self.token_account, self.mint, self.authority], self.signers)
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
    /// Informational: the credit counter value the client expected before this
    /// apply. Token-2022 stores it but does not validate it.
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
        let mut data = [MaybeUninit::<u8>::uninit(); 46];
        data[0].write(CT_EXT);
        data[1].write(CT_APPLY_PENDING);
        for (i, b) in self.expected_pending_balance_credit_counter.to_le_bytes().iter().enumerate() {
            data[2 + i].write(*b);
        }
        for (i, b) in self.new_decryptable_available_balance.iter().enumerate() {
            data[10 + i].write(*b);
        }

        let instruction_accounts = [
            InstructionAccount::writable(self.token_account.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ];

        let instruction = InstructionView {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &instruction_accounts,
            data: unsafe { from_raw_parts(data.as_ptr() as *const u8, 46) },
        };

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
        let mut data = [MaybeUninit::<u8>::uninit(); 169];
        data[0].write(CT_EXT);
        data[1].write(CT_TRANSFER);

        // new_source_decryptable_available_balance (36 bytes)
        for (i, b) in self.transfer_data.new_source_decryptable_available_balance.iter().enumerate() {
            data[2 + i].write(*b);
        }
        // transfer_amount_auditor_ciphertext_lo (64 bytes)
        for (i, b) in self.transfer_data.transfer_amount_auditor_ciphertext_lo.iter().enumerate() {
            data[38 + i].write(*b);
        }
        // transfer_amount_auditor_ciphertext_hi (64 bytes)
        for (i, b) in self.transfer_data.transfer_amount_auditor_ciphertext_hi.iter().enumerate() {
            data[102 + i].write(*b);
        }
        // proof instruction offsets
        data[166].write(self.transfer_data.equality_proof_instruction_offset as u8);
        data[167].write(self.transfer_data.ciphertext_validity_proof_instruction_offset as u8);
        data[168].write(self.transfer_data.range_proof_instruction_offset as u8);

        let instruction_accounts = [
            InstructionAccount::writable(self.source.address()),
            InstructionAccount::readonly(self.mint.address()),
            InstructionAccount::writable(self.destination.address()),
            InstructionAccount::readonly(self.equality_proof_context.address()),
            InstructionAccount::readonly(self.ciphertext_validity_proof_context.address()),
            InstructionAccount::readonly(self.range_proof_context.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ];

        let instruction = InstructionView {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &instruction_accounts,
            data: unsafe { from_raw_parts(data.as_ptr() as *const u8, 169) },
        };

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
