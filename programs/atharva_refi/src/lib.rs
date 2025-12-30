#![allow(deprecated, unexpected_cfgs)]
use anchor_lang::prelude::*;

mod actions;
mod constants;
mod errors;
mod events;
mod instructions;
mod states;
mod utilities;

use instructions::*;
use states::ScheduleStreamArgs;

declare_id!("HesZ7kke1KynNjhizTAAtRoxQZasxYqJ2oTrdw7JNkBx");

#[program]
pub mod atharva_refi {
    use super::*;

    pub fn create_pool(
        ctx: Context<CreatePool>,
        organization_name: String,
        organization_pubkey: Pubkey,
        species_name: String,
        species_id: String,
    ) -> Result<()> {
        ctx.accounts.process(
            organization_name,
            organization_pubkey,
            species_name,
            species_id,
            &ctx.bumps,
        )
    }
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.process(amount)
    }
    pub fn org_withdraw(ctx: Context<OrgWithdraw>, amount: u64) -> Result<()> {
        org_withdraw::handler(ctx, amount)
    }
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        ctx.accounts.handler(amount)
    }
    pub fn unstake(ctx: Context<LiquidUnstake>, amount: u64) -> Result<()> {
        ctx.accounts.process(amount)
    }
    pub fn stream_yields(ctx: Context<StreamYields>) -> Result<()> {
        ctx.accounts.process()
    }
    pub fn schedule_streams(ctx: Context<ScheduleStreams>, args: ScheduleStreamArgs) -> Result<()> {
        ctx.accounts.process(args)
    }
}
