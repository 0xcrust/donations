use solana_program::program_error::ProgramError;
use std::convert::TryInto;

use crate::error::CampaignError::{
    InvalidInstruction,
};

use arrayref::{ array_ref, array_refs };

pub enum Instruction {
    /// Starts a new fundraiser campaign and initializes a state account(a PDA),
    /// and a vault for storing tokens.
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the fundstarter
    /// 1. `[writable]` The campaign state account. A pda with seeds [&[b"state"], fundstarter.key.as_ref()]
    /// 2. `[]` The campaign vault account to hold all the donations. A pda with seeds [&[b"vault"], fundstarter.key.as_ref]
    InitCampaign {
        /// The campaign target
        target: u64,
        /// The campaign description. A string of not more than 200 bytes
        description: [u8; 200],
    },
    /// Donates to a fundraiser campaign
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer, writable]` The donator's account
    /// 1. `[writable]` The state account of the campaign they're donating to
    /// 2. `[writable]` The vault account they'll be sending their donations to.
    Donate {
        /// The donation size
        amount: u64
    },
    /// Withdraws from a campaign when the target is met
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer, writable]` The fundstarter's main account
    /// 1. `[writable]` The campaign state account
    /// 2. `[writable]` The vault account
    Withdraw {},
}


impl Instruction {
    /// Unpacks a byte buffer into a [Instruction]
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::unpack_init_campaign_args(rest)?,
            1 => Self::unpack_donate_args(rest)?,
            2 => Self::Withdraw {},
            _ => return Err(InvalidInstruction.into()),            
        })
    }

    pub fn unpack_init_campaign_args(src: &[u8]) -> Result<Instruction, ProgramError> {
        let src = array_ref![src, 0, 208];
        let (target, description) = array_refs![src, 8, 200];
        Ok(Instruction::InitCampaign {
            target: u64::from_le_bytes(*target),
            description: *description,
        })
    }

    pub fn unpack_donate_args(src: &[u8]) -> Result<Instruction, ProgramError> {
        let amount = src
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(Instruction::Donate { amount })
    }
}


