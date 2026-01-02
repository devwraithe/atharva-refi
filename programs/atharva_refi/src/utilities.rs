use anchor_lang::solana_program::hash;

pub fn calculate_ix_discriminator(ix_name: &str) -> Vec<u8> {
    // "global:" prefix for instructions
    let input = format!("global:{}", ix_name);

    // Hash the string using SHA256
    let hash_result = hash::hash(input.as_bytes());

    hash_result.to_bytes()[..8].to_vec()
}

pub fn bytes_to_string(bytes: &[u8]) -> String {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..len]).to_string()
}
