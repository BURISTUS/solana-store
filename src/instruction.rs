use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::{
    id,
    state::{Price, Settings},
};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum PriceInstruction {
    Price,
    UpdateSettings { admin: [u8; 32], updated_price: u64 },
}

impl PriceInstruction {
    pub fn price(user: &Pubkey) -> Instruction {
        let price_pubkey = Price::get_price_pubkey(user);
        let (settings_pubkey, _) = Settings::get_settings_pubkey();
        Instruction::new_with_borsh(
            id(),
            &PriceInstruction::Price,
            vec![
                AccountMeta::new_readonly(*user, true),
                AccountMeta::new(price_pubkey, false),
                AccountMeta::new(settings_pubkey, false),
            ],
        )
    }

    pub fn update_settings(
        admin: &Pubkey,
        new_admin: [u8; 32],
        updated_price: u64,
    ) -> Instruction {
        let (settings_pubkey, _) = Settings::get_settings_pubkey();
        Instruction::new_with_borsh(
            id(),
            &PriceInstruction::UpdateSettings {
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
}
