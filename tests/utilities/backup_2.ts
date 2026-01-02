// import {
//   PublicKey,
//   Transaction,
//   Keypair,
//   SystemProgram,
//   LAMPORTS_PER_SOL,
// } from "@solana/web3.js";
// import { LiteSVM } from "litesvm";
// import * as anchor from "@coral-xyz/anchor";
// import { Program, AnchorProvider, Wallet } from "@coral-xyz/anchor";
// import { AtharvaRefi } from "../target/types/atharva_refi";
// import fs from "fs";
// import path from "path";
// import { expect } from "chai";
// import { PROGRAM_ID } from "./utilities/constants";
// import idl from "../target/idl/atharva_refi.json";

// describe("Create Pool Tests", () => {
//   let svm: LiteSVM;
//   let program: Program<AtharvaRefi>;
//   let admin: Keypair;
//   let provider: AnchorProvider;

//   before(async () => {
//     // Initialize LiteSVM
//     svm = new LiteSVM();

//     // Load your program
//     const programPath = path.join(
//       __dirname,
//       "../target/deploy/atharva_refi.so"
//     );
//     const programBuffer = fs.readFileSync(programPath);
//     svm.addProgram(PROGRAM_ID, programBuffer);

//     // Create admin keypair (this should match ADMIN_PUBKEY in your program)
//     admin = Keypair.generate();
//     svm.airdrop(admin.publicKey, BigInt(10 * LAMPORTS_PER_SOL));

//     // Create Anchor provider
//     const wallet = new Wallet(admin);
//     provider = anchor.AnchorProvider.env();
//     program = new Program(idl, provider);

//     // provider = new AnchorProvider(
//     //   {
//     //     // Mock connection methods for LiteSVM
//     //     getLatestBlockhash: async () => ({
//     //       blockhash: svm.latestBlockhash(),
//     //       lastValidBlockHeight: 0,
//     //     }),
//     //     getBalance: async (pubkey: PublicKey) => Number(svm.getBalance(pubkey)),
//     //     sendTransaction: async (tx: Transaction) => {
//     //       const result = svm.sendTransaction(tx);
//     //       return result.signature;
//     //     },
//     //     confirmTransaction: async () => ({ value: { err: null } }),
//     //   } as any,
//     //   wallet,
//     //   { commitment: "confirmed" }
//     // );

//     // Load IDL and create program instance
//     // const idl = JSON.parse(
//     //   fs.readFileSync(
//     //     path.join(__dirname, "../target/idl/atharva_refi.json"),
//     //     "utf8"
//     //   )
//     // );
//   });

//   it("Creates a pool successfully", async () => {
//     // Test data
//     const organizationPubkey = Keypair.generate().publicKey;
//     const organizationName = "Londolozi Reserve";
//     const speciesName = "African Lion";
//     const speciesId = "panthera_leo";

//     // Derive PDAs
//     const [poolPda, poolBump] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from(POOL_SEED),
//         organizationPubkey.toBuffer(),
//         Buffer.from(speciesId),
//       ],
//       PROGRAM_ID
//     );

//     const [poolVaultPda, poolVaultBump] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from(POOL_VAULT_SEED),
//         organizationPubkey.toBuffer(),
//         Buffer.from(speciesId),
//       ],
//       PROGRAM_ID
//     );

//     const [orgVaultPda, orgVaultBump] = PublicKey.findProgramAddressSync(
//       [Buffer.from(ORG_VAULT_SEED), organizationPubkey.toBuffer()],
//       PROGRAM_ID
//     );

//     // Build transaction using Anchor
//     const tx = await program.methods
//       .createPool(organizationName, organizationPubkey, speciesName, speciesId)
//       .accounts({
//         admin: admin.publicKey,
//         pool: poolPda,
//         poolVault: poolVaultPda,
//         organizationVault: orgVaultPda,
//         systemProgram: SystemProgram.programId,
//       })
//       .transaction();

//     // Set blockhash and sign
//     tx.recentBlockhash = svm.latestBlockhash();
//     tx.feePayer = admin.publicKey;
//     tx.sign(admin);

//     // Send transaction
//     const result = svm.sendTransaction(tx);

//     console.log("Transaction logs:", result.logs);

//     // Verify result
//     expect(result.err).to.be.null;

//     // Fetch and verify pool account
//     const poolAccount = await program.account.pool.fetch(poolPda);

//     expect(poolAccount.organizationPubkey.toString()).to.equal(
//       organizationPubkey.toString()
//     );
//     expect(poolAccount.organizationName).to.equal(organizationName);
//     expect(poolAccount.speciesName).to.equal(speciesName);
//     expect(poolAccount.speciesId).to.equal(speciesId);
//     expect(poolAccount.isActive).to.be.true;
//     expect(poolAccount.totalDeposits.toNumber()).to.equal(0);
//     expect(poolAccount.totalShares.toNumber()).to.equal(0);
//     expect(poolAccount.poolBump).to.equal(poolBump);
//     expect(poolAccount.orgVaultBump).to.equal(orgVaultBump);
//     expect(poolAccount.poolVaultBump).to.equal(poolVaultBump);

//     // Verify vault accounts exist
//     const poolVaultBalance = svm.getBalance(poolVaultPda);
//     const orgVaultBalance = svm.getBalance(orgVaultPda);

//     expect(poolVaultBalance).to.be.greaterThan(0n); // Rent-exempt balance
//     expect(orgVaultBalance).to.be.greaterThan(0n);
//   });

//   it("Fails when unauthorized user tries to create pool", async () => {
//     const unauthorizedUser = Keypair.generate();
//     svm.airdrop(unauthorizedUser.publicKey, BigInt(LAMPORTS_PER_SOL));

//     const organizationPubkey = Keypair.generate().publicKey;
//     const speciesId = "panthera_tigris";

//     const [poolPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from(POOL_SEED),
//         organizationPubkey.toBuffer(),
//         Buffer.from(speciesId),
//       ],
//       PROGRAM_ID
//     );

//     const [poolVaultPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from(POOL_VAULT_SEED),
//         organizationPubkey.toBuffer(),
//         Buffer.from(speciesId),
//       ],
//       PROGRAM_ID
//     );

//     const [orgVaultPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from(ORG_VAULT_SEED), organizationPubkey.toBuffer()],
//       PROGRAM_ID
//     );

//     try {
//       const tx = await program.methods
//         .createPool("Test Org", organizationPubkey, "Tiger", speciesId)
//         .accounts({
//           admin: unauthorizedUser.publicKey, // Wrong admin
//           pool: poolPda,
//           poolVault: poolVaultPda,
//           organizationVault: orgVaultPda,
//           systemProgram: SystemProgram.programId,
//         })
//         .transaction();

//       tx.recentBlockhash = svm.latestBlockhash();
//       tx.feePayer = unauthorizedUser.publicKey;
//       tx.sign(unauthorizedUser);

//       const result = svm.sendTransaction(tx);

//       // Should fail
//       expect(result.err).to.not.be.null;
//       console.log("Expected error:", result.err);
//     } catch (error) {
//       // Expected to fail
//       expect(error).to.exist;
//     }
//   });

//   it("Fails when string is too long", async () => {
//     const organizationPubkey = Keypair.generate().publicKey;
//     const tooLongName = "A".repeat(51); // Max is 50
//     const speciesId = "test_species";

//     const [poolPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from(POOL_SEED),
//         organizationPubkey.toBuffer(),
//         Buffer.from(speciesId),
//       ],
//       PROGRAM_ID
//     );

//     const [poolVaultPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from(POOL_VAULT_SEED),
//         organizationPubkey.toBuffer(),
//         Buffer.from(speciesId),
//       ],
//       PROGRAM_ID
//     );

//     const [orgVaultPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from(ORG_VAULT_SEED), organizationPubkey.toBuffer()],
//       PROGRAM_ID
//     );

//     try {
//       const tx = await program.methods
//         .createPool(tooLongName, organizationPubkey, "Species", speciesId)
//         .accounts({
//           admin: admin.publicKey,
//           pool: poolPda,
//           poolVault: poolVaultPda,
//           organizationVault: orgVaultPda,
//           systemProgram: SystemProgram.programId,
//         })
//         .transaction();

//       tx.recentBlockhash = svm.latestBlockhash();
//       tx.feePayer = admin.publicKey;
//       tx.sign(admin);

//       const result = svm.sendTransaction(tx);

//       expect(result.err).to.not.be.null;
//     } catch (error) {
//       expect(error).to.exist;
//     }
//   });

//   it("Creates multiple pools for same organization", async () => {
//     const organizationPubkey = Keypair.generate().publicKey;

//     // Create first pool (Lion)
//     const lionSpeciesId = "panthera_leo";
//     const [lionPoolPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from(POOL_SEED),
//         organizationPubkey.toBuffer(),
//         Buffer.from(lionSpeciesId),
//       ],
//       PROGRAM_ID
//     );

//     const [lionPoolVaultPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from(POOL_VAULT_SEED),
//         organizationPubkey.toBuffer(),
//         Buffer.from(lionSpeciesId),
//       ],
//       PROGRAM_ID
//     );

//     const [orgVaultPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from(ORG_VAULT_SEED), organizationPubkey.toBuffer()],
//       PROGRAM_ID
//     );

//     let tx = await program.methods
//       .createPool("Reserve", organizationPubkey, "Lion", lionSpeciesId)
//       .accounts({
//         admin: admin.publicKey,
//         pool: lionPoolPda,
//         poolVault: lionPoolVaultPda,
//         organizationVault: orgVaultPda,
//         systemProgram: SystemProgram.programId,
//       })
//       .transaction();

//     tx.recentBlockhash = svm.latestBlockhash();
//     tx.feePayer = admin.publicKey;
//     tx.sign(admin);

//     let result = svm.sendTransaction(tx);
//     expect(result.err).to.be.null;

//     // Create second pool (Elephant)
//     const elephantSpeciesId = "loxodonta_africana";
//     const [elephantPoolPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from(POOL_SEED),
//         organizationPubkey.toBuffer(),
//         Buffer.from(elephantSpeciesId),
//       ],
//       PROGRAM_ID
//     );

//     const [elephantPoolVaultPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from(POOL_VAULT_SEED),
//         organizationPubkey.toBuffer(),
//         Buffer.from(elephantSpeciesId),
//       ],
//       PROGRAM_ID
//     );

//     tx = await program.methods
//       .createPool("Reserve", organizationPubkey, "Elephant", elephantSpeciesId)
//       .accounts({
//         admin: admin.publicKey,
//         pool: elephantPoolPda,
//         poolVault: elephantPoolVaultPda,
//         organizationVault: orgVaultPda, // Same org vault
//         systemProgram: SystemProgram.programId,
//       })
//       .transaction();

//     tx.recentBlockhash = svm.latestBlockhash();
//     tx.feePayer = admin.publicKey;
//     tx.sign(admin);

//     result = svm.sendTransaction(tx);
//     expect(result.err).to.be.null;

//     // Verify both pools exist
//     const lionPool = await program.account.pool.fetch(lionPoolPda);
//     const elephantPool = await program.account.pool.fetch(elephantPoolPda);

//     expect(lionPool.speciesId).to.equal(lionSpeciesId);
//     expect(elephantPool.speciesId).to.equal(elephantSpeciesId);

//     // Both share same org vault
//     expect(lionPool.organizationPubkey.toString()).to.equal(
//       elephantPool.organizationPubkey.toString()
//     );
//   });
// });
