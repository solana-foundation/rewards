use rewards_program_client::instructions::{AddDirectRecipientBuilder, CloseDirectRecipientBuilder};
use rewards_program_client::types::VestingSchedule;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::fixtures::{ClaimDirectSetup, CreateDirectDistributionSetup};
use crate::utils::{
    find_direct_recipient_pda, find_event_authority_pda, InstructionTestFixture, TestContext, TestInstruction,
};

pub struct CloseDirectRecipientSetup {
    pub recipient: Keypair,
    pub original_payer: Keypair,
    pub distribution_pda: Pubkey,
    pub recipient_pda: Pubkey,
    pub token_program: Pubkey,
}

impl CloseDirectRecipientSetup {
    pub fn builder(ctx: &mut TestContext) -> CloseDirectRecipientSetupBuilder<'_> {
        CloseDirectRecipientSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn new_token_2022(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).token_2022().build()
    }

    pub fn build_instruction(&self, _ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = CloseDirectRecipientBuilder::new();
        builder
            .recipient(self.recipient.pubkey())
            .original_payer(self.original_payer.pubkey())
            .distribution(self.distribution_pda)
            .recipient_account(self.recipient_pda)
            .event_authority(event_authority);

        TestInstruction { instruction: builder.instruction(), signers: vec![], name: "CloseDirectRecipient" }
    }

    pub fn build_instruction_with_wrong_recipient(
        &self,
        _ctx: &TestContext,
        wrong_recipient: &Keypair,
    ) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();
        let (wrong_recipient_pda, _) = find_direct_recipient_pda(&self.distribution_pda, &wrong_recipient.pubkey());

        let mut builder = CloseDirectRecipientBuilder::new();
        builder
            .recipient(wrong_recipient.pubkey())
            .original_payer(self.original_payer.pubkey())
            .distribution(self.distribution_pda)
            .recipient_account(wrong_recipient_pda)
            .event_authority(event_authority);

        TestInstruction { instruction: builder.instruction(), signers: vec![], name: "CloseDirectRecipient" }
    }

    pub fn build_instruction_with_wrong_original_payer(
        &self,
        _ctx: &TestContext,
        wrong_payer: Pubkey,
    ) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = CloseDirectRecipientBuilder::new();
        builder
            .recipient(self.recipient.pubkey())
            .original_payer(wrong_payer)
            .distribution(self.distribution_pda)
            .recipient_account(self.recipient_pda)
            .event_authority(event_authority);

        TestInstruction { instruction: builder.instruction(), signers: vec![], name: "CloseDirectRecipient" }
    }
}

pub struct CloseDirectRecipientSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    token_program: Pubkey,
    amount: u64,
}

impl<'a> CloseDirectRecipientSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self { ctx, token_program: TOKEN_PROGRAM_ID, amount: 1_000_000 }
    }

    pub fn token_2022(mut self) -> Self {
        self.token_program = TOKEN_2022_PROGRAM_ID;
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
        self
    }

    pub fn build(self) -> CloseDirectRecipientSetup {
        let distribution_setup =
            CreateDirectDistributionSetup::builder(self.ctx).token_program(self.token_program).build();
        let create_ix = distribution_setup.build_instruction(self.ctx);
        create_ix.send_expect_success(self.ctx);

        let current_ts = self.ctx.get_current_timestamp();
        let recipient = self.ctx.create_funded_keypair();
        let rent_payer = self.ctx.create_funded_keypair();
        let (recipient_pda, recipient_bump) =
            find_direct_recipient_pda(&distribution_setup.distribution_pda, &recipient.pubkey());

        let authority_token_account = self.ctx.create_ata_for_program_with_balance(
            &distribution_setup.authority.pubkey(),
            &distribution_setup.mint.pubkey(),
            self.amount,
            &self.token_program,
        );

        // Build add_direct_recipient instruction with a dedicated payer (not ctx.payer)
        let (event_authority, _) = find_event_authority_pda();
        let mut add_builder = AddDirectRecipientBuilder::new();
        add_builder
            .payer(rent_payer.pubkey())
            .authority(distribution_setup.authority.pubkey())
            .distribution(distribution_setup.distribution_pda)
            .recipient_account(recipient_pda)
            .recipient(recipient.pubkey())
            .mint(distribution_setup.mint.pubkey())
            .distribution_vault(distribution_setup.distribution_vault)
            .authority_token_account(authority_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .bump(recipient_bump)
            .amount(self.amount)
            .schedule(VestingSchedule::Immediate);

        let add_ix = TestInstruction {
            instruction: add_builder.instruction(),
            signers: vec![rent_payer.insecure_clone(), distribution_setup.authority.insecure_clone()],
            name: "AddDirectRecipient",
        };
        add_ix.send_expect_success(self.ctx);

        // Claim full amount so recipient can be closed
        let recipient_token_account = self.ctx.create_ata_for_program(
            &recipient.pubkey(),
            &distribution_setup.mint.pubkey(),
            &self.token_program,
        );

        let claim_setup = ClaimDirectSetup {
            recipient: recipient.insecure_clone(),
            distribution_pda: distribution_setup.distribution_pda,
            recipient_pda,
            recipient_bump,
            mint: distribution_setup.mint.pubkey(),
            distribution_vault: distribution_setup.distribution_vault,
            recipient_token_account,
            token_program: self.token_program,
            amount: self.amount,
            start_ts: current_ts,
            end_ts: current_ts + 1,
        };
        let claim_ix = claim_setup.build_instruction(self.ctx);
        claim_ix.send_expect_success(self.ctx);

        CloseDirectRecipientSetup {
            recipient,
            original_payer: rent_payer,
            distribution_pda: distribution_setup.distribution_pda,
            recipient_pda,
            token_program: self.token_program,
        }
    }
}

pub struct CloseDirectRecipientFixture;

impl InstructionTestFixture for CloseDirectRecipientFixture {
    const INSTRUCTION_NAME: &'static str = "CloseDirectRecipient";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = CloseDirectRecipientSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    fn required_signers() -> &'static [usize] {
        &[]
    }

    /// Account indices that must be writable:
    /// 1: original_payer
    /// 3: recipient_account
    fn required_writable() -> &'static [usize] {
        &[1, 3]
    }

    fn system_program_index() -> Option<usize> {
        None
    }

    fn current_program_index() -> Option<usize> {
        Some(5)
    }

    fn data_len() -> usize {
        1 // discriminator only
    }
}
