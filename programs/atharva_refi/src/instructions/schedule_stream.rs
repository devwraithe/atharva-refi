use anchor_lang::prelude::*;

use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
};
use anchor_spl::token::{Mint, Token, TokenAccount};
use ephemeral_rollups_sdk::consts::MAGIC_PROGRAM_ID;
use magicblock_magic_program_api::{args::ScheduleTaskArgs, instruction::MagicBlockInstruction};

use crate::constants::{
    MARINADE_PROGRAM_ID, ORG_VAULT_SEED, POOL_SEED, POOL_VAULT_SEED, STREAM_INTERVAL_MS,
};
use crate::errors::ErrorCode;
use crate::states::{Pool, ScheduleStreamArgs};

/// Schedules automated yield streaming via MagicBlock Cranks
/// Crank calls Stream instruction every 2 days to distribute yields
///
/// The crank will periodically call `stream` instruction to:
/// - Calculate accumulated yield from Marinade staking
/// - Distribute organization's percentage to their vault
/// - Keep remainder in pool for supporters

#[derive(Accounts)]
pub struct ScheduleStream<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [
            POOL_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id
        ],
        bump = pool.pool_bump,
        constraint = pool.is_active @ ErrorCode::PoolNotActive,
        constraint = !pool.is_crank_scheduled @ ErrorCode::CrankAlreadyScheduled,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id,
        ],
        bump = pool.pool_vault_bump,
    )]
    pub pool_vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [
            ORG_VAULT_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id
        ],
        bump = pool.org_vault_bump,
    )]
    pub organization_vault: SystemAccount<'info>,

    /// CHECK: Validate by Marinade program
    #[account(mut)]
    pub marinade_state: AccountInfo<'info>,

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,

    /// CHECK: Validated by Marinade program
    #[account(mut)]
    pub liq_pool_sol_leg: AccountInfo<'info>,

    /// CHECK: Validated by Marinade program
    #[account(mut)]
    pub liq_pool_msol_leg: Account<'info, TokenAccount>,

    /// CHECK: Validated by Marinade program. Holds mSOl backing SOL
    #[account(mut)]
    pub treasury_msol_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Only Marinade program ID can be passed
    #[account(address = MARINADE_PROGRAM_ID)]
    pub marinade_program: AccountInfo<'info>,

    /// CHECK: used for MagicBlock program CPI
    #[account(address = MAGIC_PROGRAM_ID)]
    pub magic_program: AccountInfo<'info>,

    /// CHECK: used for Atharva ReFi CPI
    pub program: AccountInfo<'info>,
}
impl<'info> ScheduleStream<'info> {
    pub fn process(&mut self, args: ScheduleStreamArgs) -> Result<()> {
        // Validation
        require!(
            args.execution_interval_millis >= STREAM_INTERVAL_MS,
            ErrorCode::IntervalTooShort
        );
        require!(args.iterations > 0, ErrorCode::InvalidIterations);

        // Update pool state
        self.pool.is_crank_scheduled = true;
        self.pool.last_stream_ts = Clock::get()?.unix_timestamp as u64;

        // Build stream instruction for cranking
        let stream_ix = self.build_stream_ix()?;

        // Serialize MagicBlock Task Args
        let ix_data = bincode::serialize(&MagicBlockInstruction::ScheduleTask(ScheduleTaskArgs {
            task_id: args.task_id,
            execution_interval_millis: args.execution_interval_millis,
            iterations: args.iterations,
            instructions: vec![stream_ix],
        }))
        .map_err(|_| ErrorCode::SerializationError)?;

        // Create the schedule instruction for CPI
        let schedule_ix = Instruction::new_with_bytes(
            self.magic_program.key(),
            &ix_data,
            vec![
                AccountMeta::new(self.pool_vault.key(), true),
                AccountMeta::new(self.pool.key(), false),
            ],
        );

        let seeds = &[
            POOL_VAULT_SEED.as_bytes(),
            self.pool.organization_pubkey.as_ref(),
            &self.pool.new_species_id,
            &[self.pool.pool_vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        invoke_signed(
            &schedule_ix,
            &[
                self.pool_vault.to_account_info(),
                self.pool.to_account_info(),
            ],
            signer_seeds,
        )?;

        msg!(
            "Crank scheduled: task_id={}, interval={}ms, iterations={}",
            args.task_id,
            args.execution_interval_millis,
            args.iterations
        );

        Ok(())
    }

    fn build_stream_ix(&self) -> Result<Instruction> {
        let pool_msol_account = anchor_spl::associated_token::get_associated_token_address(
            &self.pool_vault.key(),
            &self.msol_mint.key(),
        );

        Ok(Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(self.pool.key(), false),
                AccountMeta::new(self.pool_vault.key(), true),
                AccountMeta::new(pool_msol_account, false),
                AccountMeta::new(self.organization_vault.key(), false),
                AccountMeta::new(self.marinade_state.key(), false),
                AccountMeta::new(self.msol_mint.key(), false),
                AccountMeta::new(self.liq_pool_sol_leg.key(), false),
                AccountMeta::new(self.liq_pool_msol_leg.key(), false),
                AccountMeta::new(self.treasury_msol_account.key(), false),
                AccountMeta::new_readonly(self.marinade_program.key(), false),
                AccountMeta::new_readonly(self.token_program.key(), false),
                AccountMeta::new_readonly(self.system_program.key(), false),
            ],
            data: anchor_lang::InstructionData::data(&crate::instruction::Stream {}),
        })
    }
}
