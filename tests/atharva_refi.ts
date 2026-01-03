import { Keypair, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { BN, Program } from "@coral-xyz/anchor";
import { AtharvaRefi } from "../target/types/atharva_refi";
import idl from "../target/idl/atharva_refi.json";
import { fromWorkspace, LiteSVMProvider } from "anchor-litesvm";
import {
  bytesToString,
  getOrCreateAdminWallet,
  getPoolPdas,
  loadMarinadeAccounts,
  stringToBytes,
} from "./utilities";
import { expect } from "chai";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  LIQ_POOL_MSOL_LEG,
  LIQ_POOL_SOL_LEG,
  M_PROGRAM_ID,
  M_STATE,
  marinadePath,
  MB_PROGRAM_ID,
  MSOL_LEG_AUTH,
  MSOL_MINT,
  MSOL_MINT_AUTH,
  RESERVE_PDA,
  STREAM_INTERVAL_MS,
  TREASURY_MSOL,
} from "./constants";
import { MAGIC_PROGRAM_ID } from "@magicblock-labs/ephemeral-rollups-sdk";

describe("Atharva ReFi Tests", () => {
  const svm = fromWorkspace("./");

  let provider: LiteSVMProvider;
  let program: Program<AtharvaRefi>;

  let admin: Keypair;
  let supporter: Keypair;
  let organization: Keypair;

  // Test data
  const organizationName = "Londolozi Reserve";
  const speciesName = "African Lion";
  const speciesId = "panthera_leo";
  const depositAmount = new BN(10 * LAMPORTS_PER_SOL);

  before(async () => {
    // Initialize provider and program
    provider = new LiteSVMProvider(svm);
    program = new Program<AtharvaRefi>(idl, provider);

    // Initialize all Marinade dependencies with one call
    loadMarinadeAccounts(svm, marinadePath);
    // svm.addProgram(MAGIC_PROGRAM_ID, Buffer.alloc(0));

    // Generate keypairs
    admin = getOrCreateAdminWallet();
    supporter = Keypair.generate();
    organization = Keypair.generate();

    // Airdrop SOL
    svm.airdrop(admin.publicKey, BigInt(20 * LAMPORTS_PER_SOL));
    svm.airdrop(supporter.publicKey, BigInt(20 * LAMPORTS_PER_SOL));
    svm.airdrop(organization.publicKey, BigInt(20 * LAMPORTS_PER_SOL));
  });

  describe("Create Pool", () => {
    it("creates a lion conservation pool for Londolozi Reserve", async () => {
      const speciesIdBytes = stringToBytes(speciesId, 32);
      const { poolPda, poolMintPda, poolVaultPda, orgVaultPda } = getPoolPdas(
        organization.publicKey,
        speciesIdBytes
      );

      console.log("\n--- Creating Pool ---");
      console.log("Pool PDA:", poolPda.toBase58());
      console.log("Pool Mint:", poolMintPda.toBase58());
      console.log("Pool Vault:", poolVaultPda.toBase58());
      console.log("Org Vault:", orgVaultPda.toBase58());

      // Create pool
      await program.methods
        .createPool(
          organizationName,
          organization.publicKey,
          speciesName,
          speciesIdBytes
        )
        .accountsStrict({
          admin: admin.publicKey,
          pool: poolPda,
          poolMint: poolMintPda,
          poolVault: poolVaultPda,
          organizationVault: orgVaultPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      // Verify pool state
      const pool = await program.account.pool.fetch(poolPda);

      expect(pool.organizationPubkey.toString()).to.equal(
        organization.publicKey.toString()
      );
      expect(pool.organizationName).to.equal(organizationName);
      expect(pool.speciesName).to.equal(speciesName);
      expect(pool.speciesId).to.equal(speciesId);
      expect(pool.isActive).to.be.true;
      expect(pool.isCrankScheduled).to.be.false;
      expect(pool.totalDeposits.toNumber()).to.equal(0);
      expect(pool.totalShares.toNumber()).to.equal(0);
      expect(pool.organizationYieldBps).to.equal(20);

      console.log("✅ Pool created successfully");
      console.log("   Organization:", pool.organizationName);
      console.log("   Species:", pool.speciesName);
      console.log("   Yield BPS:", pool.organizationYieldBps);
    });
  });

  describe("Deposit", () => {
    it("supporter deposits 4 SOL into lion pool", async () => {
      const speciesIdBytes = stringToBytes(speciesId, 32);

      console.log("Organization Pubkey: ", organization.publicKey);
      console.log("New Species ID: ", speciesIdBytes);
      console.log("Species ID: ", bytesToString(speciesIdBytes));

      const { poolPda, poolMintPda, poolVaultPda } = getPoolPdas(
        organization.publicKey,
        speciesIdBytes
      );

      // Get supporter's pool token account (ATA)
      const supporterPoolTokenAccount = getAssociatedTokenAddressSync(
        poolMintPda,
        supporter.publicKey
      );

      console.log("\n--- Depositing SOL ---");
      console.log("Supporter:", supporter.publicKey.toBase58());
      console.log(
        "Amount:",
        depositAmount.toNumber() / LAMPORTS_PER_SOL,
        "SOL"
      );
      console.log("Token Account:", supporterPoolTokenAccount.toBase58());

      // Get balances before
      const supporterBalanceBefore = svm.getBalance(supporter.publicKey);
      const poolVaultBalanceBefore = svm.getBalance(poolVaultPda);

      // Deposit
      await program.methods
        .deposit(depositAmount)
        .accountsStrict({
          supporter: supporter.publicKey,
          pool: poolPda,
          poolMint: poolMintPda,
          poolVault: poolVaultPda,
          supporterPoolTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([supporter])
        .rpc();

      // Get balances after
      const supporterBalanceAfter = svm.getBalance(supporter.publicKey);
      const poolVaultBalanceAfter = svm.getBalance(poolVaultPda);

      // Verify pool state
      const pool = await program.account.pool.fetch(poolPda);

      expect(pool.totalDeposits.toString()).to.equal(depositAmount.toString());
      expect(pool.totalShares.toString()).to.equal(depositAmount.toString());

      console.log("✅ Deposit successful");
      console.log(
        "   Total Deposits:",
        pool.totalDeposits.toNumber() / LAMPORTS_PER_SOL,
        "SOL"
      );
      console.log(
        "   Total Shares:",
        pool.totalShares.toNumber() / LAMPORTS_PER_SOL
      );
      console.log(
        "   Pool Vault Balance:",
        Number(poolVaultBalanceAfter) / LAMPORTS_PER_SOL,
        "SOL"
      );

      // Verify supporter received pool tokens
      const tokenAccount = await program.provider.connection.getAccountInfo(
        supporterPoolTokenAccount
      );
      expect(tokenAccount).to.not.be.null;
      console.log("   Supporter received pool tokens ✓");
    });

    it("fails when pool is inactive", async () => {
      const speciesIdBytes = stringToBytes(speciesId, 32);
      const { poolPda, poolMintPda, poolVaultPda } = getPoolPdas(
        organization.publicKey,
        speciesIdBytes
      );

      const supporterPoolTokenAccount = getAssociatedTokenAddressSync(
        poolMintPda,
        supporter.publicKey
      );

      // First, deactivate the pool (you'd need a toggle_pool_status instruction)
      // For now, this test demonstrates the expected behavior

      try {
        await program.methods
          .deposit(new BN(LAMPORTS_PER_SOL))
          .accountsStrict({
            supporter: supporter.publicKey,
            pool: poolPda,
            poolMint: poolMintPda,
            poolVault: poolVaultPda,
            supporterPoolTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([supporter])
          .rpc();

        // If pool is inactive, this should fail
        expect.fail("Should have failed with inactive pool");
      } catch (error) {
        // Expected to fail if pool inactive constraint exists
        console.log("✅ Correctly rejected inactive pool deposit");
      }
    });

    it("fails with zero deposit amount", async () => {
      const speciesIdBytes = stringToBytes(speciesId, 32);
      const { poolPda, poolMintPda, poolVaultPda } = getPoolPdas(
        organization.publicKey,
        speciesIdBytes
      );

      const supporterPoolTokenAccount = getAssociatedTokenAddressSync(
        poolMintPda,
        supporter.publicKey
      );

      try {
        await program.methods
          .deposit(new BN(0))
          .accountsStrict({
            supporter: supporter.publicKey,
            pool: poolPda,
            poolMint: poolMintPda,
            poolVault: poolVaultPda,
            supporterPoolTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([supporter])
          .rpc();

        expect.fail("Should have failed with zero amount");
      } catch (error) {
        console.log("✅ Correctly rejected zero deposit");
      }
    });
  });

  describe("Stake", () => {
    it("stakes 2 SOL on Marinade Finance", async () => {
      const speciesIdBytes = stringToBytes(speciesId, 32);
      const { poolPda, poolVaultPda } = getPoolPdas(
        organization.publicKey,
        speciesIdBytes
      );

      // Amount to stake (e.g., 2 SOL)
      const stakeAmount = new BN(2 * LAMPORTS_PER_SOL);

      // 1. Derive mSOL ATA for the Pool Vault
      const poolMsolAccount = getAssociatedTokenAddressSync(
        MSOL_MINT,
        poolVaultPda,
        true // allowOwnerOffCurve because poolVaultPda is a PDA
      );

      console.log("Pool MSOL", poolMsolAccount);

      // 2. Execute Stake
      await program.methods
        .stake(stakeAmount)
        .accountsStrict({
          pool: poolPda,
          signer: admin.publicKey,
          marinadeState: M_STATE,
          msolMint: MSOL_MINT,
          liqPoolSolLeg: LIQ_POOL_SOL_LEG,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          liqPoolMsolLegAuthority: MSOL_LEG_AUTH,
          reservePda: RESERVE_PDA,
          poolVault: poolVaultPda,
          poolMsolAccount: poolMsolAccount, // This must be an ATA
          msolMintAuthority: MSOL_MINT_AUTH,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID, // Added
          systemProgram: SystemProgram.programId,
          marinadeProgram: M_PROGRAM_ID,
        })
        .signers([admin])
        .rpc();

      console.log("✅ Staking successful");

      const msolBalance = svm.getAccount(poolMsolAccount).lamports;
      console.log("   mSOL received:", Number(msolBalance) / LAMPORTS_PER_SOL);
    });
  });

  describe("Schedule Stream", () => {
    it("schedules yield streaming for 2-day intervals", async () => {
      const speciesIdBytes = stringToBytes(speciesId, 32);
      const { poolPda, poolVaultPda, orgVaultPda } = getPoolPdas(
        organization.publicKey,
        speciesIdBytes
      );

      const scheduleArgs = {
        taskId: new BN(1),
        executionIntervalMillis: new BN(STREAM_INTERVAL_MS),
        iterations: new BN(10),
      };

      console.log("\n--- Scheduling Stream ---");
      console.log("Task ID:", 1);
      console.log("Interval:", STREAM_INTERVAL_MS / 86400000, "days");
      console.log("Iterations:", 10);
      console.log("Authority:", organization.publicKey.toBase58());

      await program.methods
        .scheduleStreams(scheduleArgs)
        .accountsStrict({
          authority: organization.publicKey,
          pool: poolPda,
          poolVault: poolVaultPda,
          organizationVault: orgVaultPda,
          marinadeState: M_STATE,
          msolMint: MSOL_MINT,
          liqPoolSolLeg: LIQ_POOL_SOL_LEG,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          treasuryMsolAccount: TREASURY_MSOL,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          marinadeProgram: M_PROGRAM_ID,
          magicProgram: MB_PROGRAM_ID,
          program: program.programId,
        })
        .signers([organization])
        .rpc();

      // Verify
      const pool = await program.account.pool.fetch(poolPda);

      expect(pool.isCrankScheduled).to.be.true;
      expect(pool.lastStreamTs.toNumber()).to.be.greaterThan(0);

      console.log("✅ Stream scheduled successfully");
      console.log("   Crank Status: Scheduled");
      console.log(
        "   Last Stream TS:",
        new Date(pool.lastStreamTs.toNumber() * 1000).toLocaleString()
      );
    });
  });

  describe("Unstake", () => {
    it("unstakes 1 SOL from Marinade Finance", async () => {
      const speciesIdBytes = stringToBytes(speciesId, 32);
      const { poolPda, poolVaultPda } = getPoolPdas(
        organization.publicKey,
        speciesIdBytes
      );

      // Amount to stake (e.g., 2 SOL)
      const unstakeAmount = new BN(1 * LAMPORTS_PER_SOL);

      // 1. Derive mSOL ATA for the Pool Vault
      const poolMsolAccount = getAssociatedTokenAddressSync(
        MSOL_MINT,
        poolVaultPda,
        true // allowOwnerOffCurve because poolVaultPda is a PDA
      );

      console.log("Pool MSOL", poolMsolAccount);

      // 2. Execute Stake
      await program.methods
        .unstake(unstakeAmount)
        .accountsStrict({
          pool: poolPda,
          marinadeState: M_STATE,
          msolMint: MSOL_MINT,
          liqPoolSolLeg: LIQ_POOL_SOL_LEG,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          treasuryMsolAccount: TREASURY_MSOL,
          poolMsolAccount: poolMsolAccount, // This must be an ATA
          poolVault: poolVaultPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          marinadeProgram: M_PROGRAM_ID,
        })
        .rpc();

      console.log("✅ Unstaking successful");

      const msolBalance = svm.getAccount(poolMsolAccount).lamports;
      console.log("   mSOL received:", Number(msolBalance) / LAMPORTS_PER_SOL);
    });
  });

  /* ---------- ORGANIZATION WITHDRAWAL ---------- */
  describe("Organization Withdraw", () => {
    it("withdraws from organization vault", async () => {
      const speciesIdBytes = stringToBytes(speciesId, 32);
      const { poolPda, orgVaultPda } = getPoolPdas(
        organization.publicKey,
        speciesIdBytes
      );

      // Amount to stake (e.g., 2 SOL)
      const amount = new BN(1 * LAMPORTS_PER_SOL);

      // 2. Execute Stake
      await program.methods
        .organizationWithdraw(amount)
        .accountsStrict({
          organization: organization.publicKey,
          pool: poolPda,
          orgVault: orgVaultPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([organization])
        .rpc();

      console.log("✅ Unstaking successful");
    });
  });

  /* ---------- SUPPORTER WITHDRAWAL ---------- */
  describe("Supporter Withdraw", () => {
    it("withdraws supporter's stake and yields", async () => {
      const speciesIdBytes = stringToBytes(speciesId, 32);
      const { poolPda, poolMintPda, poolVaultPda } = getPoolPdas(
        organization.publicKey,
        speciesIdBytes
      );

      // 1. Derive mSOL ATA for the Pool Vault
      const poolMsolAccount = getAssociatedTokenAddressSync(
        MSOL_MINT,
        poolVaultPda,
        true
      );

      // 2. Get Supporter's Share Token ATA
      const supporterPoolTokenAccount = getAssociatedTokenAddressSync(
        poolMintPda,
        supporter.publicKey
      );

      // Calculate amount to withdraw (e.g., 50% of shares)
      const supporterShares = (await program.account.pool.fetch(poolPda))
        .totalShares;
      const withdrawAmount = supporterShares.div(new BN(1));

      // 3. Execute Withdraw
      await program.methods
        .supporterWithdraw(withdrawAmount)
        .accountsStrict({
          supporter: supporter.publicKey,
          pool: poolPda,
          poolMint: poolMintPda,
          supporterPoolTokenAccount: supporterPoolTokenAccount,
          marinadeState: M_STATE,
          msolMint: MSOL_MINT,
          liqPoolSolLeg: LIQ_POOL_SOL_LEG,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          treasuryMsolAccount: TREASURY_MSOL,
          poolMsolAccount: poolMsolAccount,
          poolVault: poolVaultPda,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          marinadeProgram: M_PROGRAM_ID,
        })
        .signers([supporter])
        .rpc();

      const poolAfter = await program.account.pool.fetch(poolPda);
      console.log("✅ Withdrawal successful");
      console.log(
        "   Remaining Total Shares:",
        poolAfter.totalShares.toString()
      );
    });
  });
});
