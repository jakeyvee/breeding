use anchor_lang::prelude::*;

use crate::state::*;
use anchor_spl::token::{Mint, TokenAccount};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init, payer = user, space = 8 + 10, seeds = [mount_mint_account.key().as_ref()], bump)]
    pub mount_data_account: Account<'info, Data>,
    #[account(
        constraint = mount_token_account.mint == mount_mint_account.key()
    )]
    pub mount_mint_account: Account<'info, Mint>,
    #[account(
        constraint = mount_token_account.amount == 1,
        constraint = mount_token_account.owner == *user.to_account_info().key
    )]
    pub mount_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>
}

pub fn handler(ctx: Context<Initialize>, data_bump: u8) -> ProgramResult {
    ctx.accounts.mount_data_account.count = 0;
    ctx.accounts.mount_data_account.bump = data_bump;
    ctx.accounts.mount_data_account.timestamp = 0;
    Ok(())
}