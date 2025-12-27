use crate::constants::{POOL_MINT_SEED, POOL_SEED, POOL_VAULT_SEED};
use crate::errors::ErrorCode;
use crate::events::SupporterDeposited;
use crate::states::Pool;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub supporter: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED.as_bytes(), pool.organization_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump = pool.pool_bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [POOL_MINT_SEED.as_bytes(), pool.organization_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump,
    )]
    pub pool_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [POOL_VAULT_SEED.as_bytes(), pool.key().as_ref(), pool.organization_pubkey.as_ref()],
        bump = pool.org_vault_bump,
    )]
    pub pool_vault: SystemAccount<'info>,

    #[account(
        init_if_needed,
        payer = supporter,
        associated_token::mint = pool_mint,
        associated_token::authority = supporter,
        associated_token::token_program = token_program,
    )]
    pub supporter_pool_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn process(&mut self, amount: u64) -> Result<()> {
        // Verify checks
        require!(amount > 0, ErrorCode::InvalidAmount);
        require!(self.pool.is_active, ErrorCode::PoolNotActive);
        require!(
            self.supporter.lamports() >= amount,
            ErrorCode::InsufficientFunds
        );

        // Update states
        self.pool.pool_mint = self.pool_mint.key();
        self.pool.total_deposits = self
            .pool
            .total_deposits
            .checked_add(amount)
            .ok_or(ErrorCode::MathError)?;
        self.pool.total_shares = self
            .pool
            .total_shares
            .checked_add(amount)
            .ok_or(ErrorCode::MathError)?;

        // Transfer
        let transfer_cpi_accounts = Transfer {
            from: self.supporter.to_account_info(),
            to: self.pool_vault.to_account_info(),
        };

        let transfer_cpi_ctx =
            CpiContext::new(self.system_program.to_account_info(), transfer_cpi_accounts);

        system_program::transfer(transfer_cpi_ctx, amount)?;

        // Minting
        let pool_key = self.pool.key();
        let vault_seeds = &[
            POOL_VAULT_SEED.as_bytes(),
            pool_key.as_ref(),
            self.pool.organization_pubkey.as_ref(),
            &[self.pool.org_vault_bump],
        ];

        let signer_seeds = &[&vault_seeds[..]];

        let mint_cpi_accounts = MintTo {
            mint: self.pool_mint.to_account_info(),
            to: self.supporter_pool_token_account.to_account_info(),
            authority: self.pool_vault.to_account_info(),
        };

        let mint_cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            mint_cpi_accounts,
            signer_seeds,
        );

        mint_to(mint_cpi_ctx, amount)?;

        // Emit event
        emit!(SupporterDeposited {
            organization_pubkey: self.pool.organization_pubkey,
            species_name: self.pool.species_name.clone(),
            amount,
        });

        Ok(())
    }
}
