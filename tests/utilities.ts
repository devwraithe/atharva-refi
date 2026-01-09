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
} from "./constants";
import fs from "fs";
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

// Fund an account with SOL
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
  console.log(`[DONE] Sent ${amountInSol} SOL to ${toPubkey.toBase58()}`);
}

// Show account balance
export async function getBalance(
  provider: anchor.Provider,
  owner: string,
  pubkey: PublicKey
) {
  const balance = await provider.connection.getBalance(pubkey);
  console.log(`${owner} Balance: ${balance} (${balance / LAMPORTS_PER_SOL})`);
}

export async function fetchBalance(
  provider: anchor.Provider,
  pubkey: PublicKey
): Promise<number> {
  const balance = await provider.connection.getBalance(pubkey);
  return balance / LAMPORTS_PER_SOL;
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

export async function fetchTokenBalance(
  provider: anchor.Provider,
  tokenAccount: PublicKey
): Promise<number> {
  const balance = await provider.connection.getTokenAccountBalance(
    tokenAccount
  );
  return balance.value.uiAmount;
}

export function logSignature(label: string, signature: string) {
  console.log(`\n${label} Txn Signature: ${signature}`);
}

export function logDone(label: String) {
  console.log(`[DONE] ${label}`);
}

export function logData(label: String) {
  console.log(`[DATA] ${label}`);
}

export function lamportsToSol(lamports: number): number {
  return lamports / LAMPORTS_PER_SOL;
}
