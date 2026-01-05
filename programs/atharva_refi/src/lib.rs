#![allow(deprecated, unexpected_cfgs)]
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;

mod constants;
mod errors;
mod events;
mod instructions;
mod marinade;
mod states;
mod utilities;

use instructions::*;
use states::ScheduleStreamArgs;

declare_id!("5MQdy7SUtMR5qQqryuizd7WXKE18RRn7sNS4uX64ih96");

#[ephemeral]
#[program]
pub mod atharva_refi {

    use super::*;

    pub fn create_pool(
        ctx: Context<CreatePool>,
        organization_name: String,
        organization_pubkey: Pubkey,
        species_name: String,
        species_id: [u8; 32],
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
    pub fn supporter_withdraw(ctx: Context<SupporterWithdraw>, share_amount: u64) -> Result<()> {
        ctx.accounts.process(share_amount)
    }
    pub fn organization_withdraw(ctx: Context<OrganizationWithdraw>, amount: u64) -> Result<()> {
        ctx.accounts.process(amount)
    }
    pub fn delegate(ctx: Context<DelegatePool>) -> Result<()> {
        delegate_process(ctx)
    }
    pub fn undelegate(ctx: Context<UndelegatePool>) -> Result<()> {
        undelegate_process(ctx)
    }
}
