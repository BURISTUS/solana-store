use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};


#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct State {
    pub counter: u16,
    pub price: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InstructionData {
    pub updated_price: u64,
}

pub fn process_instruction(_program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    msg!("update-state start: {:?}", instruction_data);
    let account_info_iter = &mut accounts.iter();
    let account = next_account_info(account_info_iter)?;

    msg!("account: {:?}", account);
    let instruction = InstructionData::try_from_slice(instruction_data)?;
    msg!("instruction: {:?}", instruction);

    let mut state = State::try_from_slice(&account.data.borrow())?;
    msg!("prev state: {:?}", state);

    state.counter += 1;
    state.price = instruction.updated_price;
    state.serialize(&mut &mut account.data.borrow_mut()[..]);
    msg!("new state: {:?}", state);

    msg!("data: {:?}", &account.data);

    msg!("update-state finish");
    Ok(())
}
