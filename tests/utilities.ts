import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import {
  PROGRAM_ID,
  POOL_SEED,
  POOL_VAULT_SEED,
  ORG_VAULT_SEED,
  walletPath,
  POOL_MINT_SEED,
} from "./constants";
import fs from "fs";

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
