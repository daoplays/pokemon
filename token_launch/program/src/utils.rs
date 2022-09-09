use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_pack::Pack, pubkey::Pubkey, rent,
    program_error::ProgramError, native_token::LAMPORTS_PER_SOL
};
use spl_associated_token_account::instruction::create_associated_token_account;
use crate::state::{get_state_size};

pub fn create_program_account<'a>(
    funding_account: &AccountInfo<'a>,
    pda : &AccountInfo<'a>,
    program_id :  &Pubkey,
    bump_seed : u8

) -> ProgramResult
{

     // Check if the account has already been initialized
     if **pda.try_borrow_lamports()? > 0 {
        msg!("This account is already initialized. skipping");
        return Ok(());
    }

    msg!("Creating programs derived account");

    let data_size = get_state_size();
    let space : u64 = data_size.try_into().unwrap();
    let lamports = rent::Rent::default().minimum_balance(data_size);

    msg!("Require {} lamports for {} size data", lamports, data_size);
    let ix = solana_program::system_instruction::create_account(
        funding_account.key,
        pda.key,
        lamports,
        space,
        program_id,
    );

    // Sign and submit transaction
    invoke_signed(
        &ix,
        &[funding_account.clone(), pda.clone()],
        &[&[b"launch_account", &[bump_seed]]]
    )?;

    Ok(())
}

pub fn transfer_tokens<'a>(
    amount : u64,
    token_source_account : &AccountInfo<'a>,
    token_dest_account : &AccountInfo<'a>,
    authority_account : &AccountInfo<'a>,
    token_program_account : &AccountInfo<'a>,
    bump_seed : u8

) -> ProgramResult
{
    let ix = spl_token::instruction::transfer(
        token_program_account.key,
        token_source_account.key,
        token_dest_account.key,
        authority_account.key,
        &[],
        amount,
    )?;

    invoke_signed(
        &ix,
        &[token_source_account.clone(), token_dest_account.clone(), authority_account.clone(), token_program_account.clone()],
        &[&[b"launch_account", &[bump_seed]]]
    )?;

    Ok(())
}

pub fn create_token_account<'a>(
    funding_account : &AccountInfo<'a>,
    wallet_account : &AccountInfo<'a>,
    token_mint_account : &AccountInfo<'a>,
    new_token_account : &AccountInfo<'a>,
    token_program_account : &AccountInfo<'a>

) -> ProgramResult
{
    if **new_token_account.try_borrow_lamports()? > 0 {
        msg!("Token account is already initialised.");
        return Ok(());

    }

    msg!("creating Token account");
    let create_ata_idx = create_associated_token_account(&funding_account.key, &wallet_account.key,&token_mint_account.key);

    invoke(
        &create_ata_idx,
        &[funding_account.clone(), new_token_account.clone(), wallet_account.clone(), token_mint_account.clone(), token_program_account.clone()],
    )?;

    Ok(())
}


pub fn close_program_token_account<'a>(
    program_account_info : &AccountInfo<'a>,
    program_token_account_info : &AccountInfo<'a>,
    destination_account_info : &AccountInfo<'a>,
    destination_token_account_info : &AccountInfo<'a>,
    token_program_account_info : &AccountInfo<'a>,
    bump_seed : u8
) -> ProgramResult
{
    // Check the destination token account exists, which it should do if we are the ones that set it up
    if **destination_token_account_info.try_borrow_lamports()? > 0 {
        msg!("Confirmed destination token account is already initialised.");
    }
    else {

        msg!("destination token account should already exist");
        return Err(ProgramError::InvalidAccountData);
    }

    // And check that we haven't already closed out the program token account
    let program_token_account_lamports = **program_token_account_info.try_borrow_lamports()?;
    if program_token_account_lamports > 0 {
        msg!("Confirmed program token account is still initialised.");
    }
    else {

        msg!("program's token account already closed");
        return Ok(());
    }

    let program_token_account = spl_token::state::Account::unpack_unchecked(&program_token_account_info.try_borrow_data()?)?;

    msg!("transfer token balance: {}", program_token_account.amount);

    if program_token_account.amount > 0 {
        transfer_tokens(
            program_token_account.amount,
            program_token_account_info,
            destination_token_account_info,
            program_account_info,
            token_program_account_info,
            bump_seed
        )?;
    }

    msg!("close account and transfer SOL balance: {}", to_sol(program_token_account_lamports));

    let close_token_account_idx = spl_token::instruction::close_account(
        token_program_account_info.key,
        program_token_account_info.key, 
        destination_account_info.key, 
        program_account_info.key, 
        &[]
    )?;

    invoke_signed(
        &close_token_account_idx,
        &[program_token_account_info.clone(), destination_account_info.clone(), program_account_info.clone()],
        &[&[b"launch_account", &[bump_seed]]]
    )?;

    Ok(())
}


pub fn to_sol(value : u64) -> f64 {
    (value as f64) / (LAMPORTS_PER_SOL as f64)
}