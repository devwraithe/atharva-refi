use anchor_lang::{prelude::*, InstructionData};

use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
};
use ephemeral_rollups_sdk::consts::MAGIC_PROGRAM_ID;
use magicblock_magic_program_api::{args::ScheduleTaskArgs, instruction::MagicBlockInstruction};

use crate::constants::{marinade_finance, POOL_SEED, POOL_VAULT_SEED};
use crate::errors::ErrorCode;
use crate::states::{Pool, ScheduleStreamArgs};

/// Schedules automated yield streaming via MagicBlock Cranks
///
/// The crank will periodically call stream_yields instruction to:
/// - Calculate accumulated yield from Marinade staking
/// - Distribute org's percentage to organization vault
/// - Keep remainder in pool for supporters

#[derive(Accounts)]
pub struct ScheduleStreams<'info> {
    // Admin or authorized caller
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED.as_bytes(), pool.organization_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump = pool.pool_bump,
        constraint = pool.is_active @ ErrorCode::PoolNotActive,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [POOL_VAULT_SEED.as_bytes(), pool.key().as_ref(), pool.organization_pubkey.as_ref()],
        bump = pool.pool_vault_bump,
    )]
    pub pool_vault: SystemAccount<'info>,

    /// CHECK: used for CPI
    #[account()]
    pub magic_program: AccountInfo<'info>,

    /// CHECK: used for CPI
    pub program: AccountInfo<'info>,
}
impl<'info> ScheduleStreams<'info> {
    pub fn process(&mut self, args: ScheduleStreamArgs) -> Result<()> {
        // Validation
        require!(
            args.execution_interval_millis >= 86_400_000,
            ErrorCode::IntervalTooShort
        ); // 1 day min
        require!(args.iterations > 0, ErrorCode::InvalidIterations);

        // Update pool state to track scheduling
        self.pool.last_settlement_ts = Clock::get()?.unix_timestamp as u64;
        self.pool.is_crank_scheduled = true;

        // Build stream yields instruction
        let stream_yields_ix = self.build_stream_yields_instruction()?;

        // Serialize MagicBlock schedule task
        let schedule_args = ScheduleTaskArgs {
            task_id: args.task_id,
            execution_interval_millis: args.execution_interval_millis,
            iterations: args.iterations,
            instructions: vec![stream_yields_ix],
        };

        let ix_data = bincode::serialize(&MagicBlockInstruction::ScheduleTask(schedule_args))
            .map_err(|_| ErrorCode::SerializationError)?;

        // Build MagicBlock schedule instruction
        let schedule_ix = Instruction::new_with_bytes(
            MAGIC_PROGRAM_ID,
            &ix_data,
            vec![
                AccountMeta::new(self.pool_vault.key(), true), // Payer & signer
                AccountMeta::new(self.pool.key(), false),      // Pool to update
            ],
        );

        // Invoke with PDA signing
        let pool_key = self.pool.key();
        let vault_seeds = &[
            POOL_VAULT_SEED.as_bytes(),
            pool_key.as_ref(),
            self.pool.organization_pubkey.as_ref(),
            &[self.pool.pool_vault_bump],
        ];
        let signer_seeds = &vault_seeds[..];

        invoke_signed(
            &schedule_ix,
            &[
                self.pool_vault.to_account_info(),
                self.pool.to_account_info(),
            ],
            &[signer_seeds],
        )?;

        msg!(
            "Crank scheduled: task_id={}, interval={}ms",
            args.task_id,
            args.execution_interval_millis
        );

        Ok(())
    }

    /// Builds the stream_yields instruction with all required accounts
    fn build_stream_yields_instruction(&self) -> Result<Instruction> {
        // Get associated mSOL token account
        let pool_msol_account = anchor_spl::associated_token::get_associated_token_address(
            &self.pool_vault.key(),
            &marinade_finance::MSOL_MINT,
        );

        // Derive org vault PDA
        let (org_vault, _) = Pubkey::find_program_address(
            &[b"org_vault", self.pool.organization_pubkey.as_ref()],
            &crate::ID,
        );

        Ok(Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(self.pool_vault.key(), true),  // payer
                AccountMeta::new(self.pool.key(), false),       // pool
                AccountMeta::new(self.pool_vault.key(), false), // pool_vault
                AccountMeta::new(pool_msol_account, false),     // pool_msol_account
                AccountMeta::new(org_vault, false),             // organization_vault
                AccountMeta::new_readonly(marinade_finance::STATE, false), // marinade_state
                AccountMeta::new(marinade_finance::MSOL_MINT, false), // msol_mint
                AccountMeta::new(marinade_finance::LIQ_POOL_SOL_LEG, false),
                AccountMeta::new(marinade_finance::LIQ_POOL_MSOL_LEG, false),
                AccountMeta::new(marinade_finance::TREASURY_MSOL, false),
                AccountMeta::new_readonly(marinade_finance::PROGRAM_ID, false),
                AccountMeta::new_readonly(anchor_spl::token::ID, false),
                AccountMeta::new_readonly(anchor_lang::system_program::ID, false),
            ],
            data: crate::instruction::StreamYields {}.data(),
        })
    }
}
