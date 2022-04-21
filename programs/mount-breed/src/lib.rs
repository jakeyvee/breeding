use anchor_lang::prelude::*;

use instructions::*;

pub mod state;
pub mod instructions;

declare_id!("39JMEP5Ss4uEXJfqiJdL6M2nrwPfJ9W632iR2h1hZUnf");

#[program]
pub mod mount_breed {
    use super::*;

    pub fn genesis(
        ctx: Context<Genesis>,
        _vault_account_bump: u8,
        escrow_account_bump: u8,
    ) -> ProgramResult {
        instructions::genesis::handler(ctx, _vault_account_bump, escrow_account_bump)
    }

    pub fn cancel(ctx: Context<Cancel>) -> ProgramResult {
        instructions::cancel::handler(ctx)
    }

    pub fn initialize(ctx: Context<Initialize>, data_bump: u8) -> ProgramResult {
        instructions::initialize::handler(ctx, data_bump)
    }

    pub fn redeem(ctx: Context<Redeem>) -> ProgramResult {
        instructions::redeem::handler(ctx)
    }
}