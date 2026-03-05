use rewards_program_client::{instructions::ClaimMerkleBuilder, types::VestingSchedule};
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

pub const DEFAULT_CLAIMANT_AMOUNT: u64 = 1_000_000;

pub struct ClaimMerkleSetup {
    pub claimant: Keypair,
    pub distribution_pda: Pubkey,
    pub claim_pda: Pubkey,
    pub claim_bump: u8,
    pub revocation_pda: Pubkey,
    pub mint: Pubkey,
    pub distribution_vault: Pubkey,
    pub claimant_token_account: Pubkey,
    pub token_program: Pubkey,
    pub total_amount: u64,
    pub schedule: VestingSchedule,
    pub proof: Vec<[u8; 32]>,
    pub merkle_tree: MerkleTree,
    pub authority: Keypair,
}

impl ClaimMerkleSetup {
    pub fn builder(ctx: &mut TestContext) -> ClaimMerkleSetupBuilder<'_> {
        ClaimMerkleSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn new_token_2022(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).token_2022().build()
    }

    pub fn start_ts(&self) -> i64 {
        match &self.schedule {
            VestingSchedule::Immediate => 0,
            VestingSchedule::Linear { start_ts, .. } => *start_ts,
            VestingSchedule::Cliff { .. } => 0,
            VestingSchedule::CliffLinear { start_ts, .. } => *start_ts,
        }
    }

    pub fn end_ts(&self) -> i64 {
        match &self.schedule {
            VestingSchedule::Immediate => 0,
            VestingSchedule::Linear { end_ts, .. } => *end_ts,
            VestingSchedule::Cliff { cliff_ts } => *cliff_ts,
            VestingSchedule::CliffLinear { end_ts, .. } => *end_ts,
        }
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        self.build_instruction_with_amount(ctx, 0)
    }

    pub fn build_instruction_with_amount(&self, ctx: &TestContext, claim_amount: u64) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = ClaimMerkleBuilder::new();
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
            .amount(claim_amount)
            .proof(self.proof.clone());

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.claimant.insecure_clone()],
            name: "ClaimMerkle",
        }
    }

    pub fn build_instruction_with_wrong_claimant(
        &self,
        ctx: &TestContext,
        wrong_claimant: &Keypair,
        wrong_token_account: Pubkey,
    ) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();
        let (wrong_claim_pda, wrong_claim_bump) =
            find_merkle_claim_pda(&self.distribution_pda, &wrong_claimant.pubkey());
        let (wrong_revocation_pda, _) = find_revocation_pda(&self.distribution_pda, &wrong_claimant.pubkey());

        let mut builder = ClaimMerkleBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .claimant(wrong_claimant.pubkey())
            .distribution(self.distribution_pda)
            .claim_account(wrong_claim_pda)
            .revocation_marker(wrong_revocation_pda)
            .mint(self.mint)
            .distribution_vault(self.distribution_vault)
            .claimant_token_account(wrong_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .claim_bump(wrong_claim_bump)
            .total_amount(self.total_amount)
            .schedule(self.schedule.clone())
            .amount(0)
            .proof(self.proof.clone());

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![wrong_claimant.insecure_clone()],
            name: "ClaimMerkle",
        }
    }

    pub fn build_instruction_with_wrong_proof(&self, ctx: &TestContext, wrong_proof: Vec<[u8; 32]>) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = ClaimMerkleBuilder::new();
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
            .proof(wrong_proof);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.claimant.insecure_clone()],
            name: "ClaimMerkle",
        }
    }

    pub fn build_instruction_with_wrong_amount(&self, ctx: &TestContext, wrong_total_amount: u64) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = ClaimMerkleBuilder::new();
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
            .total_amount(wrong_total_amount)
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

pub struct ClaimMerkleSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    token_program: Pubkey,
    claimant_amount: u64,
    schedule: Option<VestingSchedule>,
    warp_to_end: bool,
    num_claimants: usize,
}

impl<'a> ClaimMerkleSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self {
            ctx,
            token_program: TOKEN_PROGRAM_ID,
            claimant_amount: DEFAULT_CLAIMANT_AMOUNT,
            schedule: None,
            warp_to_end: true,
            num_claimants: 2,
        }
    }

    pub fn token_2022(mut self) -> Self {
        self.token_program = TOKEN_2022_PROGRAM_ID;
        self
    }

    pub fn token_program(mut self, program: Pubkey) -> Self {
        self.token_program = program;
        self
    }

    pub fn claimant_amount(mut self, amount: u64) -> Self {
        self.claimant_amount = amount;
        self
    }

    pub fn schedule(mut self, schedule: VestingSchedule) -> Self {
        self.schedule = Some(schedule);
        self
    }

    pub fn linear(self) -> Self {
        self
    }

    pub fn immediate(mut self) -> Self {
        self.schedule = Some(VestingSchedule::Immediate);
        self
    }

    pub fn warp_to_end(mut self, warp: bool) -> Self {
        self.warp_to_end = warp;
        self
    }

    pub fn num_claimants(mut self, num: usize) -> Self {
        self.num_claimants = num;
        self
    }

    pub fn build(self) -> ClaimMerkleSetup {
        let current_ts = self.ctx.get_current_timestamp();
        let schedule =
            self.schedule.unwrap_or(VestingSchedule::Linear { start_ts: current_ts, end_ts: current_ts + 86400 * 365 });

        let end_ts = match &schedule {
            VestingSchedule::Immediate => current_ts,
            VestingSchedule::Linear { end_ts, .. } => *end_ts,
            VestingSchedule::Cliff { cliff_ts } => *cliff_ts,
            VestingSchedule::CliffLinear { end_ts, .. } => *end_ts,
        };

        let claimant = self.ctx.create_funded_keypair();
        let mut leaves = vec![MerkleLeaf::new(claimant.pubkey(), self.claimant_amount, schedule.clone())];

        for _ in 1..self.num_claimants {
            let other_claimant = Keypair::new();
            leaves.push(MerkleLeaf::new(other_claimant.pubkey(), self.claimant_amount, schedule.clone()));
        }

        let merkle_tree = MerkleTree::new(leaves);
        let total_distribution_amount = self.claimant_amount * self.num_claimants as u64;

        let distribution_setup = CreateMerkleDistributionSetup::builder(self.ctx)
            .amount(total_distribution_amount)
            .total_amount(total_distribution_amount)
            .merkle_root(merkle_tree.root)
            .token_program(self.token_program)
            .build();
        let create_ix = distribution_setup.build_instruction(self.ctx);
        create_ix.send_expect_success(self.ctx);

        let (claim_pda, claim_bump) = find_merkle_claim_pda(&distribution_setup.distribution_pda, &claimant.pubkey());
        let (revocation_pda, _) = find_revocation_pda(&distribution_setup.distribution_pda, &claimant.pubkey());

        let proof = merkle_tree.get_proof_for_claimant(&claimant.pubkey()).unwrap();

        let claimant_token_account =
            self.ctx.create_ata_for_program(&claimant.pubkey(), &distribution_setup.mint.pubkey(), &self.token_program);

        if self.warp_to_end {
            self.ctx.warp_to_timestamp(end_ts);
        }

        ClaimMerkleSetup {
            claimant,
            distribution_pda: distribution_setup.distribution_pda,
            claim_pda,
            claim_bump,
            revocation_pda,
            mint: distribution_setup.mint.pubkey(),
            distribution_vault: distribution_setup.distribution_vault,
            claimant_token_account,
            token_program: self.token_program,
            total_amount: self.claimant_amount,
            schedule,
            proof,
            merkle_tree,
            authority: distribution_setup.authority,
        }
    }
}

pub struct ClaimMerkleFixture;

impl InstructionTestFixture for ClaimMerkleFixture {
    const INSTRUCTION_NAME: &'static str = "ClaimMerkle";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = ClaimMerkleSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// Account indices that must be signers:
    /// 0: payer (handled by TestContext)
    /// 1: claimant
    fn required_signers() -> &'static [usize] {
        &[0, 1]
    }

    /// Account indices that must be writable:
    /// 0: payer
    /// 2: distribution
    /// 3: claim_account
    /// 6: distribution_vault
    /// 7: claimant_token_account
    fn required_writable() -> &'static [usize] {
        &[0, 2, 3, 6, 7]
    }

    fn system_program_index() -> Option<usize> {
        Some(8)
    }

    fn current_program_index() -> Option<usize> {
        Some(11)
    }

    fn data_len() -> usize {
        // discriminator(1) + claim_bump(1) + total_amount(8) + amount(8) + Linear schedule(17) + proof_len(4) + proof(32)
        1 + 1 + 8 + 8 + 17 + 4 + 32
    }
}
