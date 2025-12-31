use crate::constants::MARINADE_PROGRAM_ID;
use crate::utilities::calculate_ix_discriminator;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
};

pub struct LiquidUnstakeAccounts<'info> {
    pub marinade_state: AccountInfo<'info>,
    pub msol_mint: AccountInfo<'info>,
    pub liq_pool_sol_leg: AccountInfo<'info>,
    pub liq_pool_msol_leg: AccountInfo<'info>,
    pub treasury_msol_account: AccountInfo<'info>,
    pub get_msol_from: AccountInfo<'info>,
    pub get_msol_from_authority: AccountInfo<'info>,
    pub transfer_sol_to: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

pub fn marinade_liquid_unstake<'info>(
    msol_amount: u64,
    accounts: LiquidUnstakeAccounts<'info>,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    require!(msol_amount > 0, crate::errors::ErrorCode::AmountTooSmall);

    // Marinade discriminator
    let mut data = calculate_ix_discriminator("liquid_unstake");
    data.extend_from_slice(&msol_amount.to_le_bytes());

    let metas = vec![
        AccountMeta::new(accounts.marinade_state.key(), false),
        AccountMeta::new(accounts.msol_mint.key(), false),
        AccountMeta::new(accounts.liq_pool_sol_leg.key(), false),
        AccountMeta::new(accounts.liq_pool_msol_leg.key(), false),
        AccountMeta::new(accounts.treasury_msol_account.key(), false),
        AccountMeta::new(accounts.get_msol_from.key(), false),
        AccountMeta::new_readonly(accounts.get_msol_from_authority.key(), true),
        AccountMeta::new(accounts.transfer_sol_to.key(), false),
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
        accounts.treasury_msol_account,
        accounts.get_msol_from,
        accounts.get_msol_from_authority,
        accounts.transfer_sol_to,
        accounts.system_program,
        accounts.token_program,
    ];

    match signer_seeds {
        Some(seeds) => invoke_signed(&ix, infos, seeds)?,
        None => invoke(&ix, infos)?,
    }

    Ok(())
}
