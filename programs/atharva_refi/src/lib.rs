use anchor_lang::prelude::*;

mod constants;
mod errors;
mod events;
mod instructions;
mod states;

use instructions::*;

declare_id!("HesZ7kke1KynNjhizTAAtRoxQZasxYqJ2oTrdw7JNkBx");

#[program]
pub mod atharva_refi {
    use super::*;

    pub fn create_pool(
        ctx: Context<CreatePool>,
        org_name: String,
        org_pubkey: Pubkey,
        species_name: String,
        species_id: String,
    ) -> Result<()> {
        create_pool::handler(ctx, org_name, org_pubkey, species_name, species_id)
    }
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        deposit::handler(ctx, amount)
    }
}
