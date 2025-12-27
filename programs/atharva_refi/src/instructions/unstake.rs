use crate::constants::MARINADE_FINANCE;
use crate::errors::ErrorCode;
use crate::utilities::calculate_ix_discriminator;
use anchor_lang::prelude::program::invoke;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    system_program,
};
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

        // Marinade "LiquidUnstake" instruction discriminator
        let mut instruction_data = calculate_ix_discriminator("liquid_unstake");
        instruction_data.extend_from_slice(&msol_amount.to_le_bytes());

        // Build account metas for Marinade's liquid_unstake instruction
        let accounts = vec![
            AccountMeta::new(self.marinade_state.key(), false),
            AccountMeta::new(self.msol_mint.key(), false),
            AccountMeta::new(self.liq_pool_sol_leg.key(), false),
            AccountMeta::new(self.liq_pool_msol_leg.key(), false),
            AccountMeta::new(self.treasury_msol_account.key(), false),
            AccountMeta::new(self.user_msol_account.key(), false),
            AccountMeta::new_readonly(self.user.key(), true),
            AccountMeta::new(self.user.key(), false), // receives SOL
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(anchor_spl::token::ID, false),
        ];

        // Create the instruction
        let liquid_unstake_ix = Instruction {
            program_id: MARINADE_FINANCE,
            accounts,
            data: instruction_data,
        };

        // Invoke CPI
        invoke(
            &liquid_unstake_ix,
            &[
                self.marinade_state.to_account_info(),
                self.msol_mint.to_account_info(),
                self.liq_pool_sol_leg.to_account_info(),
                self.liq_pool_msol_leg.to_account_info(),
                self.treasury_msol_account.to_account_info(),
                self.user_msol_account.to_account_info(),
                self.user.to_account_info(),
                self.user.to_account_info(),
                self.system_program.to_account_info(),
                self.token_program.to_account_info(),
            ],
        )?;

        msg!("Successfully unstaked {} mSOL for SOL", msol_amount);

        Ok(())
    }
}
