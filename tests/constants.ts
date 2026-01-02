import { PublicKey } from "@solana/web3.js";
import idl from "../target/idl/atharva_refi.json";

export const POOL_SEED = "pool";
export const POOL_VAULT_SEED = "pool_vault";
export const POOL_MINT_SEED = "pool_mint";
export const ORG_VAULT_SEED = "organization_vault";
export const PROGRAM_ID = new PublicKey(idl.address);
export const ADMIN_PUBKEY = new PublicKey(
  "BkDW9kxJxVC2KGDs94GQRpkowbWfpG1N7sX7HqsNCSL7"
);

export const walletPath = "./tests/admin_wallet.json";
