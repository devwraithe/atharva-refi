// import { PublicKey } from "@solana/web3.js";
// import {
//   ORG_VAULT_SEED,
//   POOL_SEED,
//   POOL_VAULT_SEED,
//   PROGRAM_ID,
// } from "./constants";

// export function getPda(
//   seed: String,
//   organizationPubkey: PublicKey,
//   speciesId: String
// ) {
//   return PublicKey.findProgramAddressSync(
//     [Buffer.from(seed), organizationPubkey.toBuffer(), Buffer.from(speciesId)],
//     PROGRAM_ID
//   );
// }

// export const getPoolPdas = (
//   programId: PublicKey,
//   organizationPubkey: PublicKey,
//   speciesId: string // Plain string, max 32 chars
// ) => {
//   // Validate length
//   if (speciesId.length > 32) {
//     throw new Error("Species ID must be 32 characters or less");
//   }

//   const [poolPda] = PublicKey.findProgramAddressSync(
//     [
//       Buffer.from(POOL_SEED),
//       organizationPubkey.toBuffer(),
//       Buffer.from(speciesId), // Direct string to buffer
//     ],
//     programId
//   );

//   const [poolVaultPda] = PublicKey.findProgramAddressSync(
//     [
//       Buffer.from(POOL_VAULT_SEED),
//       organizationPubkey.toBuffer(),
//       Buffer.from(speciesId),
//     ],
//     programId
//   );

//   const [orgVaultPda] = PublicKey.findProgramAddressSync(
//     [Buffer.from(ORG_VAULT_SEED), organizationPubkey.toBuffer()],
//     programId
//   );

//   return { poolPda, poolVaultPda, orgVaultPda };
// };
