use litesvm::LiteSVM;
use solana_program::clock::Clock;
use solana_sdk::{
    account::Account,
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};

pub use rewards_program_client::REWARDS_PROGRAM_ID as PROGRAM_ID;

const MIN_LAMPORTS: u64 = 500_000_000;

pub struct TestContext {
    pub svm: LiteSVM,
    pub payer: Keypair,
    pub authority: Keypair,
}

impl TestContext {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new();

        let current_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;

        svm.set_sysvar(&Clock {
            slot: 1,
            epoch_start_timestamp: current_time,
            epoch: 0,
            leader_schedule_epoch: 0,
            unix_timestamp: current_time,
        });

        let program_bytes = include_bytes!("../../../../target/deploy/rewards_program.so");
        let _ = svm.add_program(PROGRAM_ID, program_bytes);

        let payer = Keypair::new();
        let authority = Keypair::new();

        svm.airdrop(&payer.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();
        svm.airdrop(&authority.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

        Self { svm, payer, authority }
    }

    pub fn airdrop_if_required(&mut self, pubkey: &Pubkey, lamports: u64) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(account) = self.svm.get_account(pubkey) {
            if account.lamports < MIN_LAMPORTS {
                return match self.svm.airdrop(pubkey, lamports) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("Airdrop failed: {:?}", e).into()),
                };
            }
        } else {
            return match self.svm.airdrop(pubkey, lamports) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Airdrop failed: {:?}", e).into()),
            };
        }

        Ok(())
    }

    pub fn send_transaction(
        &mut self,
        instruction: Instruction,
        signers: &[&Keypair],
    ) -> Result<u64, Box<dyn std::error::Error>> {
        self.send_transaction_inner(instruction, signers).map_err(|e| format!("Transaction failed: {:?}", e).into())
    }

    pub fn send_transaction_expect_error(
        &mut self,
        instruction: Instruction,
        signers: &[&Keypair],
    ) -> TransactionError {
        self.send_transaction_inner(instruction, signers).expect_err("Transaction should fail")
    }

    fn send_transaction_inner(
        &mut self,
        instruction: Instruction,
        signers: &[&Keypair],
    ) -> Result<u64, TransactionError> {
        let mut all_signers = vec![&self.payer as &dyn Signer];
        all_signers.extend(signers.iter().map(|k| *k as &dyn Signer));

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &all_signers,
            self.svm.latest_blockhash(),
        );

        self.svm.send_transaction(transaction).map(|meta| meta.compute_units_consumed).map_err(|e| e.err)
    }

    pub fn get_account(&self, pubkey: &Pubkey) -> Option<Account> {
        self.svm.get_account(pubkey)
    }

    pub fn create_funded_keypair(&mut self) -> Keypair {
        let kp = Keypair::new();
        self.svm.airdrop(&kp.pubkey(), MIN_LAMPORTS).unwrap();
        kp
    }

    pub fn warp_to_timestamp(&mut self, unix_timestamp: i64) {
        let clock = self.svm.get_sysvar::<Clock>();
        self.svm.set_sysvar(&Clock {
            slot: clock.slot + 1,
            epoch_start_timestamp: unix_timestamp,
            epoch: 0,
            leader_schedule_epoch: 0,
            unix_timestamp,
        });
        self.svm.expire_blockhash();
    }

    pub fn get_current_timestamp(&self) -> i64 {
        self.svm.get_sysvar::<Clock>().unix_timestamp
    }

    pub fn warp_to_slot(&mut self, slot: u64) {
        let clock = self.svm.get_sysvar::<Clock>();
        self.svm.set_sysvar(&Clock { slot, ..clock });
        self.svm.expire_blockhash();
    }

    pub fn advance_slot(&mut self) {
        let clock = self.svm.get_sysvar::<Clock>();
        self.svm.set_sysvar(&Clock { slot: clock.slot + 1, ..clock });
        self.svm.expire_blockhash();
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}
