use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Clone, Debug, Error, Copy)]
pub enum TokenError {
    #[error("insufficient funds")]
    InsufficientFunds,
    #[error("token mismatch")]
    TokenMismatch,
    #[error("not a delegate")]
    NotDelegate,
    #[error("no owner")]
    NoOwner,
}


impl From<TokenError> for ProgramError {
    fn from(e: TokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}


#[derive(Error, Debug, Copy, Clone)]
pub enum PriceError {
    #[error("Admin signature is required")]
    AdminRequired,

    #[error("Wrong price PDA for this user")]
    WrongCounterPDA,

    #[error("Wrong settings PDA")]
    WrongSettingsPDA,

    #[error("Invalud Instruction")]
    InvalidInstruction
}

impl From<PriceError> for ProgramError {
    fn from(e: PriceError) -> Self {
        ProgramError::Custom(e as u32)
    }
}