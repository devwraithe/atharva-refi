import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {
  PROGRAM_ID,
  POOL_SEED,
  POOL_VAULT_SEED,
  ORG_VAULT_SEED,
  walletPath,
  POOL_MINT_SEED,
  LIQ_POOL_MSOL_LEG,
  LIQ_POOL_SOL_LEG,
  MSOL_LEG_AUTH,
  MSOL_MINT,
  MSOL_MINT_AUTH,
  RESERVE_PDA,
  TREASURY_MSOL,
  MB_PROGRAM_ID,
  mblockPath,
  MAGIC_BLOCK_PROGRAM_ID,
  MAR_STATE,
  MAR_PROGRAM_ID,
} from "./constants";
import fs from "fs";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  DELEGATION_PROGRAM_ID,
  MAGIC_PROGRAM_ID,
} from "@magicblock-labs/ephemeral-rollups-sdk";
import * as anchor from "@coral-xyz/anchor";

// Simulate wait times for rollup processing
export async function waitForRollup(
  durationMs: number,
  action = "Processing"
): Promise<void> {
  const TICK_MS = 500;
  const totalTicks = Math.ceil(durationMs / TICK_MS);

  for (let tick = 0; tick < totalTicks; tick++) {
    const dots = ".".repeat((tick % 3) + 1).padEnd(3, " ");

    const remainingMs = Math.max(0, durationMs - tick * TICK_MS);
    const remainingSec = (remainingMs / 1000).toFixed(1);

    process.stdout.write(
      `\r[MagicBlock] ${action}${dots} (${remainingSec}s remaining)`
    );

    await sleep(TICK_MS);
  }

  clearLine();
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function clearLine() {
  process.stdout.write("\r\x1b[K");
}

// Mock MagicBlock program (using Atharva ReFi as donor)
export const mockMagicBlockProgram = (svm: any) => {
  const MAGIC_BLOCK_ID = MAGIC_BLOCK_PROGRAM_ID;

  // Load my compiled program bytecode for a vaild ELF/SBF for LiteSVM
  const donorBytecode = fs.readFileSync("./target/deploy/atharva_refi.so");

  // SBF executable that returns success
  const NO_OP_PROGRAM_SBF = Buffer.from(
    "7f454c4601010100000000000000000002002800010000000000000000000000" +
      "0000000000000000340000000000000000000000340020000100000000000000" +
      "010000000000000000000000000000000000000000000000d000000000000000" +
      "d00000000000000005000000000000000010000000000000b700000000000000" +
      "9500000000000000",
    "hex"
  );

  // Force-inject into LiteSVM
  svm.addProgram(MAGIC_BLOCK_ID, NO_OP_PROGRAM_SBF);

  console.log("✅ MagicBlock Crank Mocked at:", MAGIC_BLOCK_ID.toBase58());
};

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

export function loadMagicBlock(svm: any) {
  // svm.addProgram(
  //   MAGIC_PROGRAM_ID,
  //   fs.readFileSync(`${mblockPath}/magic_block.so`)
  // );

  mockMagicBlockProgram(svm);

  // svm.addProgram(
  //   DELEGATION_PROGRAM_ID,
  //   fs.readFileSync(`${mblockPath}/delegation_program.so`)
  // );

  console.log("MagicBlock accounts loaded!");
}

/**
 * Centralized loader for Marinade-related accounts in LiteSVM
 */
export function loadMarinadeAccounts(svm: any, marinadePath: string) {
  // Mapping of constant addresses to their specific filenames and owners
  const marinadeAccounts = [
    { address: MAR_STATE, file: "marinade_state.json", owner: MAR_PROGRAM_ID },
    { address: MSOL_MINT, file: "msol_mint.json", owner: TOKEN_PROGRAM_ID },
    {
      address: RESERVE_PDA,
      file: "reserve_pda.json",
      owner: SystemProgram.programId,
    },
    {
      address: MSOL_MINT_AUTH,
      file: "msol_mint_auth.json",
      owner: MAR_PROGRAM_ID,
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
    {
      address: MSOL_LEG_AUTH,
      file: "msol_leg_auth.json",
      owner: MAR_PROGRAM_ID,
    },
    {
      address: TREASURY_MSOL,
      file: "treasury_msol.json",
      owner: TOKEN_PROGRAM_ID,
    },
  ];

  // Load the program bytecode first
  svm.addProgram(
    MAR_PROGRAM_ID,
    fs.readFileSync(`${marinadePath}/marinade.so`)
  );

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

export async function fundAccount(
  connection: anchor.web3.Connection,
  payer: anchor.web3.Keypair,
  toPubkey: PublicKey,
  amountInSol: number
) {
  const transaction = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: payer.publicKey,
      toPubkey: toPubkey,
      lamports: amountInSol * LAMPORTS_PER_SOL,
    })
  );

  await sendAndConfirmTransaction(connection, transaction, [payer]);
  console.log(`✅ Transferred ${amountInSol} SOL to ${toPubkey.toBase58()}`);
}

// Show account balance
export async function getBalance(
  provider: anchor.Provider,
  owner: string,
  pubkey: PublicKey
) {
  const supporterBalance = await provider.connection.getBalance(pubkey);
  console.log(
    `${owner} Balance: ${supporterBalance} (${
      supporterBalance / LAMPORTS_PER_SOL
    })`
  );
}

// Show token account balance
export async function getTokenBalance(
  provider: anchor.Provider,
  ownerLabel: string,
  tokenAccount: PublicKey
): Promise<number> {
  const balance = await provider.connection.getTokenAccountBalance(
    tokenAccount
  );

  console.log(
    `${ownerLabel} Token Balance: ${balance.value.amount} (${
      balance.value.uiAmount ?? 0
    })`
  );

  return balance.value.uiAmount;
}
