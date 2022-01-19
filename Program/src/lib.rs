pub mod entrypoint;
pub mod processor;
pub mod instruction;
pub mod state;
pub mod error;

pub const PRICE_SEED: &str = "price";
pub const SETTINGS_SEED: &str = "settings";

solana_program::declare_id!("uMv1Gq62jXMHEZFy2YyqM7jP5hYV8QDfMP4kXJFNxZG");
