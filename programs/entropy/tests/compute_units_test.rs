#![cfg(feature = "test-bpf")]

use {
    anchor_client::{
        anchor_lang::Discriminator,
        solana_sdk::{
            account::Account,
            commitment_config::CommitmentConfig,
            pubkey::Pubkey,
            signature::{Keypair, Signer},
            transaction::Transaction,
        },
        Client, Cluster,
    },
    solana_program_test::{tokio, ProgramTest},
    std::rc::Rc,
};

#[tokio::test]
async fn test_compute_units() {
    let mut pt = ProgramTest::new("entropy", entropy::id(), None);
    pt.set_compute_max_units(10_000_000_000);
    let (mut banks_client, payer, recent_blockhash) = pt.start().await;

    let client = Client::new_with_options(
        Cluster::Debug,
        Rc::new(Keypair::new()),
        CommitmentConfig::processed(),
    );
    let program = client.program(entropy::id());
    let prime_ix = program
        .request()
        .accounts(entropy::accounts::Prime {})
        .args(entropy::instruction::Prime {})
        .instructions()
        .unwrap()
        .pop()
        .unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[prime_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();
}
