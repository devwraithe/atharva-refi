// import { Keypair, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
// import { Program } from "@coral-xyz/anchor";
// import { AtharvaRefi } from "../target/types/atharva_refi";
// import {
//   ORG_VAULT_SEED,
//   POOL_SEED,
//   POOL_VAULT_SEED,
// } from "./utilities/constants";
// import idl from "../target/idl/atharva_refi.json";
// import { getPda } from "./utilities/pdas";
// import { fromWorkspace, LiteSVMProvider } from "anchor-litesvm";

// describe("atharva refi tests", () => {
//   const svm = fromWorkspace("./");

//   let provider: LiteSVMProvider;
//   let program: Program<AtharvaRefi>;

//   let admin: Keypair;
//   let supporter: Keypair;
//   let organization: Keypair;

//   before(async () => {
//     provider = new LiteSVMProvider(svm);
//     program = new Program<AtharvaRefi>(idl, provider);

//     admin = Keypair.generate();
//     supporter = Keypair.generate();
//     organization = Keypair.generate();

//     svm.airdrop(admin.publicKey, BigInt(10 * LAMPORTS_PER_SOL));
//     svm.airdrop(supporter.publicKey, BigInt(10 * LAMPORTS_PER_SOL));
//     svm.airdrop(organization.publicKey, BigInt(10 * LAMPORTS_PER_SOL));
//   });

//   it("creates a lion pool for londolozi fund", async () => {
//     const organizationPubkey = organization.publicKey;
//     const organizationName = "Londolozi Reserve";
//     const speciesName = "African Lion";
//     const speciesId = "panthera_leo";

//     const [poolPda] = getPda(POOL_SEED, organizationPubkey, speciesId);
//     const [poolVaultPda] = getPda(
//       POOL_VAULT_SEED,
//       organizationPubkey,
//       speciesId
//     );
//     const [orgVaultPda] = getPda(ORG_VAULT_SEED, organizationPubkey, speciesId);

//     await program.methods
//       .createPool(organizationName, organizationPubkey, speciesName, speciesId)
//       .accountsStrict({
//         admin: admin.publicKey,
//         pool: poolPda,
//         poolVault: poolVaultPda,
//         organizationVault: orgVaultPda,
//         systemProgram: SystemProgram.programId,
//       })
//       .signers([admin])
//       .rpc();

//     // expect(result.err).to.be.null;
//     // console.log("Londolozi Lion Pool Created: ", result);

//     // // -----------------------------
//     // // Verify state (LiteSVM)
//     // // -----------------------------
//     // const poolAccountInfo = svm.getAccount(poolPda);
//     // // expect(poolAccountInfo).to.not.be.null;

//     // console.log("Pool Account Info: ", poolAccountInfo);

//     // const poolAccount = program.coder.accounts.decode(
//     //   "Pool",
//     //   poolAccountInfo.data
//     // );

//     // console.log("Created Pool Account:", poolAccount);

//     // expect(poolAccount.organization).to.eql(organizationPubkey);
//     // expect(poolAccount.speciesId).to.eql(speciesId);

//     // // -----------------------------
//     // // Assert execution success
//     // // -----------------------------
//     // expect(result.err).to.be.null;

//     // // -----------------------------
//     // // Read state via LiteSVM
//     // // -----------------------------
//     // const poolAccountInfo = svm.getAccount(poolPda);
//     // expect(poolAccountInfo).to.not.be.null;

//     // // Optional: decode with Anchor coder
//     // const poolAccount = program.coder.accounts.decode(
//     //   "Pool",
//     //   poolAccountInfo!.data
//     // );

//     // console.log("Created Pool Account:", poolAccount);

//     // expect(poolAccount.organization).to.eql(organizationPubkey);
//     // expect(poolAccount.speciesId).to.eql(speciesId);

//     // expect(poolAccount.organizationPubkey.toString()).to.equal(
//     //   organizationPubkey.toString()
//     // );
//     // expect(poolAccount.organizationName).to.equal(organizationName);
//     // expect(poolAccount.speciesName).to.equal(speciesName);
//     // expect(poolAccount.speciesId).to.equal(speciesId);
//     // expect(poolAccount.isActive).to.be.true;
//     // expect(poolAccount.totalDeposits.toNumber()).to.equal(0);
//     // expect(poolAccount.totalShares.toNumber()).to.equal(0);
//     // expect(poolAccount.poolBump).to.equal(poolBump);
//     // expect(poolAccount.orgVaultBump).to.equal(orgVaultBump);
//     // expect(poolAccount.poolVaultBump).to.equal(poolVaultBump);

//     // // Verify vault accounts exist
//     // const poolVaultBalance = svm.getBalance(poolVaultPda);
//     // const orgVaultBalance = svm.getBalance(orgVaultPda);

//     // expect(poolVaultBalance).to.be.greaterThan(0n); // Rent-exempt balance
//     // expect(orgVaultBalance).to.be.greaterThan(0n);
//   });

//   // it("Fails when unauthorized user tries to create pool", async () => {
//   //   const unauthorizedUser = Keypair.generate();
//   //   svm.airdrop(unauthorizedUser.publicKey, BigInt(LAMPORTS_PER_SOL));

//   //   const organizationPubkey = Keypair.generate().publicKey;
//   //   const speciesId = "panthera_tigris";

//   //   const [poolPda] = PublicKey.findProgramAddressSync(
//   //     [
//   //       Buffer.from(POOL_SEED),
//   //       organizationPubkey.toBuffer(),
//   //       Buffer.from(speciesId),
//   //     ],
//   //     PROGRAM_ID
//   //   );

//   //   const [poolVaultPda] = PublicKey.findProgramAddressSync(
//   //     [
//   //       Buffer.from(POOL_VAULT_SEED),
//   //       organizationPubkey.toBuffer(),
//   //       Buffer.from(speciesId),
//   //     ],
//   //     PROGRAM_ID
//   //   );

//   //   const [orgVaultPda] = PublicKey.findProgramAddressSync(
//   //     [Buffer.from(ORG_VAULT_SEED), organizationPubkey.toBuffer()],
//   //     PROGRAM_ID
//   //   );

//   //   try {
//   //     const tx = await program.methods
//   //       .createPool("Test Org", organizationPubkey, "Tiger", speciesId)
//   //       .accounts({
//   //         admin: unauthorizedUser.publicKey, // Wrong admin
//   //         pool: poolPda,
//   //         poolVault: poolVaultPda,
//   //         organizationVault: orgVaultPda,
//   //         systemProgram: SystemProgram.programId,
//   //       })
//   //       .transaction();

//   //     tx.recentBlockhash = svm.latestBlockhash();
//   //     tx.feePayer = unauthorizedUser.publicKey;
//   //     tx.sign(unauthorizedUser);

//   //     const result = svm.sendTransaction(tx);

//   //     // Should fail
//   //     expect(result.err).to.not.be.null;
//   //     console.log("Expected error:", result.err);
//   //   } catch (error) {
//   //     // Expected to fail
//   //     expect(error).to.exist;
//   //   }
//   // });

//   // it("Fails when string is too long", async () => {
//   //   const organizationPubkey = Keypair.generate().publicKey;
//   //   const tooLongName = "A".repeat(51); // Max is 50
//   //   const speciesId = "test_species";

//   //   const [poolPda] = PublicKey.findProgramAddressSync(
//   //     [
//   //       Buffer.from(POOL_SEED),
//   //       organizationPubkey.toBuffer(),
//   //       Buffer.from(speciesId),
//   //     ],
//   //     PROGRAM_ID
//   //   );

//   //   const [poolVaultPda] = PublicKey.findProgramAddressSync(
//   //     [
//   //       Buffer.from(POOL_VAULT_SEED),
//   //       organizationPubkey.toBuffer(),
//   //       Buffer.from(speciesId),
//   //     ],
//   //     PROGRAM_ID
//   //   );

//   //   const [orgVaultPda] = PublicKey.findProgramAddressSync(
//   //     [Buffer.from(ORG_VAULT_SEED), organizationPubkey.toBuffer()],
//   //     PROGRAM_ID
//   //   );

//   //   try {
//   //     const tx = await program.methods
//   //       .createPool(tooLongName, organizationPubkey, "Species", speciesId)
//   //       .accounts({
//   //         admin: admin.publicKey,
//   //         pool: poolPda,
//   //         poolVault: poolVaultPda,
//   //         organizationVault: orgVaultPda,
//   //         systemProgram: SystemProgram.programId,
//   //       })
//   //       .transaction();

//   //     tx.recentBlockhash = svm.latestBlockhash();
//   //     tx.feePayer = admin.publicKey;
//   //     tx.sign(admin);

//   //     const result = svm.sendTransaction(tx);

//   //     expect(result.err).to.not.be.null;
//   //   } catch (error) {
//   //     expect(error).to.exist;
//   //   }
//   // });

//   // it("Creates multiple pools for same organization", async () => {
//   //   const organizationPubkey = Keypair.generate().publicKey;

//   //   // Create first pool (Lion)
//   //   const lionSpeciesId = "panthera_leo";
//   //   const [lionPoolPda] = PublicKey.findProgramAddressSync(
//   //     [
//   //       Buffer.from(POOL_SEED),
//   //       organizationPubkey.toBuffer(),
//   //       Buffer.from(lionSpeciesId),
//   //     ],
//   //     PROGRAM_ID
//   //   );

//   //   const [lionPoolVaultPda] = PublicKey.findProgramAddressSync(
//   //     [
//   //       Buffer.from(POOL_VAULT_SEED),
//   //       organizationPubkey.toBuffer(),
//   //       Buffer.from(lionSpeciesId),
//   //     ],
//   //     PROGRAM_ID
//   //   );

//   //   const [orgVaultPda] = PublicKey.findProgramAddressSync(
//   //     [Buffer.from(ORG_VAULT_SEED), organizationPubkey.toBuffer()],
//   //     PROGRAM_ID
//   //   );

//   //   let tx = await program.methods
//   //     .createPool("Reserve", organizationPubkey, "Lion", lionSpeciesId)
//   //     .accounts({
//   //       admin: admin.publicKey,
//   //       pool: lionPoolPda,
//   //       poolVault: lionPoolVaultPda,
//   //       organizationVault: orgVaultPda,
//   //       systemProgram: SystemProgram.programId,
//   //     })
//   //     .transaction();

//   //   tx.recentBlockhash = svm.latestBlockhash();
//   //   tx.feePayer = admin.publicKey;
//   //   tx.sign(admin);

//   //   let result = svm.sendTransaction(tx);
//   //   expect(result.err).to.be.null;

//   //   // Create second pool (Elephant)
//   //   const elephantSpeciesId = "loxodonta_africana";
//   //   const [elephantPoolPda] = PublicKey.findProgramAddressSync(
//   //     [
//   //       Buffer.from(POOL_SEED),
//   //       organizationPubkey.toBuffer(),
//   //       Buffer.from(elephantSpeciesId),
//   //     ],
//   //     PROGRAM_ID
//   //   );

//   //   const [elephantPoolVaultPda] = PublicKey.findProgramAddressSync(
//   //     [
//   //       Buffer.from(POOL_VAULT_SEED),
//   //       organizationPubkey.toBuffer(),
//   //       Buffer.from(elephantSpeciesId),
//   //     ],
//   //     PROGRAM_ID
//   //   );

//   //   tx = await program.methods
//   //     .createPool("Reserve", organizationPubkey, "Elephant", elephantSpeciesId)
//   //     .accounts({
//   //       admin: admin.publicKey,
//   //       pool: elephantPoolPda,
//   //       poolVault: elephantPoolVaultPda,
//   //       organizationVault: orgVaultPda, // Same org vault
//   //       systemProgram: SystemProgram.programId,
//   //     })
//   //     .transaction();

//   //   tx.recentBlockhash = svm.latestBlockhash();
//   //   tx.feePayer = admin.publicKey;
//   //   tx.sign(admin);

//   //   result = svm.sendTransaction(tx);
//   //   expect(result.err).to.be.null;

//   //   // Verify both pools exist
//   //   const lionPool = await program.account.pool.fetch(lionPoolPda);
//   //   const elephantPool = await program.account.pool.fetch(elephantPoolPda);

//   //   expect(lionPool.speciesId).to.equal(lionSpeciesId);
//   //   expect(elephantPool.speciesId).to.equal(elephantSpeciesId);

//   //   // Both share same org vault
//   //   expect(lionPool.organizationPubkey.toString()).to.equal(
//   //     elephantPool.organizationPubkey.toString()
//   //   );
//   // });
// });
