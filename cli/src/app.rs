use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_client::RpcClient,
    rpc_request,
};
use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_token::state::{Account, Mint};
use std::convert::TryInto;
use solana_sdk::commitment_config::CommitmentConfig;

pub struct Client {
    program_id: Pubkey,
    rpc_client: RpcClient,
}

impl Client {
    pub fn new(program_id: &Pubkey, url: &str) -> Client {
        let rpc_client =
            RpcClient::new_with_commitment(url.to_owned(), CommitmentConfig::confirmed());

        Client{
            program_id: *program_id,
            rpc_client
        }
    }

    fn create_account<T: Pack>(
        &self,
        payer: &dyn Signer,
        owner: &Pubkey,
    ) -> Result<Pubkey, ClientError> {
        let account = Keypair::new();
        let space = T::LEN;
        let lamports = self
            .rpc_client
            .get_minimum_balance_for_rent_exemption(space)?;

        let ix = create_account(
            &payer.pubkey(),
            &account.pubkey(),
            lamports,
            space.try_into().unwrap(),
            owner,
        );

        let blockhash = self.rpc_client.get_recent_blockhash()?.0;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &vec![payer, &account],
            blockhash,
        );

        self.rpc_client
            .send_and_confirm_transaction(&tx)
            .map(|_| account.pubkey())
    }

    pub fn create_mint_account(&self, payer: &dyn Signer) -> Result<Pubkey, ClientError> {
        self.create_account::<Mint>(payer, &spl_token::ID)
    }

    pub fn create_token_account(&self, payer: &dyn Signer) -> Result<Pubkey, ClientError> {
        self.create_account::<Account>(payer, &spl_token::ID)
    }

}

