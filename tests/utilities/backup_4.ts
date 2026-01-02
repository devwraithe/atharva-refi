// import { PublicKey, SystemProgram } from "@solana/web3.js";
// import { PROGRAM_ID, POOL_SEED, POOL_VAULT_SEED, ORG_VAULT_SEED } from "./constants";

// // Helper to convert string to [u8; 50] buffer
// function stringToFixedBuffer(str: string, size: number): Buffer {
//   const buffer = Buffer.alloc(size);
//   buffer.write(str, 0, "utf-8");
//   return buffer;
// }

// // Get PDAs
// export const getPoolPdas = (
//   programId: PublicKey,
//   organizationPubkey: PublicKey,
//   speciesIdStr: string
// ) => {
//   const speciesIdBuffer = stringToFixedBuffer(speciesIdStr, 50);

//   const [poolPda] = PublicKey.findProgramAddressSync(
//     [Buffer.from(POOL_SEED), organizationPubkey.toBuffer(), speciesIdBuffer],
//     programId
//   );

//   const [poolVaultPda] = PublicKey.findProgramAddressSync(
//     [Buffer.from(POOL_VAULT_SEED), organizationPubkey.toBuffer(), speciesIdBuffer],
//     programId
//   );

//   const [orgVaultPda] = PublicKey.findProgramAddressSync(
//     [Buffer.from(ORG_VAULT_SEED), organizationPubkey.toBuffer()],
//     programId
//   );

//   return { poolPda, poolVaultPda, orgVaultPda };
// };

// describe("Create Pool Tests", () => {
//   it("creates a lion pool for Londolozi fund", async () => {
//     const organizationPubkey = organization.publicKey;
//     const organizationName = "Londolozi Reserve";
//     const speciesName = "African Lion";
//     const speciesId = "panthera_leo";

//     // Convert species_id to fixed buffer
//     const speciesIdBuffer = stringToFixedBuffer(speciesId, 50);

//     const { poolPda, poolVaultPda, orgVaultPda } = getPoolPdas(
//       PROGRAM_ID,
//       organizationPubkey,
//       speciesId
//     );

//     await program.methods
//       .createPool(
//         organizationName,
//         organizationPubkey,
//         speciesName,
//         Array.from(speciesIdBuffer) // Convert to number array
//       )
//       .accountsStrict({
//         admin: admin.publicKey,
//         pool: poolPda,
//         poolVault: poolVaultPda,
//         organizationVault: orgVaultPda,
//         systemProgram: SystemProgram.programId,
//       })
//       .signers([admin])
//       .rpc();

//     // Verify pool was created
//     const poolAccount = await program.account.pool.fetch(poolPda);

//     console.log("Pool created:");
//     console.log("- Organization:", poolAccount.organizationName);
//     console.log("- Species:", poolAccount.speciesName);
//     console.log("- Species ID:", poolAccount.speciesIdStr()); // Use helper method
//     console.log("- Active:", poolAccount.isActive);
//     console.log("- Yield BPS:", poolAccount.organizationYieldBps);
//   });

//   it("creates multiple pools for same organization", async () => {
//     const organizationPubkey = organization.publicKey;
//     const organizationName = "Londolozi Reserve";

//     // Create Lion pool
//     const lionSpeciesId = "panthera_leo";
//     const lionSpeciesIdBuffer = stringToFixedBuffer(lionSpeciesId, 50);
//     const { poolPda: lionPoolPda, poolVaultPda: lionVaultPda, orgVaultPda } = getPoolPdas(
//       PROGRAM_ID,
//       organizationPubkey,
//       lionSpeciesId
//     );

//     await program.methods
//       .createPool(
//         organizationName,
//         organizationPubkey,
//         "African Lion",
//         Array.from(lionSpeciesIdBuffer)
//       )
//       .accountsStrict({
//         admin: admin.publicKey,
//         pool: lionPoolPda,
//         poolVault: lionVaultPda,
//         organizationVault: orgVaultPda,
//         systemProgram: SystemProgram.programId,
//       })
//       .signers([admin])
//       .rpc();

//     // Create Elephant pool (same org, different species)
//     const elephantSpeciesId = "loxodonta_africana";
//     const elephantSpeciesIdBuffer = stringToFixedBuffer(elephantSpeciesId, 50);
//     const { poolPda: elephantPoolPda, poolVaultPda: elephantVaultPda } = getPoolPdas(
//       PROGRAM_ID,
//       organizationPubkey,
//       elephantSpeciesId
//     );

//     await program.methods
//       .createPool(
//         organizationName,
//         organizationPubkey,
//         "African Elephant",
//         Array.from(elephantSpeciesIdBuffer)
//       )
//       .accountsStrict({
//         admin: admin.publicKey,
//         pool: elephantPoolPda,
//         poolVault: elephantVaultPda,
//         organizationVault: orgVaultPda, // Same org vault
//         systemProgram: SystemProgram.programId,
//       })
//       .signers([admin])
//       .rpc();

//     // Verify both pools exist
//     const lionPool = await program.account.pool.fetch(lionPoolPda);
//     const elephantPool = await program.account.pool.fetch(elephantPoolPda);

//     console.log("Lion pool species:", lionPool.speciesIdStr());
//     console.log("Elephant pool species:", elephantPool.speciesIdStr());
//   });
// });
