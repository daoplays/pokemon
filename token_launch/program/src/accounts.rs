use spl_associated_token_account::get_associated_token_address;
use solana_program::{pubkey::Pubkey, declare_id};
// functions to calculate expected public keys


mod daoplays {
    use super::*;
    declare_id!("FxVpjJ5AGY6cfCwZQP5v8QBfS4J2NPa62HbGh1Fu2LpD");   
}

mod token_mint {
    use super::*;
    declare_id!("6PRgpKnwT9xgGF7cgS7ZMkPBeQmd5mdS97eg26ir8Kki");   
}


mod supporters_token_mint {
    use super::*;
    declare_id!("7B1yoU3EsbABt1kNXcJLeJRT8jwPy9rZfhrhWzuCA9Fq");   
}

pub fn get_expected_daoplays_key() -> Pubkey
{
    daoplays::ID
}

pub fn get_expected_token_mint_key() -> Pubkey
{
    token_mint::ID
}

pub fn get_expected_supporters_token_mint_key() -> Pubkey
{
    supporters_token_mint::ID
}

pub fn get_expected_daoplays_token_key() -> Pubkey
{
    get_associated_token_address(
        &get_expected_daoplays_key(), 
        &get_expected_token_mint_key()
    )
}


pub fn get_expected_daoplays_supporters_token_key() -> Pubkey
{
    get_associated_token_address(
        &get_expected_daoplays_key(), 
        &get_expected_supporters_token_mint_key()
    )
}

pub fn get_pda_bump() -> u8
{
    254
}

pub fn get_expected_program_address_key(program_id : &Pubkey) -> (Pubkey, u8)
{
    let program_address = Pubkey::create_program_address(&[b"launch_account", &[get_pda_bump()]], &program_id).unwrap();

    (program_address, get_pda_bump())
}

pub fn get_expected_program_token_key(program_id : &Pubkey) -> Pubkey
{
    get_associated_token_address(
        &get_expected_program_address_key(program_id).0, 
        &get_expected_token_mint_key()
    )
}

pub fn get_expected_program_supporters_token_key(program_id : &Pubkey) -> Pubkey
{
    get_associated_token_address(
        &get_expected_program_address_key(program_id).0, 
        &get_expected_supporters_token_mint_key()
    )
}
