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
