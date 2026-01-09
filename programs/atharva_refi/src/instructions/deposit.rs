use crate::constants::{POOL_MINT_SEED, POOL_SEED, POOL_VAULT_SEED};
use crate::errors::ErrorCode;
use crate::events::SupporterDeposited;
use crate::states::Pool;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};

/// Deposits to pool vault and mints reciept tokens to supporter

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub supporter: Signer<'info>,

    #[account(
        mut,
        seeds = [
            POOL_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id
        ],
        bump = pool.pool_bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        mint::authority = pool,
        mint::token_program = token_program,
        seeds = [
            POOL_MINT_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id
        ],
        bump,
    )]
    pub pool_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id
        ],
        bump,
    )]
    pub pool_vault: SystemAccount<'info>,

    #[account(
        init_if_needed,
        payer = supporter,
        associated_token::mint = pool_mint,
        associated_token::authority = supporter,
        associated_token::token_program = token_program,
    )]
    pub supporter_pool_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}
impl<'info> Deposit<'info> {
    pub fn process(&mut self, amount: u64) -> Result<()> {
        // Validation
        require!(amount > 0, ErrorCode::InvalidAmount);
        require!(self.pool.is_active, ErrorCode::PoolNotActive);

        let pool = &mut self.pool;

        // Update state
        pool.total_deposits = pool
            .total_deposits
            .checked_add(amount)
            .ok_or(ErrorCode::MathError)?;
        pool.total_shares = pool
            .total_shares
            .checked_add(amount)
            .ok_or(ErrorCode::MathError)?;

        // Transfer from supporter to pool vault
        let transfer_cpi_accounts = Transfer {
            from: self.supporter.to_account_info(),
            to: self.pool_vault.to_account_info(),
        };

        let transfer_cpi_ctx =
            CpiContext::new(self.system_program.to_account_info(), transfer_cpi_accounts);

        system_program::transfer(transfer_cpi_ctx, amount)?;

        msg!("Deposited {} lamports to pool vault", amount);

        // Mint receipt tokens for supporter
        let seeds = &[
            POOL_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id,
            &[pool.pool_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let mint_cpi_accounts = MintTo {
            mint: self.pool_mint.to_account_info(),
            to: self.supporter_pool_token_account.to_account_info(),
            authority: pool.to_account_info(),
        };

        let mint_cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            mint_cpi_accounts,
            signer_seeds,
        );

        mint_to(mint_cpi_ctx, amount)?;

        msg!("Minted {} receipt tokens to supporter", amount);

        // Emit event
        emit!(SupporterDeposited {
            organization_pubkey: self.pool.organization_pubkey,
            species_name: self.pool.species_name.clone(),
            amount,
        });

        msg!("Supporter Balance: {}", self.supporter.lamports());
        msg!("Pool Vault Balance: {}", self.pool_vault.lamports());

        self.supporter_pool_token_account.reload()?;

        msg!(
            "Supporter Token Account Balance: {}",
            self.supporter_pool_token_account.amount
        );

        Ok(())
    }
}
