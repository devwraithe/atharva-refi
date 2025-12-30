use crate::actions::{marinade_liquid_unstake, MarinadeLiquidUnstakeAccounts};
use crate::constants::MARINADE_FINANCE;
use crate::errors::ErrorCode;
use crate::utilities::calculate_ix_discriminator;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_program;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct LiquidUnstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_msol_account: Account<'info, TokenAccount>,

    /// CHECK: Marinade state
    #[account(mut)]
    pub marinade_state: AccountInfo<'info>,

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,

    /// CHECK: Liquidity pool SOL leg
    #[account(mut)]
    pub liq_pool_sol_leg: AccountInfo<'info>,

    /// CHECK: Liquidity pool mSOL leg
    #[account(mut)]
    pub liq_pool_msol_leg: AccountInfo<'info>,

    /// CHECK: Treasury mSOL account
    #[account(mut)]
    pub treasury_msol_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
impl<'info> LiquidUnstake<'info> {
    /// Unstake mSOL to receive SOL
    /// Manually constructs the Marinade liquid_unstake instruction
    pub fn process(&self, msol_amount: u64) -> Result<()> {
        msg!("Preparing Marinade Finance CPI call...");

        require!(msol_amount > 0, ErrorCode::AmountTooSmall);

        marinade_liquid_unstake(
            msol_amount,
            MarinadeLiquidUnstakeAccounts {
                marinade_state: self.marinade_state.to_account_info(),
                msol_mint: self.msol_mint.to_account_info(),
                liq_pool_sol_leg: self.liq_pool_sol_leg.to_account_info(),
                liq_pool_msol_leg: self.liq_pool_msol_leg.to_account_info(),
                treasury_msol_account: self.treasury_msol_account.to_account_info(),
                burn_msol_from: self.user_msol_account.to_account_info(),
                burn_msol_authority: self.user.to_account_info(),
                sol_receiver: self.user.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
            None, // user signs directly
        );

        msg!("Successfully unstaked {} mSOL for SOL", msol_amount);

        Ok(())
    }
}
