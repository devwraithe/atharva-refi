use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ScheduleStreamArgs {
    pub task_id: u64,                   // Task unique identifier
    pub execution_interval_millis: u64, // Time between execution in milliseconds, 2_592_000_000 for 30 days
    pub iterations: u64,                // Numbber of times to execute, 0 = infinite
}
