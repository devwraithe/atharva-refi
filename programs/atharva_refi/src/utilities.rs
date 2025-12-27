pub fn calculate_ix_discriminator(ix_name: &str) -> Vec<u8> {
    // "global:" prefix for instructions
    let input = format!("global:{}", ix_name);

    // Hash the string using SHA256
    let hash_result = solana_program::hash::hash(input.as_bytes());

    hash_result.to_bytes()[..8].to_vec()
}
