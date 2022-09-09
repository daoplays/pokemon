use borsh::{BorshDeserialize, BorshSerialize};
use std::str::FromStr;
use crate::state::{JoinMeta, InitMeta, Charity, TokenLaunchData};
use enum_map::{enum_map, EnumMap};
use crate::accounts;
use crate::utils;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,msg,
    program_error::ProgramError,
    program::invoke,
    system_instruction, program_pack::Pack
};

use spl_associated_token_account::{get_associated_token_address};


use crate::{instruction::TokenLaunchInstruction};

pub struct Processor;
impl Processor {
    
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

        let instruction = TokenLaunchInstruction::try_from_slice(&instruction_data[..])?;

        match instruction {
            TokenLaunchInstruction::InitTokenLaunch {metadata} => {

                Self::init_token_launch(program_id, accounts, metadata)
            },
            TokenLaunchInstruction::JoinTokenLaunch {metadata} => {

                Self::join_token_launch(program_id, accounts, metadata)
            },
            TokenLaunchInstruction::EndTokenLaunch => {
                Self::end_token_launch(program_id, accounts)
            }
        }
    } 

    fn init_token_launch(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        metadata : InitMeta
    ) ->ProgramResult 
    {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed eight accounts, get them all first and then check their value is as expected
        let funding_account_info = next_account_info(account_info_iter)?;
        let program_derived_account_info = next_account_info(account_info_iter)?;
        let token_source_account_info = next_account_info(account_info_iter)?;
        let program_token_account_info = next_account_info(account_info_iter)?;
        let token_mint_account_info = next_account_info(account_info_iter)?;

        let supporters_token_source_account_info = next_account_info(account_info_iter)?;
        let program_supporters_token_account_info = next_account_info(account_info_iter)?;
        let supporters_token_mint_account_info = next_account_info(account_info_iter)?;

        let token_program_account_info = next_account_info(account_info_iter)?;
        let associated_token_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;

        // the first account should be the funding account and should be a signer
        if !funding_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // only we should be able to call this function
        if funding_account_info.key != &accounts::get_expected_daoplays_key() {
            msg!("expected first account to be a daoplays account  {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the second account is the program derived address which we can verify with find_program_address
        let (expected_pda_key, bump_seed) = accounts::get_expected_program_address_key(program_id);
         
        if program_derived_account_info.key != &expected_pda_key {
            msg!("expected second account to be PDA {}", expected_pda_key);
            return Err(ProgramError::InvalidAccountData);
        }
  
        // the third account is the source of the tokens which we can verify with get_associated_token_address
        if token_source_account_info.key != &accounts::get_expected_daoplays_token_key() {
            msg!("expected third account to be the funder's token account {}", accounts::get_expected_daoplays_token_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the fourth account is the program's token account
        if program_token_account_info.key != &accounts::get_expected_program_token_key(program_id) {
            msg!("expected fourth account to be the program's token account {}", accounts::get_expected_program_token_key(program_id));
            return Err(ProgramError::InvalidAccountData);
        }

        // the fifth account is the mint address for the token
        if token_mint_account_info.key != &accounts::get_expected_token_mint_key() {
            msg!("expected fifth account to be the token's mint account {}", accounts::get_expected_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the sixth account is the source of the supporter tokens
        if supporters_token_source_account_info.key != &accounts::get_expected_daoplays_supporters_token_key() {
            msg!("expected sixth account to be the funder's supporter token account {}", accounts::get_expected_daoplays_supporters_token_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the seventh account is the program's supporters token account
        if program_supporters_token_account_info.key != &accounts::get_expected_program_supporters_token_key(program_id) {
            msg!("expected seventh account to be the program's supporters token account {}", accounts::get_expected_program_supporters_token_key(program_id));
            return Err(ProgramError::InvalidAccountData);
        }

        // the eighth account is the mint address for the supporter token
        if supporters_token_mint_account_info.key != &accounts::get_expected_supporters_token_mint_key() {
            msg!("expected eighth account to be the supporter token's mint account {}", accounts::get_expected_supporters_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the ninth account is the token_program
        if token_program_account_info.key != &spl_token::id() {
            msg!("expected ninth account to be the token program {}", spl_token::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the tenth account is the associated_token_program
        if associated_token_account_info.key != &spl_associated_token_account::id() {
            msg!("expected tenth account to be the associated token program {}", spl_associated_token_account::id());
            return Err(ProgramError::InvalidAccountData);
        }
        
        // the eleventh and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected eleventh account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }
        


        utils::create_program_account(
            funding_account_info,
            program_derived_account_info,
            program_id,
            bump_seed
        )?;
        

     
        utils::create_token_account(
            funding_account_info,
            program_derived_account_info,
            token_mint_account_info,
            program_token_account_info,
            token_program_account_info
        )?;
        


        utils::create_token_account(
            funding_account_info,
            program_derived_account_info,
            supporters_token_mint_account_info,
            program_supporters_token_account_info,
            token_program_account_info
        )?;
        

        utils::transfer_tokens(
            metadata.amount,
            token_source_account_info,
            program_token_account_info,
            funding_account_info,
            token_program_account_info,
            bump_seed
    
        )?;

        utils::transfer_tokens(
            metadata.supporter_amount,
            supporters_token_source_account_info,
            program_supporters_token_account_info,
            funding_account_info,
            token_program_account_info,
            bump_seed
    
        )?;


        Ok(())

    }

    fn join_token_launch(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        meta: JoinMeta
        ) ->ProgramResult {



        let charity_key_map = enum_map!{
            Charity::EvidenceAction => "9fF5EQV6FVy7V5SaHBXfAaTUBvuyimQ9X3jarc2mRHzi",
            Charity::GirlsWhoCode => "5qrmDeRFhBTnEkqJsRKJAkTJzrZnyC9bWmRhL6RZqWt1",
            Charity::OneTreePlanted => "GeCaNYhRswBFoTxtNaf9wKYJEBZoxHa9Fao6aQKzDDo2",
            Charity::OutrightActionInt => "AiY4t79umvBqGvR43f5rL8jR8F2JZwG87mB55adAF2cf",
            Charity::TheLifeYouCanSave => "8qQpHYjLkNiKvLtFzrjzgFZfveNJZ9AnQuBUoQj1t3DB",
            Charity::UkraineERF  => "E6TPLh77cx9b5aWsmxM8geit2PBLVEBVAvF6ye9Qe4ZQ",
            Charity::WaterOrg => "5UNSVwtiSdfsCbJokL4fHtzV28mVNi8fQkMjPQw6v7Xd"
        };

        let charity_index_map: EnumMap<Charity, usize> = enum_map!{
            Charity::EvidenceAction => 0,
            Charity::GirlsWhoCode => 1,
            Charity::OneTreePlanted => 2,
            Charity::OutrightActionInt => 3,
            Charity::TheLifeYouCanSave => 4,
            Charity::UkraineERF => 5,
            Charity::WaterOrg => 6
        };

        // get the accounts
        let account_info_iter = &mut accounts.iter();

        let joiner_account_info = next_account_info(account_info_iter)?;
        let joiner_token_account_info = next_account_info(account_info_iter)?;
        let joiner_supporters_token_account_info = next_account_info(account_info_iter)?;
       
        let program_data_account_info = next_account_info(account_info_iter)?;
        let program_token_account_info = next_account_info(account_info_iter)?;
        let program_supporters_token_account_info = next_account_info(account_info_iter)?;
        
        let charity_account_info = next_account_info(account_info_iter)?;
        let daoplays_account_info = next_account_info(account_info_iter)?;

        let token_mint_account_info = next_account_info(account_info_iter)?;
        let supporters_token_mint_account_info = next_account_info(account_info_iter)?;

        let token_program_account_info = next_account_info(account_info_iter)?;
        let associated_token_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;


        // now check all the accounts
        // the joiners account should be the signer
        if !joiner_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // the second account should be the joiners associated token account
        let expected_joiner_token_key = get_associated_token_address(
            &joiner_account_info.key, 
            &token_mint_account_info.key
        );

        if joiner_token_account_info.key != &expected_joiner_token_key
        { 
            msg!("expected second account to be the joiner's associated token account {}", expected_joiner_token_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the third account should be the joiners supporter associated token account
        let expected_joiner_supporters_token_key = get_associated_token_address(
            &joiner_account_info.key, 
            &supporters_token_mint_account_info.key
        );

        if joiner_supporters_token_account_info.key != &expected_joiner_supporters_token_key
        { 
            msg!("expected third account to be the joiner's supporter associated token account {}", expected_joiner_supporters_token_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the fourth account should be the programs derived account
        let (expected_pda_key, bump_seed) = accounts::get_expected_program_address_key(program_id);

        if program_data_account_info.key != &expected_pda_key
        { 
            msg!("expected fourth account to be the programs derived account {}", expected_pda_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the fifth account should be the programs token address
        if program_token_account_info.key != &accounts::get_expected_program_token_key(program_id)
        { 
            msg!("expected fifth account to be the programs token account {}", accounts::get_expected_program_token_key(program_id));
            return Err(ProgramError::InvalidAccountData); 
        }

        // the sixth account should be the programs token address
        if program_supporters_token_account_info.key != &accounts::get_expected_program_supporters_token_key(program_id)
        { 
            msg!("expected sixth account to be the programs supporter token account {}", accounts::get_expected_program_supporters_token_key(program_id));
            return Err(ProgramError::InvalidAccountData); 
        }

        // the seventh account is the charity SOL address, which we can check with the map
        let expected_charity_key = Pubkey::from_str(charity_key_map[meta.charity]).unwrap();

        if charity_account_info.key != &expected_charity_key
        {
            msg!("expected seventh account to be the chosen charities address {}", expected_charity_key);
            return Err(ProgramError::InvalidAccountData);
        }

        // the eighth account is the daoplays SOL address
         if daoplays_account_info.key != &accounts::get_expected_daoplays_key()
        {
            msg!("expected eighth account to be the daoplays address {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the ninth account is the mint address for the token
        if token_mint_account_info.key != &accounts::get_expected_token_mint_key()
        {
            msg!("expected ninth account to be the token mint address {}", accounts::get_expected_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the tenth account is the mint address for the supporters token
        if supporters_token_mint_account_info.key != &accounts::get_expected_supporters_token_mint_key()
        {
            msg!("expected tenth account to be the token mint address {}", accounts::get_expected_supporters_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }
         
        // the eleventh account is the token_program
        if token_program_account_info.key != &spl_token::id() {
            msg!("expected eleventh account to be the token program {}", spl_token::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the twelfth account is the associated_token_program
        if associated_token_account_info.key != &spl_associated_token_account::id() {
            msg!("expected twelfth account to be the associated token program {}", spl_associated_token_account::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the thirteenth and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected thirteenth account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }
 

        utils::create_token_account(
            joiner_account_info,
            joiner_account_info,
            token_mint_account_info,
            joiner_token_account_info,
            token_program_account_info
        )?;
        

        // check that this transaction is valid:
        // i) total amount should exceed the minimum
        // ii) joiner should not already have tokens
        // iii) program should have enough spare tokens

        
        msg!("Transfer {} {}", meta.amount_charity, meta.amount_dao);
        msg!("Balance {}", joiner_account_info.try_borrow_lamports()?);

        let min_amount : u64 = 100000;
        if meta.amount_charity + meta.amount_dao < min_amount {
            msg!("Amount paid is less than the minimum of 0.0001 SOL");
            return Err(ProgramError::InvalidArgument);
        }

        let program_token_account = spl_token::state::Account::unpack_unchecked(&program_token_account_info.try_borrow_data()?)?;
        let program_supporters_token_account = spl_token::state::Account::unpack_unchecked(&program_supporters_token_account_info.try_borrow_data()?)?;
        let joiner_token_account = spl_token::state::Account::unpack_unchecked(&joiner_token_account_info.try_borrow_data()?)?;

        msg!("token balances: {} {} {}", program_token_account.amount, program_supporters_token_account.amount, joiner_token_account.amount);

        if joiner_token_account.amount > 0 {
            msg!("Tokens already present in joiners account, thank you for taking part!");
            return Err(ProgramError::InvalidAccountData);
        }

        // get the data stored in the program account to access current state
        let mut current_state = TokenLaunchData::try_from_slice(&program_data_account_info.data.borrow()[..])?;

        // calculate the current average to see if this individual has paid more
        let mut current_average = 0;
        if current_state.n_donations > 0 {
            current_average = current_state.paid_total / current_state.n_donations;
        }
        let total_paid = meta.amount_charity + meta.amount_dao;
        let mut token_launch_amount : u64 = 1000;

        let mut supporter = false;
        // if they have then they get double!
        if total_paid > current_average {
            msg!("Thank you for paying over the average price!");

            token_launch_amount = 2000;
            supporter =  true;
        }
        
        // check if there are the required number of tokens remaining
        if program_token_account.amount < token_launch_amount {
            msg!("Insufficient tokens remaining in token launch");
            return Err(ProgramError::InvalidArgument);
        }

        // if we have made it this far the transaction we can try transferring the SOL
        invoke(
            &system_instruction::transfer(joiner_account_info.key, charity_account_info.key, meta.amount_charity),
            &[joiner_account_info.clone(), charity_account_info.clone()],
        )?;

        invoke(
            &system_instruction::transfer(joiner_account_info.key, daoplays_account_info.key, meta.amount_dao),
            &[joiner_account_info.clone(), daoplays_account_info.clone()],
        )?;

        // and finally transfer the tokens
        utils::transfer_tokens(
            token_launch_amount,
            program_token_account_info,
            joiner_token_account_info,
            program_data_account_info,
            token_program_account_info,
            bump_seed
        )?;

        if supporter && program_supporters_token_account.amount >= 1 {

             utils::create_token_account(
                joiner_account_info,
                joiner_account_info,
                supporters_token_mint_account_info,
                joiner_supporters_token_account_info,
                token_program_account_info
            )?;
            

            utils::transfer_tokens(
                1,
                program_supporters_token_account_info,
                joiner_supporters_token_account_info,
                program_data_account_info,
                token_program_account_info,
                bump_seed
            )?;
        }

        // update the data

        let charity_index = charity_index_map[meta.charity];

        current_state.charity_totals[charity_index] += meta.amount_charity;
        current_state.donated_total += meta.amount_charity;
        current_state.paid_total += total_paid;
        current_state.n_donations += 1;

        msg!("Updating current state: {} {} {} {}", current_state.charity_totals[charity_index], current_state.donated_total, current_state.paid_total,  current_state.n_donations);

        current_state.serialize(&mut &mut program_data_account_info.data.borrow_mut()[..])?;


        Ok(())
    }

    fn end_token_launch(
        program_id: &Pubkey,
        accounts: &[AccountInfo]
    ) ->ProgramResult {

        let account_info_iter = &mut accounts.iter();

        let daoplays_account_info = next_account_info(account_info_iter)?;
        let daoplays_token_account_info = next_account_info(account_info_iter)?;
        let daoplays_supporters_token_account_info = next_account_info(account_info_iter)?;

        let program_account_info = next_account_info(account_info_iter)?;
        let program_token_account_info = next_account_info(account_info_iter)?;
        let program_supporters_token_account_info = next_account_info(account_info_iter)?;

        let token_mint_account_info = next_account_info(account_info_iter)?;
        let supporters_token_mint_account_info = next_account_info(account_info_iter)?;

        let token_program_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;


        // the first account should be the funding account and should be a signer
        if !daoplays_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // only we should be able to call this function
        if daoplays_account_info.key != &accounts::get_expected_daoplays_key() {
            msg!("expected first account to be a daoplays account  {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the second account should be the daoplays token account we want to transfer back to
        if daoplays_token_account_info.key != &accounts::get_expected_daoplays_token_key()
        {
            msg!("expected second account to be a daoplays token account  {}", accounts::get_expected_daoplays_token_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the third account should be the daoplays supporters token account we want to transfer back to
        if daoplays_supporters_token_account_info.key != &accounts::get_expected_daoplays_supporters_token_key()
        {
            msg!("expected third account to be a daoplays supporters token account  {}", accounts::get_expected_daoplays_supporters_token_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the fourth account should be the program's derived address, which we can verify using find_program_address
        let (expected_program_account_key, bump_seed) = accounts::get_expected_program_address_key(program_id);

        if program_account_info.key != &expected_program_account_key
        {
            msg!("expected fourth account to be a program's derived account  {}", expected_program_account_key);
            return Err(ProgramError::InvalidAccountData);
        }
         
        // the fifth account should be the program's token account
        if program_token_account_info.key != &accounts::get_expected_program_token_key(program_id)
        {
            msg!("expected fifth account to be a program's token account  {}", accounts::get_expected_program_token_key(program_id));
            return Err(ProgramError::InvalidAccountData);
        }

        // the sixth account should be the programs supporters token account
        if program_supporters_token_account_info.key != &accounts::get_expected_program_supporters_token_key(program_id)
        {
            msg!("expected sixth account to be a daoplays supporters token account  {}", accounts::get_expected_program_supporters_token_key(program_id));
            return Err(ProgramError::InvalidAccountData);
        }
        
        // the seventh account is the mint address for the token
        if token_mint_account_info.key != &accounts::get_expected_token_mint_key() {
            msg!("expected seventh account to be the token's mint account {}", accounts::get_expected_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the eighth account is the mint address for the supporters token
        if supporters_token_mint_account_info.key != &accounts::get_expected_supporters_token_mint_key() {
            msg!("expected eighth account to be the token's mint account {}", accounts::get_expected_supporters_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the ninth should be the token program
        if token_program_account_info.key != &spl_token::id() {
            msg!("expected ninth account to be the token program");
            return Err(ProgramError::InvalidAccountData);
        }

        // the tenth and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected tenth account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // first close out the main token account
        utils::close_program_token_account(
            program_account_info,
            program_token_account_info,
            daoplays_account_info,
            daoplays_token_account_info,
            token_program_account_info,
            bump_seed
        )?;

        // now do the same thing for the supporters tokens
        utils::close_program_token_account(
            program_account_info,
            program_supporters_token_account_info,
            daoplays_account_info,
            daoplays_supporters_token_account_info,
            token_program_account_info,
            bump_seed
        )?;

        Ok(())

    }
}