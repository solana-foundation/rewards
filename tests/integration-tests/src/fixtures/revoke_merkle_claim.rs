use rewards_program_client::instructions::RevokeMerkleClaimBuilder;
use rewards_program_client::types::{RevokeMode, VestingSchedule};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::fixtures::CreateMerkleDistributionSetup;
use crate::utils::{
    find_event_authority_pda, find_merkle_claim_pda, find_revocation_pda, InstructionTestFixture, MerkleLeaf,
    MerkleTree, TestContext, TestInstruction,
};

pub const DEFAULT_REVOKE_MERKLE_AMOUNT: u64 = 1_000_000;

pub struct RevokeMerkleClaimSetup {
    pub authority: Keypair,
    pub payer: Keypair,
    pub distribution_pda: Pubkey,
    pub claimant: Keypair,
    pub claim_pda: Pubkey,
    pub claim_bump: u8, // still needed for ClaimMerkle instruction
    pub revocation_pda: Pubkey,
    pub mint: Pubkey,
    pub distribution_vault: Pubkey,
    pub claimant_token_account: Pubkey,
    pub authority_token_account: Pubkey,
    pub token_program: Pubkey,
    pub total_amount: u64,
    pub schedule: VestingSchedule,
    pub proof: Vec<[u8; 32]>,
    pub merkle_tree: MerkleTree,
    pub start_ts: i64,
    pub end_ts: i64,
}

impl RevokeMerkleClaimSetup {
    pub fn builder(ctx: &mut TestContext) -> RevokeMerkleClaimSetupBuilder<'_> {
        RevokeMerkleClaimSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn new_token_2022(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).token_2022().build()
    }

    pub fn build_instruction(&self, _ctx: &TestContext, revoke_mode: RevokeMode) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = RevokeMerkleClaimBuilder::new();
        builder
            .authority(self.authority.pubkey())
            .payer(self.payer.pubkey())
            .distribution(self.distribution_pda)
            .claim_account(self.claim_pda)
            .revocation_marker(self.revocation_pda)
            .claimant(self.claimant.pubkey())
            .mint(self.mint)
            .distribution_vault(self.distribution_vault)
            .claimant_token_account(self.claimant_token_account)
            .authority_token_account(self.authority_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .revoke_mode(revoke_mode)
            .total_amount(self.total_amount)
            .schedule(self.schedule.clone())
            .proof(self.proof.clone());

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone(), self.payer.insecure_clone()],
            name: "RevokeMerkleClaim",
        }
    }

    pub fn build_instruction_with_wrong_authority(
        &self,
        _ctx: &TestContext,
        wrong_authority: &Keypair,
        revoke_mode: RevokeMode,
    ) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = RevokeMerkleClaimBuilder::new();
        builder
            .authority(wrong_authority.pubkey())
            .payer(self.payer.pubkey())
            .distribution(self.distribution_pda)
            .claim_account(self.claim_pda)
            .revocation_marker(self.revocation_pda)
            .claimant(self.claimant.pubkey())
            .mint(self.mint)
            .distribution_vault(self.distribution_vault)
            .claimant_token_account(self.claimant_token_account)
            .authority_token_account(self.authority_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .revoke_mode(revoke_mode)
            .total_amount(self.total_amount)
            .schedule(self.schedule.clone())
            .proof(self.proof.clone());

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![wrong_authority.insecure_clone(), self.payer.insecure_clone()],
            name: "RevokeMerkleClaim",
        }
    }

    pub fn build_claim_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = rewards_program_client::instructions::ClaimMerkleBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .claimant(self.claimant.pubkey())
            .distribution(self.distribution_pda)
            .claim_account(self.claim_pda)
            .revocation_marker(self.revocation_pda)
            .mint(self.mint)
            .distribution_vault(self.distribution_vault)
            .claimant_token_account(self.claimant_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .claim_bump(self.claim_bump)
            .total_amount(self.total_amount)
            .schedule(self.schedule.clone())
            .amount(0)
            .proof(self.proof.clone());

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.claimant.insecure_clone()],
            name: "ClaimMerkle",
        }
    }
}

pub struct RevokeMerkleClaimSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    token_program: Pubkey,
    amount: u64,
    schedule: Option<VestingSchedule>,
    num_claimants: usize,
    revocable: u8,
}

impl<'a> RevokeMerkleClaimSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self {
            ctx,
            token_program: TOKEN_PROGRAM_ID,
            amount: DEFAULT_REVOKE_MERKLE_AMOUNT,
            schedule: None,
            num_claimants: 2,
            revocable: 3,
        }
    }

    pub fn token_2022(mut self) -> Self {
        self.token_program = TOKEN_2022_PROGRAM_ID;
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
        self
    }

    pub fn schedule(mut self, schedule: VestingSchedule) -> Self {
        self.schedule = Some(schedule);
        self
    }

    pub fn num_claimants(mut self, num: usize) -> Self {
        self.num_claimants = num;
        self
    }

    pub fn revocable(mut self, revocable: u8) -> Self {
        self.revocable = revocable;
        self
    }

    pub fn build(self) -> RevokeMerkleClaimSetup {
        let current_ts = self.ctx.get_current_timestamp();
        let schedule =
            self.schedule.unwrap_or(VestingSchedule::Linear { start_ts: current_ts, end_ts: current_ts + 86400 * 365 });

        let (start_ts, end_ts) = match &schedule {
            VestingSchedule::Linear { start_ts, end_ts } => (*start_ts, *end_ts),
            VestingSchedule::CliffLinear { start_ts, end_ts, .. } => (*start_ts, *end_ts),
            VestingSchedule::Cliff { cliff_ts } => (0, *cliff_ts),
            VestingSchedule::Immediate => (0, 0),
        };

        let claimant = self.ctx.create_funded_keypair();
        let mut leaves = vec![MerkleLeaf::new(claimant.pubkey(), self.amount, schedule.clone())];

        for _ in 1..self.num_claimants {
            let other = Keypair::new();
            leaves.push(MerkleLeaf::new(other.pubkey(), self.amount, schedule.clone()));
        }

        let merkle_tree = MerkleTree::new(leaves);
        let total_distribution_amount = self.amount * self.num_claimants as u64;

        let distribution_setup = CreateMerkleDistributionSetup::builder(self.ctx)
            .amount(total_distribution_amount)
            .total_amount(total_distribution_amount)
            .merkle_root(merkle_tree.root)
            .token_program(self.token_program)
            .revocable(self.revocable)
            .build();
        let create_ix = distribution_setup.build_instruction(self.ctx);
        create_ix.send_expect_success(self.ctx);

        let (claim_pda, claim_bump) = find_merkle_claim_pda(&distribution_setup.distribution_pda, &claimant.pubkey());
        let (revocation_pda, _) = find_revocation_pda(&distribution_setup.distribution_pda, &claimant.pubkey());

        let proof = merkle_tree.get_proof_for_claimant(&claimant.pubkey()).unwrap();

        let claimant_token_account =
            self.ctx.create_ata_for_program(&claimant.pubkey(), &distribution_setup.mint.pubkey(), &self.token_program);

        let payer = self.ctx.create_funded_keypair();

        RevokeMerkleClaimSetup {
            authority: distribution_setup.authority,
            payer,
            distribution_pda: distribution_setup.distribution_pda,
            claimant,
            claim_pda,
            claim_bump,
            revocation_pda,
            mint: distribution_setup.mint.pubkey(),
            distribution_vault: distribution_setup.distribution_vault,
            claimant_token_account,
            authority_token_account: distribution_setup.authority_token_account,
            token_program: self.token_program,
            total_amount: self.amount,
            schedule,
            proof,
            merkle_tree,
            start_ts,
            end_ts,
        }
    }
}

pub struct RevokeMerkleClaimFixture;

impl InstructionTestFixture for RevokeMerkleClaimFixture {
    const INSTRUCTION_NAME: &'static str = "RevokeMerkleClaim";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = RevokeMerkleClaimSetup::new(ctx);
        setup.build_instruction(ctx, RevokeMode::NonVested)
    }

    /// Account indices that must be signers:
    /// 0: authority
    /// 1: payer
    fn required_signers() -> &'static [usize] {
        &[0, 1]
    }

    /// Account indices that must be writable:
    /// 1: payer
    /// 2: distribution
    /// 4: revocation_marker
    /// 7: distribution_vault
    /// 8: claimant_token_account
    /// 9: authority_token_account
    fn required_writable() -> &'static [usize] {
        &[1, 2, 4, 7, 8, 9]
    }

    fn system_program_index() -> Option<usize> {
        Some(10)
    }

    fn current_program_index() -> Option<usize> {
        Some(13)
    }

    fn data_len() -> usize {
        // discriminator(1) + revoke_mode(1) + total_amount(8)
        // + Linear schedule(17) + proof_len(4) + proof(32)
        1 + 1 + 8 + 17 + 4 + 32
    }
}
