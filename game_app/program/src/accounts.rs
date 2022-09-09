use spl_associated_token_account::get_associated_token_address;
use solana_program::{pubkey::Pubkey, declare_id};
// functions to calculate expected public keys

mod btc_oracle {
    use super::*;
    declare_id!("GVXRSBjFk6e6J3NbVPXohDJetcTjaeeuykUpbQF8UoMU");   
}
mod eth_oracle {
    use super::*;
    declare_id!("JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB");   
}
mod sol_oracle {
    use super::*;
    declare_id!("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");   
}
mod daoplays {
    use super::*;
    declare_id!("FxVpjJ5AGY6cfCwZQP5v8QBfS4J2NPa62HbGh1Fu2LpD");   
}

mod token_mint {
    use super::*;
    declare_id!("6PRgpKnwT9xgGF7cgS7ZMkPBeQmd5mdS97eg26ir8Kki");   
}

pub fn get_expected_btc_key() -> Pubkey
{
    btc_oracle::ID
}

pub fn get_expected_eth_key() -> Pubkey
{
    eth_oracle::ID
}

pub fn get_expected_sol_key() -> Pubkey
{
    sol_oracle::ID
}

pub fn get_expected_daoplays_key() -> Pubkey
{
    daoplays::ID
}

pub fn get_expected_token_mint_key() -> Pubkey
{
    token_mint::ID
}

pub fn get_expected_daoplays_token_key() -> Pubkey
{
    get_associated_token_address(
        &get_expected_daoplays_key(), 
        &get_expected_token_mint_key()
    )
}

pub fn get_pda_bump() -> u8
{
    255
}

pub fn get_expected_program_address_key(program_id : &Pubkey) -> (Pubkey, u8)
{
    let program_address = Pubkey::create_program_address(&[b"token_account", &[get_pda_bump()]], &program_id).unwrap();

    (program_address, get_pda_bump())
}

pub fn get_expected_data_account_key(program_id : &Pubkey) -> Pubkey
{
    let data_key = Pubkey::create_with_seed(
        &get_expected_daoplays_key(),
        "data_account",
        &program_id,
    ).unwrap();

    return data_key;

}

pub fn get_expected_program_token_key(program_id : &Pubkey) -> Pubkey
{
    get_associated_token_address(
        &get_expected_program_address_key(program_id).0, 
        &get_expected_token_mint_key()
    )
}
