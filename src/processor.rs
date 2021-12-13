use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
    program_error::ProgramError
};

use crate::{id, SETTINGS_SEED};
use crate::{instruction::PriceInstruction, state::Price, state::Settings, error::PriceError};
pub struct Processor;

impl Processor {
    pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        msg!("price: {:?}", input);
        let instruction = PriceInstruction::try_from_slice(input)?;
        match instruction {
            PriceInstruction::Price => Self::process_price(accounts),
            PriceInstruction::UpdateSettings {
                admin,
                updated_price,
            } => Self::process_update_settings(accounts, admin, updated_price),
        }
    }

    fn process_price(accounts: &[AccountInfo]) -> ProgramResult {
        msg!("process_price");
        let acc_iter = &mut accounts.iter();
        let price_info = next_account_info(acc_iter)?;
        let user_info = next_account_info(acc_iter)?;
        let settings_info = next_account_info(acc_iter)?;

        if !user_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if !Price::is_pubkey_valid(user_info.key, price_info.key) {
            return Err(PriceError::WrongCounterPDA.into());
        }
        if !Settings::is_pubkey_ok(settings_info.key) {
            return Err(PriceError::WrongSettingsPDA.into());
        }

        let mut price = Price::try_from_slice(&price_info.data.borrow())?;
        
        price.counter += 1;
        price.value += 15;

        msg!("price is {:?}", price.value);

        let _ = price.serialize(&mut &mut price_info.data.borrow_mut()[..]);

        Ok(())
    }

    fn process_update_settings(
        accounts: &[AccountInfo],
        admin: [u8; 32],
        updated_price: u32,
    ) -> ProgramResult {
        msg!(
            "process_update_settings: admin={:?} updated_price={:?}",
            admin,
            updated_price,
        );
        let acc_iter = &mut accounts.iter();
        let admin_info = next_account_info(acc_iter)?;
        let settings_info = next_account_info(acc_iter)?;
        let rent_info = next_account_info(acc_iter)?;
        let system_program_info = next_account_info(acc_iter)?;

        let (settings_pubkey, bump_seed) = Settings::get_settings_pubkey();
        if settings_info.data_is_empty() {
            msg!("Creating settings account");
            let settings = Settings {
                admin: admin_info.key.to_bytes(),
                updated_price,
            };
            let space = settings.try_to_vec()?.len();
            let rent = &Rent::from_account_info(rent_info)?;
            let lamports = rent.minimum_balance(space);
            let signer_seeds: &[&[_]] = &[SETTINGS_SEED.as_bytes(), &[bump_seed]];
            invoke_signed(
                &system_instruction::create_account(
                    admin_info.key,
                    &settings_pubkey,
                    lamports,
                    space as u64,
                    &id(),
                ),
                &[
                    admin_info.clone(),
                    settings_info.clone(),
                    system_program_info.clone(),
                ],
                &[&signer_seeds],
            )?;
        }

        let mut settings = Settings::try_from_slice(&settings_info.data.borrow())?;
        settings.admin = admin;
        settings.updated_price = updated_price;

        let _ = settings.serialize(&mut &mut settings_info.data.borrow_mut()[..]);
        msg!("process_update_settings: done");
        Ok(())
    }
}
