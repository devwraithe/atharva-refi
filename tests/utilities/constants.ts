import { PublicKey } from "@solana/web3.js";
import idl from "../../target/idl/atharva_refi.json";

export const POOL_SEED = "pool";
export const POOL_VAULT_SEED = "pool_vault";
export const POOL_MINT_SEED = "pool_mint";
export const ORG_VAULT_SEED = "organization_vault";
export const PROGRAM_ID = new PublicKey(idl.address);
export const ADMIN_PUBKEY = new PublicKey(
  "ExYgW7TinWCdrUGEo6kYxK4AQKRvTgbqD7p1GdDchEuW"
);

export const walletPath = "./tests/admin_wallet.json";
