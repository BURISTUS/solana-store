use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program_test::*;
use solana_sdk::{
    account::{Account, ReadableAccount},
    pubkey::Pubkey,
    signature::Signer,
    transaction::Transaction,
};
use solana_store::processor::{process_instruction, InstructionData, State};

#[tokio::test]
async fn test_program() {
    let program_id = Pubkey::new_unique();
    let state_pubkey = Pubkey::new_unique();

    let mut program_test = ProgramTest::new("solana_store", program_id, processor!(process_instruction));
    let data = State { counter: 0, price: 10}.try_to_vec().unwrap();
    program_test.add_account(state_pubkey, Account { lamports: 77777, data, owner: program_id, ..Account::default() });

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Call program 10 times
    for i in 0..10 {
        let init_data = InstructionData { updated_price: 109 + i as u64 }.try_to_vec().unwrap();
        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &init_data, vec![AccountMeta::new(state_pubkey, false)])],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
    }

    // Check counter in state account
    let state_account = banks_client.get_account(state_pubkey).await.unwrap().unwrap();
    println!("state_account: {:?}", state_account);
    let data = state_account.data();
    println!("z1: {:?}", data);
    let new_state = State::try_from_slice(data).unwrap();
    println!("new state: {:?}", new_state);

    assert_eq!(new_state.counter, 10);
}
