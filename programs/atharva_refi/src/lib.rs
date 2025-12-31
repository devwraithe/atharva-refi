#![allow(deprecated, unexpected_cfgs)]
use anchor_lang::prelude::*;

mod constants;
mod errors;
mod events;
mod instructions;
mod marinade;
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
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        ctx.accounts.process(amount)
    }
    pub fn stream(ctx: Context<Stream>) -> Result<()> {
        ctx.accounts.process()
    }
    pub fn schedule_streams(ctx: Context<ScheduleStream>, args: ScheduleStreamArgs) -> Result<()> {
        ctx.accounts.process(args)
    }
    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        ctx.accounts.process(amount)
    }
    pub fn supporter_withdraw(ctx: Context<Withdraw>, share_amount: u64) -> Result<()> {
        ctx.accounts.process(share_amount)
    }
    pub fn organization_withdraw(ctx: Context<OrganizationWithdraw>, amount: u64) -> Result<()> {
        ctx.accounts.process(amount)
    }
}
