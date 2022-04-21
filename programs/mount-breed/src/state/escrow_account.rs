use anchor_lang::prelude::*;

pub const ESCROW_PDA_SEED: &[u8] = b"authority-seed";

#[account]
pub struct EscrowAccount {
    pub initializer_key: Pubkey,
    pub initializer_deposit_token_account: Pubkey,
    pub mtm_token_mint: Pubkey,
    pub whitelist_creator_a: Pubkey,
    pub whitelist_creator_b: Pubkey,
    pub bump: u8,
}

impl EscrowAccount {
}