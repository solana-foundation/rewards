use rewards_program_client::instructions::IssuePointsBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use crate::fixtures::InitPointsSetup;
use crate::utils::{find_event_authority_pda, InstructionTestFixture, TestContext, TestInstruction};

pub const DEFAULT_ISSUE_QUANTITY: u64 = 1_000;

pub struct IssuePointsSetup {
    pub authority: Keypair,
    pub points_config_pda: Pubkey,
    pub points_mint_pda: Pubkey,
    pub user: Pubkey,
    pub user_ata: Pubkey,
    pub quantity: u64,
}

impl IssuePointsSetup {
    pub fn builder(ctx: &mut TestContext) -> IssuePointsSetupBuilder<'_> {
        IssuePointsSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = IssuePointsBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .authority(self.authority.pubkey())
            .points_config(self.points_config_pda)
            .points_mint(self.points_mint_pda)
            .user(self.user)
            .user_token_account(self.user_ata)
            .token2022_program(TOKEN_2022_PROGRAM_ID)
            .event_authority(event_authority)
            .quantity(self.quantity);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone()],
            name: "IssuePoints",
        }
    }
}

pub struct IssuePointsSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    quantity: u64,
    transferable: u8,
    revocable: u8,
}

impl<'a> IssuePointsSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self { ctx, quantity: DEFAULT_ISSUE_QUANTITY, transferable: 0, revocable: 0 }
    }

    pub fn quantity(mut self, quantity: u64) -> Self {
        self.quantity = quantity;
        self
    }

    pub fn transferable(mut self, transferable: u8) -> Self {
        self.transferable = transferable;
        self
    }

    pub fn revocable(mut self, revocable: u8) -> Self {
        self.revocable = revocable;
        self
    }

    pub fn build(self) -> IssuePointsSetup {
        // Create and execute InitPoints first
        let init_setup =
            InitPointsSetup::builder(self.ctx).transferable(self.transferable).revocable(self.revocable).build();
        let init_ix = init_setup.build_instruction(self.ctx);
        init_ix.send_expect_success(self.ctx);

        let user = Keypair::new();
        let user_ata = get_associated_token_address_with_program_id(
            &user.pubkey(),
            &init_setup.points_mint_pda,
            &TOKEN_2022_PROGRAM_ID,
        );

        IssuePointsSetup {
            authority: init_setup.authority,
            points_config_pda: init_setup.points_config_pda,
            points_mint_pda: init_setup.points_mint_pda,
            user: user.pubkey(),
            user_ata,
            quantity: self.quantity,
        }
    }
}

pub struct IssuePointsFixture;

impl InstructionTestFixture for IssuePointsFixture {
    const INSTRUCTION_NAME: &'static str = "IssuePoints";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = IssuePointsSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// 0: payer, 1: authority
    fn required_signers() -> &'static [usize] {
        &[0, 1]
    }

    /// 0: payer, 3: points_mint, 5: user_token_account
    fn required_writable() -> &'static [usize] {
        &[0, 3, 5]
    }

    fn system_program_index() -> Option<usize> {
        Some(8)
    }

    fn current_program_index() -> Option<usize> {
        Some(10)
    }

    fn data_len() -> usize {
        1 + 8 // discriminator + quantity
    }
}
