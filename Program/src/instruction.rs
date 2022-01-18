use std::convert::TryInto;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};
use solana_program::entrypoint_deprecated::ProgramResult;
use solana_program::program_error::ProgramError;

use crate::{
    id,
    state::{Price, Settings},
};
use crate::error::PriceError;

use crate::error::PriceError::InvalidInstruction;
use crate::instruction::StoreInstruction::Sell;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum StoreInstruction {
    ///Price
    /// Accounts:
    /// 0. `[signer]` owner of a price
    /// 1. `[writeble]` price_account, PDA
    /// 2. `[]` settings_account, PDA
    InitializeStore,
    /// Update price for store. Only admin can do it.
    /// Accounts:
    /// 0. `[signer, writable]` Admin
    /// 1. `[writable]` settings_account, PDA
    /// 2. `[]` Rent sysvar
    /// 3. `[]` System program
    UpdateSettings { admin: [u8; 32], updated_price: u32 },
    /// Buying SPL with SOL
    /// Accounts:
    /// 0. `[signer]` store authority
    /// 1. `[writable]` store token account
    /// 2. `[writable]` user token account
    /// 3. `[]` token program
    /// 4. `[signer, writable]` Debit lamports from this account
    /// 5. `[writable]` Credit lamports to this account
    /// 6. `[]` System program
    Buy { amount: u64 },
    /// 0. `[signer]` user authority
    /// 1. `[writable]` user token account
    /// 2. `[writable]` store token account
    /// 3. `[]` token program
    /// 4. `[signer, writable]` Debit lamports from this account
    /// 5. `[writable]` Credit lamports to this account
    /// 6. `[]` System program
    Sell { amount: u64 },
}

impl StoreInstruction {
    pub fn initialize_store(user: &Pubkey) -> Instruction {
        let price_pubkey = Price::get_price_pubkey(user);
        let (settings_pubkey, _) = Settings::get_settings_pubkey();
        Instruction::new_with_borsh(
            id(),
            &StoreInstruction::InitializeStore,
            vec![
                AccountMeta::new_readonly(*user, true),
                AccountMeta::new(price_pubkey, false),
                AccountMeta::new(settings_pubkey, false),
            ],
        )
    }

    pub fn update_price(
        admin: &Pubkey,
        new_admin: [u8; 32],
        updated_price: u32,
    ) -> Instruction {
        let (settings_pubkey, _) = Settings::get_settings_pubkey();
        Instruction::new_with_borsh(
            id(),
            &StoreInstruction::UpdateSettings {
                admin: new_admin,
                updated_price,
        },
            vec![
                AccountMeta::new(*admin, true),
                AccountMeta::new(settings_pubkey, false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )
    }
    pub fn buy(
        pool_owner: &Pubkey,
        user: &Pubkey,
        admin_token_account: &Pubkey,
        user_token_account: &Pubkey,
        token_program: &Pubkey,
        amount: u64,
    ) -> Instruction {
        Instruction::new_with_borsh(
            id(),
            &StoreInstruction::Buy {
                amount
            },
            vec![
                AccountMeta::new(*pool_owner, true),
                AccountMeta::new(*user, false),
                AccountMeta::new(*admin_token_account, false),
                AccountMeta::new(*user_token_account, false),
                AccountMeta::new(*token_program, false),
                AccountMeta::new_readonly(spl_token::id(), false)
            ]
        )
    }

    pub fn sell(
        pool_owner: &Pubkey,
        user: &Pubkey,
        admin_token_account: &Pubkey,
        user_token_account: &Pubkey,
        token_program: &Pubkey,
        amount: u64,
    ) -> Instruction {
        Instruction::new_with_borsh(
            id(),
            &StoreInstruction::Buy {
                amount
            },
            vec![
                AccountMeta::new(*pool_owner, true),
                AccountMeta::new(*user, false),
                AccountMeta::new(*admin_token_account, false),
                AccountMeta::new(*user_token_account, false),
                AccountMeta::new(*token_program, false),
                AccountMeta::new_readonly(spl_token::id(), false)
            ]
        )
    }
}

// pub fn buy(
//     pool_owner: &Pubkey,
//     user: &Pubkey,
//     admin_token_account: &Pubkey,
//     user_token_account: &Pubkey,
//     token_program: &Pubkey,
// ) -> Instruction {
//     Instruction::new_with_borsh(
//         id(),
//         &StoreInstruction::Buy {
//             amount
//         },
//         vec![
//             AccountMeta::new(*admin, true),
//             AccountMeta::new(*user, false),
//             AccountMeta::new(*admin_token_account, false),
//             AccountMeta::new(*user_token_account, false),
//             AccountMeta::new(*token_program, false),
//             AccountMeta::new_readonly(spl_token::id(), false)
//         ]
//     )
// }
//
// pub fn sell(
//     pool_owner: &Pubkey,
//     user: &Pubkey,
//     admin_token_account: &Pubkey,
//     user_token_account: &Pubkey,
//     token_program: &Pubkey,
// ) -> Instruction {
//     Instruction::new_with_borsh(
//         id(),
//         &StoreInstruction::Buy {
//             amount
//         },
//         vec![
//             AccountMeta::new(*admin, true),
//             AccountMeta::new(*user, false),
//             AccountMeta::new(*admin_token_account, false),
//             AccountMeta::new(*user_token_account, false),
//             AccountMeta::new(*token_program, false),
//             AccountMeta::new_readonly(spl_token::id(), false)
//         ]
//     )
// }