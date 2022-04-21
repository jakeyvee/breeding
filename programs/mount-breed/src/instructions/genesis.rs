use anchor_lang::prelude::*;

use crate::state::*;
use spl_token::instruction::AuthorityType;
use anchor_spl::token::{self, Mint, SetAuthority, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Genesis<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        seeds = [b"token-seed".as_ref()],
        bump,
        payer = initializer,
        token::mint = mint,
        token::authority = initializer,
    )]
    pub vault_account: Account<'info, TokenAccount>,
    #[account(init, 
        payer = initializer, 
        space = 8 + 1 + 32 * 5, 
        seeds = [b"escrow-seed".as_ref()], 
        bump)]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    #[account(
        mut,
        constraint = initializer_deposit_token_account.amount >= 2202
    )]
    pub initializer_deposit_token_account: Account<'info, TokenAccount>,
    pub mint_mtm: Account<'info, Mint>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub creator_a: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub creator_b: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

impl<'info> Genesis<'info> {
    fn into_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self
                .initializer_deposit_token_account
                .to_account_info()
                .clone(),
            to: self.vault_account.to_account_info().clone(),
            authority: self.initializer.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.vault_account.to_account_info().clone(),
            current_authority: self.initializer.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

pub fn handler(
    ctx: Context<Genesis>,
    _vault_account_bump: u8,
    escrow_account_bump: u8,
) -> ProgramResult {
    ctx.accounts.escrow_account.initializer_key = *ctx
        .accounts
        .initializer
        .to_account_info()
        .key;
    ctx.accounts.escrow_account.initializer_deposit_token_account = *ctx
        .accounts
        .initializer_deposit_token_account
        .to_account_info()
        .key;
    ctx.accounts.escrow_account.mtm_token_mint = *ctx
        .accounts
        .mint_mtm
        .to_account_info()
        .key;
    ctx.accounts.escrow_account.whitelist_creator_a = *ctx
        .accounts
        .creator_a
        .key;
    ctx.accounts.escrow_account.whitelist_creator_b = *ctx
        .accounts
        .creator_b
        .key;
    ctx.accounts.escrow_account.bump = escrow_account_bump;

    let (vault_authority, _vault_authority_bump) = 
        Pubkey::find_program_address(&[ESCROW_PDA_SEED], ctx.program_id);
    token::set_authority(
        ctx.accounts.into_set_authority_context(),
        AuthorityType::AccountOwner,
        Some(vault_authority),
    )?;

    token::transfer(
        ctx.accounts.into_transfer_to_pda_context(),
        2202,
    )?;

    Ok(())
}

