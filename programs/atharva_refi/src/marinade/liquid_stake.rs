use crate::constants::MARINADE_PROGRAM_ID;
use crate::utilities::calculate_ix_discriminator;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
};

/// Accounts needed for liquid staking CPI
pub struct LiquidStakeAccounts<'info> {
    pub marinade_state: AccountInfo<'info>,
    pub msol_mint: AccountInfo<'info>,
    pub liq_pool_sol_leg: AccountInfo<'info>,
    pub liq_pool_msol_leg: AccountInfo<'info>,
    pub liq_pool_msol_leg_authority: AccountInfo<'info>,
    pub reserve_pda: AccountInfo<'info>,
    pub transfer_from: AccountInfo<'info>,
    pub mint_to: AccountInfo<'info>,
    pub msol_mint_authority: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

/// Perform a liquid staking deposit into Marinade
pub fn marinade_liquid_stake<'info>(
    amount: u64, // lamports
    accounts: LiquidStakeAccounts<'info>,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    require!(amount > 0, crate::errors::ErrorCode::AmountTooSmall);

    // Marinade instruction discriminator for 'deposit'
    let mut data = calculate_ix_discriminator("deposit");
    data.extend_from_slice(&amount.to_le_bytes());

    // Map accounts in the order Marinade expects
    let metas = vec![
        AccountMeta::new(accounts.marinade_state.key(), false),
        AccountMeta::new(accounts.msol_mint.key(), false),
        AccountMeta::new(accounts.liq_pool_sol_leg.key(), false),
        AccountMeta::new(accounts.liq_pool_msol_leg.key(), false),
        AccountMeta::new_readonly(accounts.liq_pool_msol_leg_authority.key(), false),
        AccountMeta::new(accounts.reserve_pda.key(), false),
        AccountMeta::new(accounts.transfer_from.key(), true),
        AccountMeta::new(accounts.mint_to.key(), false),
        AccountMeta::new_readonly(accounts.msol_mint_authority.key(), false),
        AccountMeta::new_readonly(accounts.system_program.key(), false),
        AccountMeta::new_readonly(accounts.token_program.key(), false),
    ];

    let ix = Instruction {
        program_id: MARINADE_PROGRAM_ID,
        accounts: metas,
        data,
    };

    let infos = &[
        accounts.marinade_state,
        accounts.msol_mint,
        accounts.liq_pool_sol_leg,
        accounts.liq_pool_msol_leg,
        accounts.liq_pool_msol_leg_authority,
        accounts.reserve_pda,
        accounts.transfer_from,
        accounts.mint_to,
        accounts.msol_mint_authority,
        accounts.system_program,
        accounts.token_program,
    ];

    match signer_seeds {
        Some(seeds) => invoke_signed(&ix, infos, seeds)?,
        None => invoke(&ix, infos)?,
    }

    Ok(())
}
