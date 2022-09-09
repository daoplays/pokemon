use solana_program::program_error::ProgramError;
use borsh::{BorshDeserialize, BorshSerialize};
use crate::state::Charity;
use crate::error::DaoPlaysError::InvalidInstruction;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum Button {
    A,
    B,
    Up,
    Down,
    Left,
    Right,
    Start,
    Select,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct ButtonData {
    pub button: Button,
    pub amount: u64
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct BidData {
    // the amount in lamports that will be donated to charity
    pub amount_charity : u64,
    // the amount in lamports being paid to the developers
    pub amount_dao : u64,
    // the chosen charity
    pub charity : Charity
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct InitData {
    // the amount of DPTTs to be sent to the program
    pub amount : u64
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum DPPInstruction {

    CreateDataAccount {
        init_data : InitData
    },

    PushButton {
        button_data: ButtonData
    },

    PlaceBid {
        bid_data: BidData
    },

    SelectWinners,

    SendTokens
}

impl DPPInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::CreateDataAccount  {
                init_data: InitData::try_from_slice(&rest)?,
            },
            1 => Self::PlaceBid{
                bid_data: BidData::try_from_slice(&rest)?,
            },
            2 => Self::SelectWinners,
            3 => Self::SendTokens,
            _ => return Err(InvalidInstruction.into()),
        })
    }
}