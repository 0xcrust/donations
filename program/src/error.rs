use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum CampaignError {
    #[error("Could not deserialize instruction data")]
    InvalidInstruction,
    #[error("Invalid Campaign Account data")]
    InvalidAccountData,
    #[error("Account different from expected")]
    AccountMismatch,
    #[error("Account has been created already")]
    AccountAlreadyInitialized,
    #[error("Signer is the wrong authority")]
    WrongAuthority
}

impl From<CampaignError> for ProgramError {
    fn from(e: CampaignError) -> Self {
        ProgramError::Custom(e as u32)
    }
}