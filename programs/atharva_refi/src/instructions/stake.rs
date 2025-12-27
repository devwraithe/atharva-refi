use crate::constants::MARINADE_FINANCE;
use crate::errors::ErrorCode;
use crate::utilities::calculate_ix_discriminator;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke,
    system_program,
};
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_msol_account: Account<'info, TokenAccount>,

    /// CHECK: Marinade state account
    #[account(mut)]
    pub marinade_state: AccountInfo<'info>,

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,

    /// CHECK: Reserve PDA
    #[account(mut)]
    pub reserve_pda: AccountInfo<'info>,

    /// CHECK: Liquidity pool SOL leg
    #[account(mut)]
    pub liq_pool_sol_leg: AccountInfo<'info>,

    /// CHECK: Liquidity pool mSOL leg
    #[account(mut)]
    pub liq_pool_msol_leg: AccountInfo<'info>,

    /// CHECK: Liquidity pool mSOL leg authority
    pub liq_pool_msol_leg_authority: AccountInfo<'info>,

    /// CHECK: mSOL mint authority
    pub msol_mint_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
impl<'info> Stake<'info> {
    /// Stake SOL with Marinade to receive mSOL
    /// Manually constructs the Marinade deposit instruction
    pub fn handler(&self, lamports: u64) -> Result<()> {
        msg!("Preparing Marinade Finance CPI call...");

        require!(lamports >= 1_000_000, ErrorCode::AmountTooSmall);

        // Marinade "Deposit" instruction discriminator
        let mut instruction_data = calculate_ix_discriminator("deposit");
        instruction_data.extend_from_slice(&lamports.to_le_bytes());

        // Build account metas for Marinade deposit instruction
        let accounts = vec![
            AccountMeta::new(self.marinade_state.key(), false),
            AccountMeta::new(self.msol_mint.key(), false),
            AccountMeta::new(self.liq_pool_sol_leg.key(), false),
            AccountMeta::new(self.liq_pool_msol_leg.key(), false),
            AccountMeta::new_readonly(self.liq_pool_msol_leg_authority.key(), false),
            AccountMeta::new(self.reserve_pda.key(), false),
            AccountMeta::new(self.user.key(), true),
            AccountMeta::new(self.user_msol_account.key(), false),
            AccountMeta::new_readonly(self.msol_mint_authority.key(), false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(anchor_spl::token::ID, false),
        ];

        // Create the instruction
        let deposit_ix = Instruction {
            program_id: MARINADE_FINANCE,
            accounts,
            data: instruction_data,
        };

        // Invoke CPI
        invoke(
            &deposit_ix,
            &[
                self.marinade_state.to_account_info(),
                self.msol_mint.to_account_info(),
                self.liq_pool_sol_leg.to_account_info(),
                self.liq_pool_msol_leg.to_account_info(),
                self.liq_pool_msol_leg_authority.to_account_info(),
                self.reserve_pda.to_account_info(),
                self.user.to_account_info(),
                self.user_msol_account.to_account_info(),
                self.msol_mint_authority.to_account_info(),
                self.system_program.to_account_info(),
                self.token_program.to_account_info(),
            ],
        )?;

        msg!("Successfully staked {} lamports for mSOL", lamports);

        Ok(())
    }
}
