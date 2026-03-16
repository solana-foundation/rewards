#[cfg(feature = "confidential")]
use crate::generated::instructions::CloseContinuousPoolBuilder;
#[cfg(feature = "confidential")]
use crate::generated::instructions::DistributeContinuousRewardBuilder;

#[cfg(feature = "confidential")]
impl DistributeContinuousRewardBuilder {
    pub fn new_decryptable_available_balance(&self, bytes: [u8; 36]) -> solana_instruction::Instruction {
        let mut ix = self.instruction();
        ix.data.extend_from_slice(&bytes);
        ix
    }
}

#[cfg(feature = "confidential")]
impl CloseContinuousPoolBuilder {
    /// Builds a `CloseContinuousPool` instruction for a confidential pool that has
    /// unclaimed rewards remaining in the vault.
    ///
    /// Appends `new_decryptable` (AES-GCM ciphertext of 0) to the instruction data
    /// and adds the two ZK proof context accounts for `ConfidentialWithdraw`.
    ///
    /// For pools where all rewards have already been claimed (`total_distributed ==
    /// total_claimed`), use `.instruction()` directly — no proof data is needed.
    pub fn confidential_close(
        &self,
        new_decryptable: [u8; 36],
        equality_proof_context: solana_pubkey::Pubkey,
        range_proof_context: solana_pubkey::Pubkey,
        zero_ciphertext_proof_context: solana_pubkey::Pubkey,
    ) -> solana_instruction::Instruction {
        let mut ix = self.instruction();
        ix.data.extend_from_slice(&new_decryptable);
        ix.accounts.push(solana_instruction::AccountMeta::new_readonly(equality_proof_context, false));
        ix.accounts.push(solana_instruction::AccountMeta::new_readonly(range_proof_context, false));
        ix.accounts.push(solana_instruction::AccountMeta::new_readonly(zero_ciphertext_proof_context, false));
        ix
    }
}
