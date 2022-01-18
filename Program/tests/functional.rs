
use borsh::{BorshDeserialize, BorshSerialize};
use solana_store::{entrypoint::process_instruction, id, instruction::StoreInstruction };
use solana_store::{
    state::{Price, Settings},
    PRICE_SEED,
};
use solana_program::pubkey::Pubkey;
use solana_program_test::{processor, tokio, ProgramTest, ProgramTestContext};
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::{
    hash::Hash,
    instruction::InstructionError,
    native_token::sol_to_lamports,
    program_pack::Pack,
    system_instruction::{self},
};
use solana_program_test::*;
use solana_sdk::{ transaction::TransactionError, transport::TransportError};
use spl_token::{error::TokenError, ui_amount_to_amount};

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
            &[StoreInstruction::update_price(
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
        &[StoreInstruction::update_price(
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




async fn create_token_mint(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    mint_rent: u64,
    decimals: u8,
    mint: &Keypair,
    mint_authority: &Pubkey,
) -> Result<(), TransportError> {
    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint.pubkey(),
                mint_rent,
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                mint_authority,
                None,
                decimals,
            )
                .unwrap(),
        ],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, mint], *recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

async fn create_token_account(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    account: &Keypair,
    account_rent: u64,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Result<(), TransportError> {
    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &account.pubkey(),
                account_rent,
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                mint,
                owner,
            )
                .unwrap(),
        ],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, account], *recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

async fn mint_token(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    amount: u64,
    mint: &Pubkey,
    account: &Pubkey,
    owner: &Keypair,
) -> Result<(), TransportError> {
    let mut transaction = Transaction::new_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            account,
            &owner.pubkey(),
            &[],
            amount,
        )
            .unwrap()],
        Some(&payer.pubkey()),
    );
    println!("transaction in {:?}", transaction);
    transaction.sign(&[payer, owner], *recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

#[tokio::test]
async fn test_transaction() {
    let program = ProgramTest::new("solana_store", id(), processor!(process_instruction));
    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let rent = banks_client.get_rent().await.unwrap();
    let account_rent = rent.minimum_balance(spl_token::state::Account::LEN);
    let mint_rent = rent.minimum_balance(spl_token::state::Mint::LEN);

    let custom_token_mint = Keypair::new();
    println!("{:?}", custom_token_mint);
    let custom_mint_authority = Keypair::new();
    println!("{:?}", custom_mint_authority);
    let decimals = 9;

    let pool_custom_token_acc = Keypair::new();
    let pool_owner = Keypair::new();
    let user_token_account = Keypair::new();

    let user_initial_token_ui_amount = 500000.0;
    let user_initial_token_amount = ui_amount_to_amount(user_initial_token_ui_amount, decimals);
    let user_wallet = Keypair::new();
    let user_token_account = Keypair::new();

    create_token_mint(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        mint_rent,
        decimals,
        &custom_token_mint,
        &custom_mint_authority.pubkey(),
    )
        .await
        .unwrap();

    create_token_account(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &pool_custom_token_acc,
        account_rent,
        &custom_token_mint.pubkey(),
        &pool_owner.pubkey(),
    )
        .await
        .unwrap();

    create_token_account(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &user_token_account,
        account_rent,
        &custom_token_mint.pubkey(),
        &user_wallet.pubkey(),
    )
        .await
        .unwrap();

    mint_token(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        user_initial_token_amount,
        &custom_token_mint.pubkey(),
        &pool_custom_token_acc.pubkey(),
        &custom_mint_authority,
    )
        .await
        .unwrap();

    println!("user token account {:?}", user_initial_token_amount);
    println!("custom token");

    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            id(),
            &(),
            vec![
                AccountMeta::new(pool_owner.pubkey(), true),
                AccountMeta::new(pool_custom_token_acc.pubkey(), false),
                AccountMeta::new(user_token_account.pubkey(), false),
                AccountMeta::new(custom_token_mint.pubkey(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &pool_owner], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}
