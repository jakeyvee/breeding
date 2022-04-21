use std::str::FromStr;

use anchor_lang::prelude::*;

use crate::state::*;

use anchor_spl::token::{self, Mint, TokenAccount, Transfer, Burn};
use metaplex_token_metadata::state::Metadata;


fn assert_valid_metadata(
  gem_metadata: &AccountInfo,
  gem_mint: &Pubkey,
) -> Result<Metadata, ProgramError> {
  let metadata_program = Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap();

  // 1 verify the owner of the account is metaplex's metadata program
  assert_eq!(gem_metadata.owner, &metadata_program);

  // 2 verify the PDA seeds match
  let seed = &[
      b"metadata".as_ref(),
      metadata_program.as_ref(),
      gem_mint.as_ref(),
  ];

  let (metadata_addr, _bump) = Pubkey::find_program_address(seed, &metadata_program);
  assert_eq!(metadata_addr, gem_metadata.key());

  Metadata::from_account_info(gem_metadata)
}


fn assert_creators <'info>(
  metadata_mount: &Metadata,
  escrow_account: &Account<'info, EscrowAccount>,
) -> ProgramResult {
  for creator in metadata_mount.data.creators.as_ref().unwrap() {
      // verify creator actually signed off on this nft
      if !creator.verified {
          continue;
      }

      if (creator.address == escrow_account.whitelist_creator_a) 
          || (creator.address == escrow_account.whitelist_creator_b){
              return Ok(())
      }
  }
  Err(ProgramError::Custom(1))
}

fn assert_whitelisted(ctx: &Context<Redeem>) -> ProgramResult {
  let mint_mount_a = &*ctx.accounts.user_mount_mint_account_a;
  let mint_mount_b = &*ctx.accounts.user_mount_mint_account_b;

  let escrow_account = &*ctx.accounts.escrow_account;
  let metadata_info_mount_a = &ctx.accounts.metadata_mount_a;
  let metadata_info_mount_b = &ctx.accounts.metadata_mount_b;

  // verify metadata is legit
  let metadata_mount_a = assert_valid_metadata(metadata_info_mount_a, &mint_mount_a.key())?;
  let metadata_mount_b = assert_valid_metadata(metadata_info_mount_b, &mint_mount_b.key())?;

  if let Err(e) = assert_creators(&metadata_mount_a, escrow_account){
      return Err(e);
  }

  if let Err(e) = assert_creators(&metadata_mount_b, escrow_account){
      return Err(e);
  }

  // if both conditions pass
  Ok(())
}

#[derive(Accounts)]
pub struct Redeem<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, 
        constraint = mount_data_account_a.count < 5,
        constraint = Clock::get()?.unix_timestamp - mount_data_account_a.timestamp > 60,
        seeds = [user_mount_mint_account_a.key().as_ref()], 
        bump = mount_data_account_a.bump
    )]
    pub mount_data_account_a: Box<Account<'info, Data>>,
    #[account(mut, 
        constraint = mount_data_account_b.count < 5,
        constraint = Clock::get()?.unix_timestamp - mount_data_account_b.timestamp > 60,
        seeds = [user_mount_mint_account_b.key().as_ref()], 
        bump = mount_data_account_b.bump
    )]
    pub mount_data_account_b: Box<Account<'info, Data>>,
    #[account(
        constraint = *user_mount_mint_account_a.to_account_info().key 
        == user_mount_token_account_a.mint,
    )]
    pub user_mount_mint_account_a: Box<Account<'info, Mint>>,
    #[account(
        constraint = user_mount_token_account_a.key() != user_mount_token_account_b.key(),
        constraint = user_mount_token_account_a.amount == 1,
        constraint = user_mount_token_account_a.owner == *user.to_account_info().key
    )]
    pub user_mount_token_account_a: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub metadata_mount_a: AccountInfo<'info>,
    #[account(
        constraint = *user_mount_mint_account_b.to_account_info().key 
        == user_mount_token_account_b.mint,
    )]
    pub user_mount_mint_account_b: Box<Account<'info, Mint>>,
    #[account(
        constraint = user_mount_token_account_b.amount == 1,
        constraint = user_mount_token_account_b.owner == *user.to_account_info().key
    )]
    pub user_mount_token_account_b: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub metadata_mount_b: AccountInfo<'info>,
    #[account(mut,
        constraint = user_custom_token_account.owner == *user.to_account_info().key
    )]
    pub user_custom_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        constraint = user_mtm_token_account.amount >= 200 * 1000000000
    )]
    pub user_mtm_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub vault_account: Box<Account<'info, TokenAccount>>,
    #[account(
        constraint = user_mtm_token_account.mint == escrow_account.mtm_token_mint,
        seeds = [b"escrow-seed".as_ref()], 
        bump = escrow_account.bump
    )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    #[account(
        mut,
        constraint = *mint_mtm.to_account_info().key == escrow_account.mtm_token_mint,
    )]
    pub mint_mtm: Account<'info, Mint>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

impl<'info> Redeem<'info> {
    fn into_transfer_to_user_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault_account.to_account_info().clone(),
            to: self
                .user_custom_token_account
                .to_account_info()
                .clone(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_burn_from_user_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        let cpi_accounts = Burn {
            mint: self.mint_mtm.to_account_info().clone(),
            to: self
                .user_mtm_token_account
                .to_account_info()
                .clone(),
            authority: self.user.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

pub fn handler(ctx: Context<Redeem>) -> ProgramResult {
    assert_whitelisted(&ctx)?;
    let (_vault_authority, vault_authority_bump) =
        Pubkey::find_program_address(&[ESCROW_PDA_SEED], ctx.program_id);
    let authority_seeds = &[&ESCROW_PDA_SEED[..], &[vault_authority_bump]];
    token::transfer(
        ctx.accounts
            .into_transfer_to_user_context()
            .with_signer(&[&authority_seeds[..]]),
        1,
    )?;

    token::burn(
        ctx.accounts.into_burn_from_user_context(),
        200 * 1000000000
    )?;

    let clock: Clock = Clock::get().unwrap();
    ctx.accounts.mount_data_account_a.count += 1;
    ctx.accounts.mount_data_account_a.timestamp = clock.unix_timestamp;
    ctx.accounts.mount_data_account_b.count += 1;
    ctx.accounts.mount_data_account_b.timestamp = clock.unix_timestamp;

    Ok(())
}