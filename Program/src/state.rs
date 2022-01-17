use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::{id, PRICE_SEED, SETTINGS_SEED};


#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Price {
    pub counter: u32,
    pub value: u32,
}

impl Price {
    pub fn get_price_pubkey(user: &Pubkey) -> Pubkey {
        Pubkey::create_with_seed(user, PRICE_SEED, &id()).unwrap()
    }

    pub fn is_pubkey_valid(user: &Pubkey, price: &Pubkey) -> bool {
        price.to_bytes() == Self::get_price_pubkey(user).to_bytes()
    }
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Settings {
    pub admin: [u8; 32],
    pub updated_price: u32,
}

impl Settings {
    pub fn get_settings_pubkey() -> (Pubkey, u8) {
        Pubkey::find_program_address(&[SETTINGS_SEED.as_bytes()], &id())
    }

    pub fn get_settings_pub() -> Pubkey {
        let (pubkey, _) = Self::get_settings_pubkey();
        pubkey
    }

    pub fn is_pubkey_ok(settings_pubkey: &Pubkey) -> bool {
        let (pubkey, _) = Self::get_settings_pubkey();
        pubkey.to_bytes() == settings_pubkey.to_bytes()
    }
}
