use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
use solana_sdk::{
    account::Account,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_associated_token_account::{
    get_associated_token_address, get_associated_token_address_with_program_id,
    instruction::create_associated_token_account_idempotent,
};
use spl_token_2022::{
    extension::{
        transfer_fee::{instruction::initialize_transfer_fee_config, TransferFeeAmount},
        transfer_hook::instruction::initialize as initialize_transfer_hook,
        BaseStateWithExtensions, ExtensionType, StateWithExtensions,
    },
    instruction::{initialize_mint2, mint_to_checked},
    state::{Account as Token2022Account, Mint as Token2022Mint},
};
use spl_token_interface::state::{Account as TokenAccount, AccountState, Mint as TokenMint};

use super::TestContext;

pub use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;
pub use spl_token_interface::ID as TOKEN_PROGRAM_ID;

pub struct Token2022TransferFeeAmounts {
    pub spendable: u64,
    pub withheld: u64,
}

pub fn calculate_token_2022_transfer_fee(amount: u64, transfer_fee_basis_points: u16, maximum_fee: u64) -> u64 {
    let numerator = u128::from(amount) * u128::from(transfer_fee_basis_points);
    let raw_fee = numerator.div_ceil(10_000) as u64;
    raw_fee.min(maximum_fee)
}

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

        self.set_account(
            mint.pubkey(),
            Account {
                lamports: self.minimum_balance_for_rent_exemption(TokenMint::LEN),
                data,
                owner: TOKEN_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );
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

        self.set_account(
            ata,
            Account {
                lamports: self.minimum_balance_for_rent_exemption(TokenAccount::LEN),
                data,
                owner: TOKEN_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        ata
    }

    pub fn set_token_balance(&mut self, token_account: &Pubkey, amount: u64) {
        let mut account = self.get_account(token_account).expect("Token account not found");
        account.data[64..72].copy_from_slice(&amount.to_le_bytes());
        self.set_account(*token_account, account);
    }

    pub fn get_token_balance(&self, token_account: &Pubkey) -> u64 {
        let account = self.get_account(token_account).expect("Token account not found");
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

        self.set_account(
            mint.pubkey(),
            Account {
                lamports: self.minimum_balance_for_rent_exemption(TokenMint::LEN),
                data,
                owner: TOKEN_2022_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );
    }

    pub fn create_token_2022_transfer_fee_mint(
        &mut self,
        mint: &Keypair,
        mint_authority: &Pubkey,
        decimals: u8,
        transfer_fee_basis_points: u16,
        maximum_fee: u64,
    ) {
        let mint_len =
            ExtensionType::try_calculate_account_len::<Token2022Mint>(&[ExtensionType::TransferFeeConfig]).unwrap();

        self.set_account(
            mint.pubkey(),
            Account {
                lamports: self.minimum_balance_for_rent_exemption(mint_len),
                data: vec![0u8; mint_len],
                owner: TOKEN_2022_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        self.send_transaction(
            initialize_transfer_fee_config(
                &TOKEN_2022_PROGRAM_ID,
                &mint.pubkey(),
                Some(mint_authority),
                Some(mint_authority),
                transfer_fee_basis_points,
                maximum_fee,
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

    pub fn create_token_2022_transfer_hook_mint(
        &mut self,
        mint: &Keypair,
        mint_authority: &Pubkey,
        decimals: u8,
        transfer_hook_program_id: &Pubkey,
    ) {
        let mint_len =
            ExtensionType::try_calculate_account_len::<Token2022Mint>(&[ExtensionType::TransferHook]).unwrap();

        self.set_account(
            mint.pubkey(),
            Account {
                lamports: self.minimum_balance_for_rent_exemption(mint_len),
                data: vec![0u8; mint_len],
                owner: TOKEN_2022_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );

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

    pub fn create_token_2022_ata(&mut self, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
        let ata = get_associated_token_address_with_program_id(owner, mint, &TOKEN_2022_PROGRAM_ID);
        self.send_transaction(
            create_associated_token_account_idempotent(&self.payer.pubkey(), owner, mint, &TOKEN_2022_PROGRAM_ID),
            &[],
        )
        .unwrap();
        ata
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

        self.set_account(
            ata,
            Account {
                lamports: self.minimum_balance_for_rent_exemption(TokenAccount::LEN),
                data,
                owner: TOKEN_2022_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        ata
    }

    pub fn mint_token_2022(&mut self, mint: &Pubkey, destination: &Pubkey, amount: u64, decimals: u8) {
        self.send_transaction(
            mint_to_checked(&TOKEN_2022_PROGRAM_ID, mint, destination, &self.payer.pubkey(), &[], amount, decimals)
                .unwrap(),
            &[],
        )
        .unwrap();
    }

    pub fn get_token_2022_transfer_fee_amounts(&self, account: &Pubkey) -> Token2022TransferFeeAmounts {
        let account_data = self.get_account(account).expect("Token account not found");
        let parsed =
            StateWithExtensions::<Token2022Account>::unpack(&account_data.data).expect("Token account should parse");
        let transfer_fee_amount =
            parsed.get_extension::<TransferFeeAmount>().expect("Transfer fee extension should exist");

        Token2022TransferFeeAmounts {
            spendable: parsed.base.amount,
            withheld: u64::from(transfer_fee_amount.withheld_amount),
        }
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
