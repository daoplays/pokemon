pub mod state;

use crate::state::{Result, DPPInstruction, InitData, Charity, BidData, State, MAX_WINNERS};

use std::env;
use std::str::FromStr;
use solana_client::rpc_client::RpcClient;
use solana_program::{pubkey::Pubkey, rent, native_token::LAMPORTS_PER_SOL, system_program};
use solana_sdk::{
    signer::Signer,
    instruction::{AccountMeta, Instruction},
    transaction::Transaction, signer::keypair::read_keypair_file, commitment_config::{CommitmentConfig, CommitmentLevel}
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_transaction_status::UiTransactionEncoding;
use spl_associated_token_account::{get_associated_token_address};
use enum_map::{enum_map, Enum};

use std::time::Instant;


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

fn get_button_from_int(index: u64) -> state::Button {
    if index == 0 {
        return state::Button::A;
    } else if index == 1 {
        return state::Button::B;
    } else if index == 2 {
        return state::Button::Up;
    } else if index == 3 {
        return state::Button::Down;
    } else if index == 4 {
        return state::Button::Left;
    } else if index == 5 {
        return state::Button::Right;
    } else {
        return state::Button::Select;
    }

}

fn main() {

    let args: Vec<String> = env::args().collect();
    let key_file = &args[1];
    let function = &args[2];

    if function == "init_data_account" {

        let amount_arg = &args[3];
        let amount: u64 = amount_arg.parse().unwrap();

        if let Err(err) = create_accounts(key_file, amount) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }
    else if function == "place_bid" {
        let charity_arg = &args[3];
        let amount_charity_arg = &args[4];
        let amount_dao_arg = &args[5];

        let charity_index : u8 = charity_arg.parse().unwrap();
        let charity = match_charity(charity_index);
        let amount_charity: u64 = amount_charity_arg.parse().unwrap();
        let amount_dao: u64 = amount_dao_arg.parse().unwrap();

        if let Err(err) = place_bid(key_file, charity, amount_charity, amount_dao) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }

    }

    else if function == "monitor_data" {
        if let Err(err) = monitor_data(key_file) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

    else if function == "select_winners" {
        if let Err(err) = select_winners(key_file) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

    else if function == "push_button" {

        let index_arg = &args[3];
        let index: u64 = index_arg.parse().unwrap();
        let pressed_button = get_button_from_int(index);
        if let Err(err) = press_button(key_file, pressed_button) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }



}


fn create_accounts(key_file: &String, amount : u64) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str("GRxdexptfCKuXfGpTGREEjtwTrZPTwZSfdSXiWDC11me").unwrap();
  
    let (expected_pda, bump_seed) = Pubkey::find_program_address(&[b"token_account"], &program);
    let mint_address = Pubkey::from_str("6PRgpKnwT9xgGF7cgS7ZMkPBeQmd5mdS97eg26ir8Kki").unwrap();
    let program_token_address = get_associated_token_address(
        &expected_pda, 
        &mint_address
    );

    let my_token_address = get_associated_token_address(
        &wallet.pubkey(), 
        &mint_address
    );

    // in this function we need to create the data account for the program
    // we need to create the data account off chain because it is too big  (45kb)
   
    let data_account = Pubkey::create_with_seed(
                            &wallet.pubkey(),
                            "data_account",
                            &program,
                        )?;

    println!("pda: {} {}", expected_pda, bump_seed);
    println!("data account {}", data_account);
    println!("token_address: {} {}", program_token_address, my_token_address);

    //return Ok(());
    let data_account_balance = connection.get_balance(&data_account)?;
    println!("data account balance: {}", data_account_balance);
    // we need to create the data account off chain because it is too big
    // Check if the account has already been initialized
    if data_account_balance > 0 {
        println!("data account is already initialized. skipping");
        
    }
    else {

        println!("Creating programs data account");
        
        let data_size: usize = 49379;
        let space : u64 = data_size.try_into().unwrap();
        let lamports = rent::Rent::default().minimum_balance(data_size);
   
        println!("Require {} lamports for {} size data", (lamports as f64) / (LAMPORTS_PER_SOL as f64), data_size); 
        let instruction = solana_sdk::system_instruction::create_account_with_seed(
            &wallet.pubkey(),
            &data_account,
            &wallet.pubkey(),
            "data_account",
            lamports,
            space,
            &program,
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

    }
        

    let init_data =  InitData{amount : amount};

    let instruction = Instruction::new_with_borsh(
        program,
        &DPPInstruction::CreateDataAccount {init_data : init_data},
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(expected_pda, false),
            AccountMeta::new(data_account, false),

            AccountMeta::new(my_token_address, false),
            AccountMeta::new(program_token_address, false),
            AccountMeta::new_readonly(mint_address, false),

            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new(solana_sdk::system_program::id(), false)
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


fn place_bid(key_file: &String, charity : Charity, amount_charity  : u64, amount_dao  : u64) -> Result<()> {

    println!("In place_bid");
    let charity_key_map = enum_map!{
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
    let program = Pubkey::from_str("GRxdexptfCKuXfGpTGREEjtwTrZPTwZSfdSXiWDC11me").unwrap();
    let daoplays  = Pubkey::from_str("FxVpjJ5AGY6cfCwZQP5v8QBfS4J2NPa62HbGh1Fu2LpD").unwrap();

    let program_data_account = Pubkey::create_with_seed(
        &daoplays,
        "data_account",
        &program,
    )?;


    let (expected_pda, _bump_seed) = Pubkey::find_program_address(&[b"token_account"], &program);

    let mint_address = Pubkey::from_str("6PRgpKnwT9xgGF7cgS7ZMkPBeQmd5mdS97eg26ir8Kki").unwrap();
    let program_token_address = get_associated_token_address(
        &expected_pda, 
        &mint_address
    );
    let my_token_address = get_associated_token_address(
        &wallet.pubkey(), 
        &mint_address
    );

    let (expected_bidder_pda, _bidder_bump_seed) = Pubkey::find_program_address(&[&wallet.pubkey().to_bytes()], &program);

    if charity == Charity::InvalidCharity {
        return Ok(println!("InvalidCharity!"));
    }

    let charity_key = Pubkey::from_str(charity_key_map[charity]).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    println!("wallet {}", wallet.pubkey().to_string()); 
    println!("my_token_address {}", my_token_address.to_string());
    println!("expected_bidder_pda  {}\n", expected_bidder_pda.to_string());

    println!("daoplays {}", daoplays.to_string());
    println!("charity_key  {}", charity_key.to_string());

    println!("data account {}\n", program_data_account.to_string());
    println!("program_token_address {}", program_token_address.to_string());

    println!("mint_address {}", mint_address.to_string());


    let lpm : f64 = LAMPORTS_PER_SOL as f64;
    println!("total bid {} + {} = {}", (amount_charity as f64) / lpm, (amount_dao as f64) / lpm, ((amount_charity + amount_dao) as f64) / lpm);

    let bid_data =  BidData{charity : charity, amount_charity : amount_charity,  amount_dao : amount_dao};
    
    let instruction = Instruction::new_with_borsh(
        program,
        &DPPInstruction::PlaceBid { bid_data : bid_data},
        vec![
            AccountMeta::new(wallet.pubkey(), true),
            AccountMeta::new(my_token_address, false),
            AccountMeta::new(expected_bidder_pda, false),


            AccountMeta::new(daoplays, false),
            AccountMeta::new(charity_key, false),

            AccountMeta::new(program_data_account, false),
            AccountMeta::new(program_token_address, false),


            AccountMeta::new_readonly(mint_address, false),

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


    let signature = connection.send_and_confirm_transaction(&txn)?;
    println!("signature: {}", signature);
    let response = connection.get_transaction(&signature, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", response);

    Ok(println!("Success!"))
}


pub fn monitor_data(_key_file: &String) -> Result<()> {

    // (2) Create a new Keypair for the new account
    let program = Pubkey::from_str("GRxdexptfCKuXfGpTGREEjtwTrZPTwZSfdSXiWDC11me").unwrap();
    let daoplays  = Pubkey::from_str("FxVpjJ5AGY6cfCwZQP5v8QBfS4J2NPa62HbGh1Fu2LpD").unwrap();

    let program_data_account = Pubkey::create_with_seed(
        &daoplays,
        "data_account",
        &program,
    )?;

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let response = connection.get_account_data(&program_data_account)?;
    //println!("data in account: {}", data_pubkey);
    //println!("{:#?}", response);

    let current_state = State::try_from_slice(&response[..]).unwrap();

    println!("data: n_bidders {} bid_amount: {}", current_state.n_bidders, (current_state.total_bid_amount as f64) / (LAMPORTS_PER_SOL as f64));

    for i in 0..1024 {
        println!("bidders: {} {} {} {}", i, current_state.bid_keys[i], current_state.bid_amounts[i],  current_state.bid_times[i]);
    }

    let n_winners = current_state.n_winners;
    let winners = current_state.winners;

    println!("\n\nn_winners:  {}", n_winners);
    for i in 0..MAX_WINNERS {
        println!("winner: {} {}", i, winners[i]);
    }

    Ok(())
}


fn select_winners(key_file: &String) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str("GRxdexptfCKuXfGpTGREEjtwTrZPTwZSfdSXiWDC11me").unwrap();
    let daoplays  = Pubkey::from_str("FxVpjJ5AGY6cfCwZQP5v8QBfS4J2NPa62HbGh1Fu2LpD").unwrap();

    // in this function we need to create the data account for the program
    // we need to create the data account off chain because it is too big  (45kb)
    let (expected_pda, _bump_seed) = Pubkey::find_program_address(&[b"token_account"], &program);
    let mint_address = Pubkey::from_str("6PRgpKnwT9xgGF7cgS7ZMkPBeQmd5mdS97eg26ir8Kki").unwrap();
    let program_token_address = get_associated_token_address(
        &expected_pda, 
        &mint_address
    );
    let data_account = Pubkey::create_with_seed(
                            &daoplays,
                            "data_account",
                            &program,
                        )?;

    // we will use 3 streams, BTC,  ETH and SOL
    let btc_key =   Pubkey::from_str("GVXRSBjFk6e6J3NbVPXohDJetcTjaeeuykUpbQF8UoMU").unwrap();
    let eth_key =   Pubkey::from_str("JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB").unwrap();
    let sol_key =   Pubkey::from_str("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG").unwrap();

    println!("wallet {}",  wallet.pubkey().to_string());
    println!("btc_key {}",  btc_key.to_string());
    println!("eth_key {}",  eth_key.to_string());
    println!("sol_key {}",  sol_key.to_string());
    println!("data_account {}",  data_account.to_string());
    println!("program_token_address {}",  program_token_address.to_string());


    let instruction = Instruction::new_with_borsh(
        program,
        &DPPInstruction::SelectWinners,
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(btc_key, false),
            AccountMeta::new(eth_key, false),
            AccountMeta::new(sol_key, false),
            AccountMeta::new(data_account, false),
            AccountMeta::new(program_token_address, false)
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

    let data_response = connection.get_account_data(&data_account)?;
    let current_state = State::try_from_slice(&data_response[..]).unwrap();


    let n_winners = current_state.n_winners;
    let winners = current_state.winners;

    println!("\n\nn_winners:  {}", n_winners);
    for i in 0..MAX_WINNERS {
        println!("winner: {} {}", i, winners[i]);
    }
 
    let mut accounts : Vec<AccountMeta> = Vec::new();
    accounts.push(AccountMeta::new_readonly(wallet.pubkey(), true));
    accounts.push(AccountMeta::new(expected_pda, false));
    accounts.push(AccountMeta::new(program_token_address, false));
    accounts.push(AccountMeta::new(data_account, false));
    accounts.push(AccountMeta::new_readonly(spl_token::id(), false));
    for i in 0..(n_winners as usize) {
        accounts.push(AccountMeta::new(winners[i], false));
    }


    let send_instruction = Instruction::new_with_borsh(
        program,
        &DPPInstruction::SendTokens,
        accounts
    );


    let send_signers = [&wallet];
    let send_instructions = vec![send_instruction];
    let send_recent_hash = connection.get_latest_blockhash()?;

    let send_txn = Transaction::new_signed_with_payer(
        &send_instructions,
        Some(&wallet.pubkey()),
        &send_signers,
        send_recent_hash,
    );

    let send_signature = connection.send_and_confirm_transaction(&send_txn)?;
    println!("signature: {}", send_signature);
    let send_response = connection.get_transaction(&send_signature, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", send_response); 

  
    Ok(println!("Success!"))
}

fn press_button(key_file: &String, pressed_button: state::Button) -> Result<()> {

    let wallet = read_keypair_file(key_file).unwrap();

    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str("GRxdexptfCKuXfGpTGREEjtwTrZPTwZSfdSXiWDC11me").unwrap();
    let mint_address = Pubkey::from_str("6PRgpKnwT9xgGF7cgS7ZMkPBeQmd5mdS97eg26ir8Kki").unwrap();

    // devnet version:
    //let mint_address = Pubkey::from_str("CisHceikLeKxYiUqgDVduw2py2GEK71FTRykXGdwf22h").unwrap();

    let (expected_pda, _bump_seed) = Pubkey::find_program_address(&[b"token_account"], &program);

    let program_token_address = get_associated_token_address(
        &expected_pda, 
        &mint_address
    );
    let my_token_address = get_associated_token_address(
        &wallet.pubkey(), 
        &mint_address
    );

    let button: state::Button = pressed_button;
    let amount: u64 = 1;

    let button_data =  state::ButtonData{button: button, amount: amount};

    println!("wallet {}",wallet.pubkey());
    println!("my_token_address {}",my_token_address);
    println!("program_token_address {}",program_token_address);
    println!("mint_address {}",mint_address);
    println!("spl_token {}",spl_token::ID);


    let press_button_idx = Instruction::new_with_borsh(
        program,
        &DPPInstruction::PushButton{button_data},
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(my_token_address, false),
            AccountMeta::new(program_token_address, false),
            AccountMeta::new_readonly(mint_address, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
    );

    let signers = [&wallet];
    let instructions = vec![press_button_idx];
    let recent_hash = connection.get_latest_blockhash()?;

    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&wallet.pubkey()),
        &signers,
        recent_hash,
    );

    let commitment_config = CommitmentConfig{commitment : CommitmentLevel::Confirmed};

    let start = Instant::now();
    println!("sending transaction..");
    let signature = connection.send_transaction(&txn)?;
    let elapsed = start.elapsed();
    println!("transaction confirmed: signature: {}, duration {:?}", signature, elapsed);

   // let response = connection.get_transaction(&signature, UiTransactionEncoding::Json)?;
    //println!("result: {:#?}", response);

    Ok(println!("Success!"))

    
}
