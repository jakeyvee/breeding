use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer, Burn};
use spl_token::instruction::AuthorityType;
use metaplex_token_metadata::state::Metadata;

declare_id!("39JMEP5Ss4uEXJfqiJdL6M2nrwPfJ9W632iR2h1hZUnf");

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

#[program]
pub mod mount_breed {
    use super::*;

    const ESCROW_PDA_SEED: &[u8] = b"authority-seed";

    pub fn genesis(
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

    pub fn cancel(ctx: Context<Cancel>) -> ProgramResult {
        let (_vault_authority, vault_authority_bump) =
            Pubkey::find_program_address(&[ESCROW_PDA_SEED], ctx.program_id);
        let authority_seeds = &[&ESCROW_PDA_SEED[..], &[vault_authority_bump]];

        token::transfer(
            ctx.accounts
                .into_transfer_to_initializer_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.vault_account.amount,
        )?;

        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&authority_seeds[..]]),
        )?;

        Ok(())
    }

    pub fn initialize(ctx: Context<Initialize>, data_bump: u8) -> ProgramResult {
        ctx.accounts.mount_data_account.count = 0;
        ctx.accounts.mount_data_account.bump = data_bump;
        ctx.accounts.mount_data_account.timestamp = 0;
        Ok(())
    }

    pub fn redeem(ctx: Context<Redeem>) -> ProgramResult {
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
}

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

#[derive(Accounts)]
pub struct Cancel<'info> {
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub initializer: Signer<'info>,
    #[account(mut)]
    pub vault_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    #[account(mut)]
    pub initializer_deposit_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = escrow_account.initializer_key == *initializer.to_account_info().key,
        constraint = escrow_account.initializer_deposit_token_account == *initializer_deposit_token_account.to_account_info().key,
        close = initializer
    )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

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

impl<'info> Cancel<'info> {
    fn into_transfer_to_initializer_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault_account.to_account_info().clone(),
            to: self
                .initializer_deposit_token_account
                .to_account_info()
                .clone(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.vault_account.to_account_info().clone(),
            destination: self.initializer.to_account_info().clone(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
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

#[account]
pub struct EscrowAccount {
    pub initializer_key: Pubkey,
    pub initializer_deposit_token_account: Pubkey,
    pub mtm_token_mint: Pubkey,
    pub whitelist_creator_a: Pubkey,
    pub whitelist_creator_b: Pubkey,
    pub bump: u8,
}

#[account]
pub struct Data {
    pub count: u8,
    pub bump: u8,
    pub timestamp: i64
}