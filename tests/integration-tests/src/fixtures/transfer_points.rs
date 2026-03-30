use rewards_program_client::instructions::TransferPointsBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use crate::fixtures::{IssuePointsSetup, DEFAULT_ISSUE_QUANTITY};
use crate::utils::{find_event_authority_pda, InstructionTestFixture, TestContext, TestInstruction};

pub struct TransferPointsSetup {
    pub authority: Keypair,
    pub from_user: Keypair,
    pub to_user: Pubkey,
    pub points_config_pda: Pubkey,
    pub points_mint_pda: Pubkey,
    pub from_token_account: Pubkey,
    pub to_token_account: Pubkey,
    pub quantity: u64,
    pub issued_quantity: u64,
}

impl TransferPointsSetup {
    pub fn builder(ctx: &mut TestContext) -> TransferPointsSetupBuilder<'_> {
        TransferPointsSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = TransferPointsBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .authority(self.authority.pubkey())
            .from_user(self.from_user.pubkey())
            .to_user(self.to_user)
            .points_config(self.points_config_pda)
            .points_mint(self.points_mint_pda)
            .from_token_account(self.from_token_account)
            .to_token_account(self.to_token_account)
            .token2022_program(TOKEN_2022_PROGRAM_ID)
            .event_authority(event_authority)
            .quantity(self.quantity);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone(), self.from_user.insecure_clone()],
            name: "TransferPoints",
        }
    }
}

pub struct TransferPointsSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    quantity: u64,
    issue_quantity: u64,
}

impl<'a> TransferPointsSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self { ctx, quantity: 500, issue_quantity: DEFAULT_ISSUE_QUANTITY }
    }

    pub fn quantity(mut self, quantity: u64) -> Self {
        self.quantity = quantity;
        self
    }

    pub fn issue_quantity(mut self, issue_quantity: u64) -> Self {
        self.issue_quantity = issue_quantity;
        self
    }

    pub fn build(self) -> TransferPointsSetup {
        // Create config with transferable=1
        let init_setup = crate::fixtures::InitPointsSetup::builder(self.ctx).transferable(1).build();
        let init_ix = init_setup.build_instruction(self.ctx);
        init_ix.send_expect_success(self.ctx);

        let from_user = self.ctx.create_funded_keypair();
        let from_token_account = get_associated_token_address_with_program_id(
            &from_user.pubkey(),
            &init_setup.points_mint_pda,
            &TOKEN_2022_PROGRAM_ID,
        );

        // Issue points to from_user
        let issue_setup = IssuePointsSetup {
            authority: init_setup.authority.insecure_clone(),
            points_config_pda: init_setup.points_config_pda,
            points_mint_pda: init_setup.points_mint_pda,
            user: from_user.pubkey(),
            user_ata: from_token_account,
            quantity: self.issue_quantity,
        };
        let issue_ix = issue_setup.build_instruction(self.ctx);
        issue_ix.send_expect_success(self.ctx);

        let to_user = Keypair::new();
        let to_token_account = get_associated_token_address_with_program_id(
            &to_user.pubkey(),
            &init_setup.points_mint_pda,
            &TOKEN_2022_PROGRAM_ID,
        );

        TransferPointsSetup {
            authority: init_setup.authority,
            from_user,
            to_user: to_user.pubkey(),
            points_config_pda: init_setup.points_config_pda,
            points_mint_pda: init_setup.points_mint_pda,
            from_token_account,
            to_token_account,
            quantity: self.quantity,
            issued_quantity: self.issue_quantity,
        }
    }
}

pub struct TransferPointsFixture;

impl InstructionTestFixture for TransferPointsFixture {
    const INSTRUCTION_NAME: &'static str = "TransferPoints";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = TransferPointsSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// 0: payer, 1: authority, 2: from_user
    fn required_signers() -> &'static [usize] {
        &[0, 1, 2]
    }

    /// 0: payer, 5: points_mint, 6: from_token_account, 7: to_token_account
    fn required_writable() -> &'static [usize] {
        &[0, 5, 6, 7]
    }

    fn system_program_index() -> Option<usize> {
        Some(10)
    }

    fn current_program_index() -> Option<usize> {
        Some(12)
    }

    fn data_len() -> usize {
        1 + 8 // discriminator + quantity
    }
}
