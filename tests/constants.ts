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
export const marinadePath = "tests/marinade";

// Marinade
export const M_PROGRAM_ID = new PublicKey(
  "MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD"
);
export const M_STATE = new PublicKey(
  "8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC"
);
export const MSOL_MINT = new PublicKey(
  "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So"
);
export const MSOL_MINT_AUTH = new PublicKey(
  "3JLPCS1qM2zRw3Dp6V4hZnYHd4toMNPkNesXdX9tg6KM"
);
export const LIQ_POOL_SOL_LEG = new PublicKey(
  "UefNb6z6yvArqe4cJHTXCqStRsKmWhGxnZzuHbikP5Q"
);
export const LIQ_POOL_MSOL_LEG = new PublicKey(
  "7GgPYjS5Dza89wV6FpZ23kUJRG5vbQ1GM25ezspYFSoE"
);
export const TREASURY_MSOL = new PublicKey(
  "8ZUcztoAEhpAeC2ixWewJKQJsSUGYSGPVAjkhDJYf5Gd"
);
export const RESERVE_PDA = new PublicKey(
  "Du3Ysj1wKbxPKkuPPnvzQLQh8oMSVifs3jGZjJWXFmHN"
);
export const MSOL_LEG_AUTH = new PublicKey(
  "EyaSjUtSgo9aRD1f8LWXwdvkpDTmXAW54yoSHZRF14WL"
);
