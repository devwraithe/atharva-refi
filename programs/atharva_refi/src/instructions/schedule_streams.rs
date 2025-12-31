use anchor_lang::{prelude::*, InstructionData};

use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
};
use anchor_spl::token::{Mint, Token, TokenAccount};
use ephemeral_rollups_sdk::consts::MAGIC_PROGRAM_ID;
use magicblock_magic_program_api::{args::ScheduleTaskArgs, instruction::MagicBlockInstruction};

use crate::constants::{MARINADE_PROGRAM_ID, ORG_VAULT_SEED, POOL_SEED, POOL_VAULT_SEED, STREAM_INTERVAL_MS};
use crate::errors::ErrorCode;
use crate::states::MarinadeState;
use crate::states::{Pool, ScheduleStreamArgs};

/// Schedules automated yield streaming via MagicBlock Cranks
/// Crank calls Stream instruction every 2 days to distribute yields
///
/// The crank will periodically call stream_yields instruction to:
/// - Calculate accumulated yield from Marinade staking
/// - Distribute org's percentage to organization vault
/// - Keep remainder in pool for supporters

#[derive(Accounts)]
pub struct ScheduleStream<'info> {
    #[account(
        mut, 
        constraint = authority.key() == pool.organization_pubkey,
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED.as_bytes(), pool.organization_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump = pool.pool_bump,
        constraint = pool.is_active @ ErrorCode::PoolNotActive,
        constraint = !pool.is_crank_scheduled @ ErrorCode::CrankAlreadyScheduled,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [POOL_VAULT_SEED.as_bytes(), pool.key().as_ref(), pool.organization_pubkey.as_ref()],
        bump = pool.pool_vault_bump,
    )]
    pub pool_vault: SystemAccount<'info>,

    #[account(
        mut,
        address = pool.organization_pubkey,
        seeds = [ORG_VAULT_SEED.as_bytes(), pool.organization_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump = pool.org_vault_bump,
    )]
    pub organization_vault: SystemAccount<'info>,

    /// Marinade state account
    /// CHECK: Verified by Marinade program
    #[account(mut)]
    pub marinade_state: Account<'info, MarinadeState>,

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,

    /// Vault to receive SOL from unstake
    /// CHECK: Validated by Marinade program
    #[account(mut)]
    pub liq_pool_sol_leg: AccountInfo<'info>,

    /// CHECK: Validated by Marinade program
    #[account(mut)]
    pub liq_pool_msol_leg: Account<'info, TokenAccount>,

    /// Holds mSOL backing SOL
    /// CHECK: Validated by Marinade program
    #[account(mut)]
    pub treasury_msol_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Manual check of the program ID
    #[account(address = MARINADE_PROGRAM_ID)]
    pub marinade_program: AccountInfo<'info>,

    /// CHECK: used for CPI
    #[account()]
    pub magic_program: AccountInfo<'info>,

    /// CHECK: used for CPI
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

        // Update pool state to track scheduling
        self.pool.is_crank_scheduled = true;
        self.pool.last_stream_ts = Clock::get()?.unix_timestamp as u64;

        // Build the stream instruction that the crank will call
        let stream_ix = self.build_stream_yields_instruction()?;

        // Serialize MagicBlock Task Args
        let schedule_args = ScheduleTaskArgs {
            task_id: args.task_id,
            execution_interval_millis: args.execution_interval_millis,
            iterations: args.iterations,
            instructions: vec![stream_ix],
        };
        let ix_data = bincode::serialize(&MagicBlockInstruction::ScheduleTask(schedule_args))
            .map_err(|_| ErrorCode::SerializationError)?;

        // Create the MagicBlock schedule instruction
        let schedule_ix = Instruction::new_with_bytes(
            MAGIC_PROGRAM_ID,
            &ix_data,
            vec![
                AccountMeta::new(self.pool_vault.key(), true),
                AccountMeta::new(self.pool.key(), false),
            ],
        );

        // Invoke Magic Program
        let seeds = &[
            POOL_VAULT_SEED.as_bytes(),
            self.pool.organization_pubkey.as_ref(),
            self.pool.species_id.as_bytes(),
            &[self.pool.pool_vault_bump],
        ];
        let signer_seeds = &seeds[..];

        invoke_signed(
            &schedule_ix,
            &[
                self.pool_vault.to_account_info(),
                self.pool.to_account_info(),
                self.magic_program.to_account_info(),
            ],
            &[signer_seeds],
        )?;

        msg!(
            "Crank scheduled: task_id={}, interval={}ms, iterations={}",
            args.task_id,
            args.execution_interval_millis,
            args.iterations
        );

        Ok(())
    }

    /// Builds the stream_yields instruction with all required accounts
    fn build_stream_yields_instruction(&self) -> Result<Instruction> {
        // Get associated mSOL token account
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
            data: crate::instruction::Stream {}.data(),
        })
    }
}
