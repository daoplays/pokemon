pub mod utils;
pub mod state;

use std::env;
use std::str::FromStr;
use crate::state::{Result};

use solana_client::rpc_client::RpcClient;
use solana_program::{pubkey::Pubkey, system_program};
use solana_sdk::{
    signature::Keypair, signer::Signer,
    instruction::{AccountMeta, Instruction},
    transaction::Transaction, signer::keypair::read_keypair_file
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_transaction_status::UiTransactionEncoding;
use spl_associated_token_account::{get_associated_token_address};
use enum_map::{enum_map, Enum};


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Enum, Copy)]
pub enum Charity {

    EvidenceAction,
    GirlsWhoCode,   
    OneTreePlanted,
    OutrightActionInt,
    TheLifeYouCanSave,
    InvalidCharity,
    UkraineERF,
    WaterOrg
    
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct ICOMeta {
    pub amount_charity : u64,
    pub amount_dao : u64,
    pub charity : Charity
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct CreateAccountMeta {
    pub amount : u64,
    pub supporter_amount : u64
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum ICOInstruction {

    InitICO {
        metadata: CreateAccountMeta
    },

    JoinICO {
        metadata: ICOMeta
    },

    EndICO

}


const URL: &str = "https://api.mainnet-beta.solana.com";

fn match_charity(index :  u8) ->  Charity 
{
    match index {
        0 => Charity::EvidenceAction,
        1 => Charity::GirlsWhoCode,
        2 => Charity::OneTreePlanted,
        3 => Charity::OutrightActionInt,
        4 => Charity::TheLifeYouCanSave,
        5 => Charity::UkraineERF,
        6 => Charity::WaterOrg,
        _ => Charity::InvalidCharity
    }
}

fn main() {

    let args: Vec<String> = env::args().collect();
    let key_file = &args[1];
    let function = &args[2];

    if function == "init_data_account" {

        let amount_arg = &args[3];
        let amount: u64 = amount_arg.parse().unwrap();

        let supporter_amount_arg = &args[4];
        let supporter_amount: u64 = supporter_amount_arg.parse().unwrap();

        if let Err(err) = init_pda_account(key_file, amount, supporter_amount) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }
    else if function == "join_ico" {
        let charity_arg = &args[3];
        let amount_charity_arg = &args[4];
        let amount_dao_arg = &args[5];

        let charity_index : u8 = charity_arg.parse().unwrap();
        let charity = match_charity(charity_index);
        let amount_charity: u64 = amount_charity_arg.parse().unwrap();
        let amount_dao: u64 = amount_dao_arg.parse().unwrap();

        if let Err(err) = join_ico(key_file, charity, amount_charity, amount_dao) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }

    }
    else if function == "end_ico" {

        if let Err(err) = end_ico(key_file) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }

    }
    

}

pub fn create_data_account(
    creator: &Keypair,
    program: &Pubkey,
    connection: &RpcClient,
    amount : u64,
    supporter_amount : u64
) -> Result<()> {

    let (expected_pda, bump_seed) = Pubkey::find_program_address(&[b"launch_account"], program);
    let mint_address = Pubkey::from_str("6PRgpKnwT9xgGF7cgS7ZMkPBeQmd5mdS97eg26ir8Kki").unwrap();
    let program_token_address = get_associated_token_address(
        &expected_pda, 
        &mint_address
    );

    let my_token_address = get_associated_token_address(
        &creator.pubkey(), 
        &mint_address
    );

    let supporter_mint_address = Pubkey::from_str("7B1yoU3EsbABt1kNXcJLeJRT8jwPy9rZfhrhWzuCA9Fq").unwrap();
    let program_supporter_token_address = get_associated_token_address(
        &expected_pda, 
        &supporter_mint_address
    );

    let my_supporter_token_address = get_associated_token_address(
        &creator.pubkey(), 
        &supporter_mint_address
    );

    println!("pda: {} {}", expected_pda, bump_seed);
    println!("token_address: {} {} {}", program_token_address, my_token_address, my_supporter_token_address);

    let meta_data =  CreateAccountMeta{amount : amount, supporter_amount: supporter_amount};

    let instruction = Instruction::new_with_borsh(
        *program,
        &ICOInstruction::InitICO{metadata : meta_data},
        vec![
            AccountMeta::new_readonly(creator.pubkey(), true),
            AccountMeta::new(expected_pda, false),

            AccountMeta::new(my_token_address, false),
            AccountMeta::new(program_token_address, false),
            AccountMeta::new_readonly(mint_address, false),

            AccountMeta::new(my_supporter_token_address, false),
            AccountMeta::new(program_supporter_token_address, false),
            AccountMeta::new_readonly(supporter_mint_address, false),

            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new(solana_sdk::system_program::id(), false)
        ],
    );

    let signers = [creator];
    let instructions = vec![instruction];
    let recent_hash = connection.get_latest_blockhash()?;

    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&creator.pubkey()),
        &signers,
        recent_hash,
    );


    let signature = connection.send_and_confirm_transaction(&txn)?;
    println!("signature: {}", signature);
    let response = connection.get_transaction(&signature, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", response);

    Ok(())
}

fn init_pda_account(key_file: &String, amount : u64, supporter_amount : u64) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str("GwsxvpsHURySgnLrkMcnYuSH2Sbd4v9eZwB5ruiVxgjE").unwrap();
  
    create_data_account(&wallet, &program, &connection, amount, supporter_amount)?;

    Ok(println!("Success!"))
}

fn join_ico(key_file: &String, charity : Charity, amount_charity  : u64, amount_dao  : u64) -> Result<()> {

    println!("In join_ico");
    
    let charity_map = enum_map!{
        Charity::EvidenceAction => "9fF5EQV6FVy7V5SaHBXfAaTUBvuyimQ9X3jarc2mRHzi",
        Charity::GirlsWhoCode => "5qrmDeRFhBTnEkqJsRKJAkTJzrZnyC9bWmRhL6RZqWt1",
        Charity::OneTreePlanted => "GeCaNYhRswBFoTxtNaf9wKYJEBZoxHa9Fao6aQKzDDo2",
        Charity::OutrightActionInt => "AiY4t79umvBqGvR43f5rL8jR8F2JZwG87mB55adAF2cf",
        Charity::TheLifeYouCanSave => "8qQpHYjLkNiKvLtFzrjzgFZfveNJZ9AnQuBUoQj1t3DB",
        Charity::UkraineERF  => "E6TPLh77cx9b5aWsmxM8geit2PBLVEBVAvF6ye9Qe4ZQ",
        Charity::WaterOrg => "5UNSVwtiSdfsCbJokL4fHtzV28mVNi8fQkMjPQw6v7Xd",
        Charity::InvalidCharity => "NULL"
    };

    let wallet = read_keypair_file(key_file).unwrap();
    let program = Pubkey::from_str("GwsxvpsHURySgnLrkMcnYuSH2Sbd4v9eZwB5ruiVxgjE").unwrap();


    let (expected_pda, _bump_seed) = Pubkey::find_program_address(&[b"launch_account"], &program);

    let mint_address = Pubkey::from_str("6PRgpKnwT9xgGF7cgS7ZMkPBeQmd5mdS97eg26ir8Kki").unwrap();
    let program_token_address = get_associated_token_address(
        &expected_pda, 
        &mint_address
    );
    let my_token_address = get_associated_token_address(
        &wallet.pubkey(), 
        &mint_address
    );

    let supporter_mint_address = Pubkey::from_str("7B1yoU3EsbABt1kNXcJLeJRT8jwPy9rZfhrhWzuCA9Fq").unwrap();
    let program_supporter_token_address = get_associated_token_address(
        &expected_pda, 
        &supporter_mint_address
    );

    let my_supporter_token_address = get_associated_token_address(
        &wallet.pubkey(), 
        &supporter_mint_address
    );


    let daoplays  = Pubkey::from_str("FxVpjJ5AGY6cfCwZQP5v8QBfS4J2NPa62HbGh1Fu2LpD").unwrap();


    if charity == Charity::InvalidCharity {
        return Ok(println!("InvalidCharity!"));
    }

    let charity_key = Pubkey::from_str(charity_map[charity]).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    println!("wallet {}", wallet.pubkey().to_string());
    println!("charity_key  {}", charity_key.to_string());
    println!("mint_address {}", mint_address.to_string());
    println!("program_token_address {}", program_token_address.to_string());
    println!("my_token_address {}", my_token_address.to_string());
    println!("daoplays {}", daoplays.to_string());
    println!("expected_pda {}\n", expected_pda.to_string());

    let meta_data =  ICOMeta{charity : charity, amount_charity : amount_charity,  amount_dao : amount_dao};

    let instruction = Instruction::new_with_borsh(
        program,
        &ICOInstruction::JoinICO{metadata : meta_data},
        vec![
            AccountMeta::new(wallet.pubkey(), true),
            AccountMeta::new(my_token_address, false),
            AccountMeta::new(my_supporter_token_address, false),

            AccountMeta::new(expected_pda, false),
            AccountMeta::new(program_token_address, false),
            AccountMeta::new(program_supporter_token_address, false),

            AccountMeta::new(charity_key, false),
            AccountMeta::new(daoplays, false),

            AccountMeta::new_readonly(mint_address, false),
            AccountMeta::new_readonly(supporter_mint_address, false),

            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new_readonly(system_program::id(), false)
        ],
    );

    let signers = [&wallet];
    let instructions = vec![instruction];
    let recent_hash = connection.get_latest_blockhash()?;

    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&wallet.pubkey()),
        &signers,
        recent_hash,
    );


    //let signature = connection.send_and_confirm_transaction(&txn)?;
    let signature = connection.simulate_transaction(&txn)?;
    println!("signature: {:?}", signature);
    //let response = connection.get_transaction(&signature, UiTransactionEncoding::Json)?;
    //println!("result: {:#?}", response);

    Ok(println!("Success!"))
}


fn end_ico(key_file: &String) -> Result<()> {


    let wallet = read_keypair_file(key_file).unwrap();
    let program = Pubkey::from_str("GwsxvpsHURySgnLrkMcnYuSH2Sbd4v9eZwB5ruiVxgjE").unwrap();


    let (expected_pda, _bump_seed) = Pubkey::find_program_address(&[b"launch_account"], &program);
    let mint_address = Pubkey::from_str("6PRgpKnwT9xgGF7cgS7ZMkPBeQmd5mdS97eg26ir8Kki").unwrap();
    let program_token_address = get_associated_token_address(
        &expected_pda, 
        &mint_address
    );
    let my_token_address = get_associated_token_address(
        &wallet.pubkey(), 
        &mint_address
    );

    let supporter_mint_address = Pubkey::from_str("7B1yoU3EsbABt1kNXcJLeJRT8jwPy9rZfhrhWzuCA9Fq").unwrap();
    let program_supporter_token_address = get_associated_token_address(
        &expected_pda, 
        &supporter_mint_address
    );

    let my_supporter_token_address = get_associated_token_address(
        &wallet.pubkey(), 
        &supporter_mint_address
    );

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    println!("wallet {}", wallet.pubkey().to_string());
    println!("mint_address {}", mint_address.to_string());
    println!("program_token_address {}", program_token_address.to_string());
    println!("my_token_address {}", my_token_address.to_string());
    println!("expected_pda {}\n", expected_pda.to_string());

    let instruction = Instruction::new_with_borsh(
        program,
        &ICOInstruction::EndICO,
        vec![
            AccountMeta::new(wallet.pubkey(), true),
            AccountMeta::new(my_token_address, false),
            AccountMeta::new(my_supporter_token_address, false),

            AccountMeta::new(expected_pda, false),
            AccountMeta::new(program_token_address, false),
            AccountMeta::new(program_supporter_token_address, false),

            AccountMeta::new_readonly(mint_address, false),
            AccountMeta::new_readonly(supporter_mint_address, false),

            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false)

        ],
    );

    let signers = [&wallet];
    let instructions = vec![instruction];
    let recent_hash = connection.get_latest_blockhash()?;

    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&wallet.pubkey()),
        &signers,
        recent_hash,
    );


    let signature = connection.send_and_confirm_transaction(&txn)?;
    println!("signature: {}", signature);
    let response = connection.get_transaction(&signature, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", response);

    Ok(println!("Success!"))
}
