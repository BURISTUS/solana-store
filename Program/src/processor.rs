use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke_signed, invoke},
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
    program_error::ProgramError
};

use crate::{id, SETTINGS_SEED};
use crate::{instruction::StoreInstruction, state::Price, state::Settings, error::PriceError};



pub struct Processor;

impl Processor {
    pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        msg!("price: {:?}", input);
        let instruction = StoreInstruction::try_from_slice(input)?;
        match instruction {
            StoreInstruction::InitializeStore => Self::process_initialize_store(accounts),
            StoreInstruction::UpdateSettings {
                admin,
                updated_price,
            } => Self::process_update_settings(accounts, admin, updated_price),
            StoreInstruction::Buy { amount } => Self::process_buy(accounts),
            StoreInstruction::Sell { amount } => Self::process_sell(accounts)
        }
    }

    fn process_initialize_store(accounts: &[AccountInfo]) -> ProgramResult {
        msg!("process_price");
        let acc_iter = &mut accounts.iter();
        let user_info = next_account_info(acc_iter)?;
        let price_info = next_account_info(acc_iter)?;
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

    fn process_buy(
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let acc_iter = &mut accounts.iter();
        let store_info = next_account_info(acc_iter)?;
        let store_token_info = next_account_info(acc_iter)?;
        let user_info = next_account_info(acc_iter)?;
        let user_token_info = next_account_info(acc_iter)?;
        let token_info = next_account_info(acc_iter)?;

        let ix = spl_token::instruction::transfer(
            token_info.key,
            store_token_info.key,
            user_token_info.key,
            store_info.key,
            &[store_info.key],
            10,
        )?;
        invoke(
            &ix,
            &[
                store_token_info.clone(),
                user_token_info.clone(),
                store_info.clone(),
                token_info.clone(),
            ],
        )?;
        let ixs = system_instruction::transfer(
            user_info.key,
            store_info.key,
            10,
        );

        invoke(
            &ixs,
            &[user_info.clone(), store_info.clone()]
        );
        Ok(())
    }
    fn process_sell(
        accounts: &[AccountInfo]
    ) -> ProgramResult {
        let acc_iter = &mut accounts.iter();
        let store_info = next_account_info(acc_iter)?;
        let store_token_info = next_account_info(acc_iter)?;
        let user_info = next_account_info(acc_iter)?;
        let user_token_info = next_account_info(acc_iter)?;
        let token_info = next_account_info(acc_iter)?;

        let ix = spl_token::instruction::transfer(
            token_info.key,
            user_token_info.key,
            store_token_info.key,
            user_info.key,
            &[user_info.key],
            10,
        )?;
        invoke(
            &ix,
            &[
                user_token_info.clone(),
                store_token_info.clone(),
                user_info.clone(),
                token_info.clone(),
            ],
        )?;
        let ixs = system_instruction::transfer(
            store_info.key,
            user_info.key,
            10,
        );

        invoke(
            &ixs,
            &[store_info.clone(), user_info.clone()]
        );
        Ok(())
    }
}


