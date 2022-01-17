
use borsh::{BorshDeserialize, BorshSerialize};
use solana_store::{entrypoint::process_instruction, id, instruction::StoreInstruction };
use solana_store::{
    state::{Price, Settings},
    PRICE_SEED,
};
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use solana_program_test::{processor, tokio, ProgramTest, ProgramTestContext};
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

struct Env {
    ctx: ProgramTestContext,
    admin: Keypair,
    user: Keypair,
}

impl Env {
    async fn new() -> Self {
        let program_test = ProgramTest::new("solana_store", id(), processor!(process_instruction));
        let mut ctx = program_test.start_with_context().await;

        let admin = Keypair::new();
        let user = Keypair::new();

        // credit admin and user accounts
        ctx.banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[
                    system_instruction::transfer(
                        &ctx.payer.pubkey(),
                        &admin.pubkey(),
                        1_000_000_000,
                    ),
                    system_instruction::transfer(
                        &ctx.payer.pubkey(),
                        &user.pubkey(),
                        1_000_000_000,
                    ),
                ],
                Some(&ctx.payer.pubkey()),
                &[&ctx.payer],
                ctx.last_blockhash,
            ))
            .await
            .unwrap();

        // init settings account
        let tx = Transaction::new_signed_with_payer(
            &[StoreInstruction::update_settings(
                &admin.pubkey(),
                admin.pubkey().to_bytes(),
                10,
            )],
            Some(&admin.pubkey()),
            &[&admin],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(tx).await.unwrap();

        let acc =
            ctx.banks_client.get_account(Settings::get_settings_pub()).await.unwrap().unwrap();
        let settings = Settings::try_from_slice(acc.data.as_slice()).unwrap();
        assert_eq!(settings.updated_price, 10);

        let space = Price { counter: 0, value: 25 }.try_to_vec().unwrap().len();
        let rent = ctx.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(space);
        let ix = system_instruction::create_account_with_seed(
            &user.pubkey(),
            &Price::get_price_pubkey(&user.pubkey()),
            &user.pubkey(),
            PRICE_SEED,
            lamports,
            space as u64,
            &id(),
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(tx).await.unwrap();

        Env { ctx, admin, user }
    }
}
#[tokio::test]
async fn test_price() {
    let mut env = Env::new().await;

    let tx = Transaction::new_signed_with_payer(
        &[StoreInstruction::initialize_store(&env.user.pubkey())],
        Some(&env.user.pubkey()),
        &[&env.user, &env.user],
        env.ctx.last_blockhash,
    );
    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(Price::get_price_pubkey(&env.user.pubkey()))
        .await
        .unwrap()
        .unwrap();
    let price = Price::try_from_slice(acc.data.as_slice()).unwrap();
    assert_eq!(price.counter, 1);
    assert_eq!(price.value, 15);
    println!("price is {:?}", price);
}

#[tokio::test]
async fn test_update_settings() {
    let mut env = Env::new().await;

    let tx = Transaction::new_signed_with_payer(
        &[StoreInstruction::update_settings(
            &env.admin.pubkey(),
            *&env.admin.pubkey().to_bytes(),
            11,
        )],
        Some(&env.admin.pubkey()),
        &[&env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc =
        env.ctx.banks_client.get_account(Settings::get_settings_pub()).await.unwrap().unwrap();
    let settings = Settings::try_from_slice(&acc.data.as_slice()).unwrap();
    assert_eq!(settings.updated_price, 11);
}
