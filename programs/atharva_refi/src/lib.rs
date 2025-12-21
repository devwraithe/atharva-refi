use anchor_lang::prelude::*;

declare_id!("HesZ7kke1KynNjhizTAAtRoxQZasxYqJ2oTrdw7JNkBx");

#[program]
pub mod atharva_refi {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
