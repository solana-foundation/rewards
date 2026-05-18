use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
use solana_sdk::{
    account::Account,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_associated_token_account::{get_associated_token_address, get_associated_token_address_with_program_id};
use spl_token_2022::{
    extension::{transfer_hook::instruction::initialize as initialize_transfer_hook, ExtensionType},
    instruction::initialize_mint2,
    state::Mint as Token2022Mint,
};
use spl_token_interface::state::{Account as TokenAccount, AccountState, Mint as TokenMint};

use super::TestContext;

pub use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;
pub use spl_token_interface::ID as TOKEN_PROGRAM_ID;

impl TestContext {
    pub fn create_mint(&mut self, mint: &Keypair, mint_authority: &Pubkey, decimals: u8) {
        let mint_state = TokenMint {
            mint_authority: COption::Some(*mint_authority),
            supply: 0,
            decimals,
            is_initialized: true,
            freeze_authority: COption::None,
        };

        let mut data = vec![0u8; TokenMint::LEN];
        mint_state.pack_into_slice(&mut data);

        self.svm
            .set_account(
                mint.pubkey(),
                Account {
                    lamports: self.svm.minimum_balance_for_rent_exemption(TokenMint::LEN),
                    data,
                    owner: TOKEN_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();
    }

    pub fn create_token_account(&mut self, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
        self.create_token_account_with_balance(owner, mint, 0)
    }

    pub fn create_token_account_with_balance(&mut self, owner: &Pubkey, mint: &Pubkey, amount: u64) -> Pubkey {
        let ata = get_associated_token_address(owner, mint);

        let token_account = TokenAccount {
            mint: *mint,
            owner: *owner,
            amount,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        };

        let mut data = vec![0u8; TokenAccount::LEN];
        token_account.pack_into_slice(&mut data);

        self.svm
            .set_account(
                ata,
                Account {
                    lamports: self.svm.minimum_balance_for_rent_exemption(TokenAccount::LEN),
                    data,
                    owner: TOKEN_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

        ata
    }

    pub fn set_token_balance(&mut self, token_account: &Pubkey, amount: u64) {
        let mut account = self.svm.get_account(token_account).expect("Token account not found");
        account.data[64..72].copy_from_slice(&amount.to_le_bytes());
        self.svm.set_account(*token_account, account).unwrap();
    }

    pub fn get_token_balance(&self, token_account: &Pubkey) -> u64 {
        let account = self.svm.get_account(token_account).expect("Token account not found");
        u64::from_le_bytes(account.data[64..72].try_into().unwrap())
    }

    pub fn create_token_2022_mint(&mut self, mint: &Keypair, mint_authority: &Pubkey, decimals: u8) {
        let mint_state = TokenMint {
            mint_authority: COption::Some(*mint_authority),
            supply: 0,
            decimals,
            is_initialized: true,
            freeze_authority: COption::None,
        };

        let mut data = vec![0u8; TokenMint::LEN];
        mint_state.pack_into_slice(&mut data);

        self.svm
            .set_account(
                mint.pubkey(),
                Account {
                    lamports: self.svm.minimum_balance_for_rent_exemption(TokenMint::LEN),
                    data,
                    owner: TOKEN_2022_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();
    }

    pub fn create_token_2022_transfer_hook_mint(
        &mut self,
        mint: &Keypair,
        mint_authority: &Pubkey,
        decimals: u8,
        transfer_hook_program_id: &Pubkey,
    ) {
        let mint_len =
            ExtensionType::try_calculate_account_len::<Token2022Mint>(&[ExtensionType::TransferHook]).unwrap();

        self.svm
            .set_account(
                mint.pubkey(),
                Account {
                    lamports: self.svm.minimum_balance_for_rent_exemption(mint_len),
                    data: vec![0u8; mint_len],
                    owner: TOKEN_2022_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

        self.send_transaction(
            initialize_transfer_hook(
                &TOKEN_2022_PROGRAM_ID,
                &mint.pubkey(),
                Some(*mint_authority),
                Some(*transfer_hook_program_id),
            )
            .unwrap(),
            &[],
        )
        .unwrap();
        self.send_transaction(
            initialize_mint2(&TOKEN_2022_PROGRAM_ID, &mint.pubkey(), mint_authority, None, decimals).unwrap(),
            &[],
        )
        .unwrap();
    }

    pub fn create_token_2022_account(&mut self, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
        self.create_token_2022_account_with_balance(owner, mint, 0)
    }

    pub fn create_token_2022_account_with_balance(&mut self, owner: &Pubkey, mint: &Pubkey, amount: u64) -> Pubkey {
        let ata = get_associated_token_address_with_program_id(owner, mint, &TOKEN_2022_PROGRAM_ID);

        let token_account = TokenAccount {
            mint: *mint,
            owner: *owner,
            amount,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        };

        let mut data = vec![0u8; TokenAccount::LEN];
        token_account.pack_into_slice(&mut data);

        self.svm
            .set_account(
                ata,
                Account {
                    lamports: self.svm.minimum_balance_for_rent_exemption(TokenAccount::LEN),
                    data,
                    owner: TOKEN_2022_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

        ata
    }

    pub fn create_mint_for_program(
        &mut self,
        mint: &Keypair,
        mint_authority: &Pubkey,
        decimals: u8,
        token_program: &Pubkey,
    ) {
        if *token_program == TOKEN_2022_PROGRAM_ID {
            self.create_token_2022_mint(mint, mint_authority, decimals);
        } else {
            self.create_mint(mint, mint_authority, decimals);
        }
    }

    pub fn create_ata_for_program(&mut self, owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
        self.create_ata_for_program_with_balance(owner, mint, 0, token_program)
    }

    pub fn create_ata_for_program_with_balance(
        &mut self,
        owner: &Pubkey,
        mint: &Pubkey,
        amount: u64,
        token_program: &Pubkey,
    ) -> Pubkey {
        if *token_program == TOKEN_2022_PROGRAM_ID {
            self.create_token_2022_account_with_balance(owner, mint, amount)
        } else {
            self.create_token_account_with_balance(owner, mint, amount)
        }
    }
}
