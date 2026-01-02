import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import {
  PROGRAM_ID,
  POOL_SEED,
  POOL_VAULT_SEED,
  ORG_VAULT_SEED,
  walletPath,
  POOL_MINT_SEED,
  LIQ_POOL_MSOL_LEG,
  LIQ_POOL_SOL_LEG,
  M_PROGRAM_ID,
  M_STATE,
  MSOL_LEG_AUTH,
  MSOL_MINT,
  MSOL_MINT_AUTH,
  RESERVE_PDA,
} from "./constants";
import fs from "fs";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

// Load external account

export function loadAccount(
  svm: any,
  address: PublicKey,
  filePath: string,
  owner: PublicKey
) {
  try {
    const content = fs.readFileSync(filePath, "utf8");

    // Check if file is empty
    if (!content || content.trim().length === 0) {
      throw new Error(`File is empty: ${filePath}`);
    }

    const data = JSON.parse(content);

    svm.setAccount(address, {
      lamports: data.account.lamports,
      data: Buffer.from(data.account.data[0], "base64"),
      owner: owner,
      executable: false,
    });
  } catch (error) {
    console.error(`❌ Failed to load account from ${filePath}`);
    throw error; // Re-throw to stop the test
  }
}

/**
 * Centralized loader for Marinade-related accounts in LiteSVM
 */
export function loadMarinadeAccounts(svm: any, marinadePath: string) {
  // Mapping of constant addresses to their specific filenames and owners
  const marinadeAccounts = [
    { address: M_STATE, file: "marinade_state.json", owner: M_PROGRAM_ID },
    { address: MSOL_MINT, file: "msol_mint.json", owner: TOKEN_PROGRAM_ID },
    {
      address: RESERVE_PDA,
      file: "reserve_pda.json",
      owner: SystemProgram.programId,
    },
    {
      address: MSOL_MINT_AUTH,
      file: "msol_mint_auth.json",
      owner: M_PROGRAM_ID,
    },
    {
      address: LIQ_POOL_SOL_LEG,
      file: "liq_pool_sol_leg.json",
      owner: SystemProgram.programId,
    },
    {
      address: LIQ_POOL_MSOL_LEG,
      file: "liq_pool_msol_leg.json",
      owner: TOKEN_PROGRAM_ID,
    },
    { address: MSOL_LEG_AUTH, file: "msol_leg_auth.json", owner: M_PROGRAM_ID },
  ];

  // Load the program bytecode first
  svm.addProgram(M_PROGRAM_ID, fs.readFileSync(`${marinadePath}/marinade.so`));

  // Iterate and load each account state
  for (const acc of marinadeAccounts) {
    loadAccount(svm, acc.address, `${marinadePath}/${acc.file}`, acc.owner);
  }

  console.log("✅ All Marinade accounts loaded into LiteSVM");
}

export function getOrCreateAdminWallet(): Keypair {
  if (fs.existsSync(walletPath)) {
    return loadKeypairFromFile(walletPath);
  } else {
    const keypair = Keypair.generate();
    fs.writeFileSync(walletPath, JSON.stringify(Array.from(keypair.secretKey)));
    console.log("Created a new admin wallet:", walletPath);
    return keypair;
  }
}

export function loadKeypairFromFile(secretFilePath: string) {
  const secret = JSON.parse(fs.readFileSync(secretFilePath, "utf-8"));
  const secretKey = Uint8Array.from(secret);
  return Keypair.fromSecretKey(secretKey);
}

// Helper to convert string to [u8; 50] buffer
export function stringToFixedBuffer(str: string, size: number): Buffer {
  const buffer = Buffer.alloc(size);
  buffer.write(str, 0, "utf-8");
  return buffer;
}

// Convert string to fixed-size byte array
export function stringToBytes(str: string, size: number): number[] {
  const buffer = Buffer.alloc(size);
  buffer.write(str, 0, "utf-8");
  return Array.from(buffer);
}

// Convert byte array back to string
export function bytesToString(bytes: number[] | Uint8Array): string {
  const buffer = Buffer.from(bytes);
  const nullIndex = buffer.indexOf(0);
  const validBytes = nullIndex === -1 ? buffer : buffer.slice(0, nullIndex);
  return validBytes.toString("utf-8");
}

// Get PDAs
export const getPoolPdas = (
  organizationPubkey: PublicKey,
  speciesIdBytes: number[]
) => {
  const speciesSeed = Buffer.from(speciesIdBytes);

  const [poolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(POOL_SEED), organizationPubkey.toBuffer(), speciesSeed],
    PROGRAM_ID
  );

  const [poolVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(POOL_VAULT_SEED), organizationPubkey.toBuffer(), speciesSeed],
    PROGRAM_ID
  );

  const [orgVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(ORG_VAULT_SEED), organizationPubkey.toBuffer(), speciesSeed],
    PROGRAM_ID
  );

  const [poolMintPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(POOL_MINT_SEED), organizationPubkey.toBuffer(), speciesSeed],
    PROGRAM_ID
  );

  return { poolPda, poolMintPda, poolVaultPda, orgVaultPda };
};
