use crate::constants::MARINADE_FINANCE;
use crate::utilities::calculate_ix_discriminator;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    system_program,
};

pub struct MarinadeLiquidUnstakeAccounts<'info> {
    pub marinade_state: AccountInfo<'info>,
    pub msol_mint: AccountInfo<'info>,
    pub liq_pool_sol_leg: AccountInfo<'info>,
    pub liq_pool_msol_leg: AccountInfo<'info>,
    pub treasury_msol_account: AccountInfo<'info>,
    pub burn_msol_from: AccountInfo<'info>,
    pub burn_msol_authority: AccountInfo<'info>,
    pub sol_receiver: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

pub fn marinade_liquid_unstake<'info>(
    msol_amount: u64,
    accounts: MarinadeLiquidUnstakeAccounts<'info>,
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
        AccountMeta::new(accounts.burn_msol_from.key(), false),
        AccountMeta::new_readonly(accounts.burn_msol_authority.key(), true),
        AccountMeta::new(accounts.sol_receiver.key(), false),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new_readonly(anchor_spl::token::ID, false),
    ];

    let ix = Instruction {
        program_id: MARINADE_FINANCE,
        accounts: metas,
        data,
    };

    let infos = &[
        accounts.marinade_state,
        accounts.msol_mint,
        accounts.liq_pool_sol_leg,
        accounts.liq_pool_msol_leg,
        accounts.treasury_msol_account,
        accounts.burn_msol_from,
        accounts.burn_msol_authority,
        accounts.sol_receiver,
        accounts.system_program,
        accounts.token_program,
    ];

    match signer_seeds {
        Some(seeds) => invoke_signed(&ix, infos, seeds)?,
        None => invoke(&ix, infos)?,
    }

    Ok(())
}
