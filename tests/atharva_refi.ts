import * as anchor from "@coral-xyz/anchor";
import { AnchorError, BN, Program } from "@coral-xyz/anchor";
import { AtharvaRefi } from "../target/types/atharva_refi";
import { Keypair, LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";
import {
  fetchBalance,
  fetchTokenBalance,
  fundAccount,
  getOrCreateAdminWallet,
  getPoolPdas,
  lamportsToSol,
  logData,
  logDone,
  logSignature,
  stringToBytes,
} from "./utilities";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  LIQ_POOL_MSOL_LEG,
  LIQ_POOL_SOL_LEG,
  MAR_PROGRAM_ID,
  MAR_STATE,
  MSOL_LEG_AUTH,
  MSOL_MINT,
  MSOL_MINT_AUTH,
  RESERVE_PDA,
  STREAM_TEST_INTERVAL_MS,
  TREASURY_MSOL,
} from "./constants";
import { MAGIC_PROGRAM_ID } from "@magicblock-labs/ephemeral-rollups-sdk";
import { expect } from "chai";

describe("atharva refi protocol", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AtharvaRefi as Program<AtharvaRefi>;

  const providerER = new anchor.AnchorProvider(
    new anchor.web3.Connection(
      process.env.EPHEMERAL_PROVIDER_ENDPOINT ||
        "https://devnet-as.magicblock.app/",
      {
        wsEndpoint:
          process.env.EPHEMERAL_WS_ENDPOINT ||
          "wss://devnet-as.magicblock.app/",
      }
    ),
    anchor.Wallet.local()
  );

  let admin: Keypair;
  let supporter: Keypair;
  let supporter2: Keypair;
  let organization: Keypair;
  let supporterPoolTokenAccount: anchor.web3.PublicKey;
  let supporter2PoolTokenAccount: anchor.web3.PublicKey;
  let poolMsolAccount: anchor.web3.PublicKey;
  let poolPda: anchor.web3.PublicKey;
  let poolMintPda: anchor.web3.PublicKey;
  let poolVaultPda: anchor.web3.PublicKey;
  let orgVaultPda: anchor.web3.PublicKey;

  const ORGANIZATION_NAME = "Londolozi Reserve";
  const SPECIES_NAME = "African Lion";
  const SPECIES_ID = "panthera_leo";
  const SPECIES_ID_BYTES = stringToBytes(SPECIES_ID, 32);

  const DEPOSIT_AMOUNT = 0.1;
  const STAKE_AMOUNT = 0.05;
  const UNSTAKE_AMOUNT = 0.02;
  const ORG_WITHDRAW_AMOUNT = 0.001;
  const SUPPORTER_WITHDRAW_AMOUNT = 0.01;

  const SCHEDULE_ITERATIONS = 2;
  const DELEGATION_WAIT_MS = 5000;
  const STREAM_WAIT_MS = STREAM_TEST_INTERVAL_MS * SCHEDULE_ITERATIONS + 5000;
  const UNDELEGATION_WAIT_MS = 5000;

  before(async () => {
    admin = await getOrCreateAdminWallet();
    supporter = Keypair.generate();
    supporter2 = Keypair.generate();
    organization = Keypair.generate();

    const payer = provider.wallet.payer;
    await fundAccount(provider.connection, payer, admin.publicKey, 0.05);
    await fundAccount(provider.connection, payer, supporter.publicKey, 0.5);
    await fundAccount(provider.connection, payer, supporter2.publicKey, 0.5);
    await fundAccount(provider.connection, payer, organization.publicKey, 0.01);

    const pdas = getPoolPdas(organization.publicKey, SPECIES_ID_BYTES);
    poolPda = pdas.poolPda;
    poolMintPda = pdas.poolMintPda;
    poolVaultPda = pdas.poolVaultPda;
    orgVaultPda = pdas.orgVaultPda;

    poolMsolAccount = getAssociatedTokenAddressSync(
      MSOL_MINT,
      poolVaultPda,
      true
    );

    supporterPoolTokenAccount = getAssociatedTokenAddressSync(
      poolMintPda,
      supporter.publicKey
    );

    supporter2PoolTokenAccount = getAssociatedTokenAddressSync(
      poolMintPda,
      supporter2.publicKey
    );
  });

  /* CREATE POOL */
  describe("create pool", () => {
    it("should create a lion conservation pool", async () => {
      const txn = await program.methods
        .createPool(
          ORGANIZATION_NAME,
          organization.publicKey,
          SPECIES_NAME,
          SPECIES_ID_BYTES
        )
        .accountsStrict({
          admin: admin.publicKey,
          msolMint: MSOL_MINT,
          pool: poolPda,
          poolMint: poolMintPda,
          poolVault: poolVaultPda,
          organizationVault: orgVaultPda,
          poolMsolAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .transaction();

      const signature = await provider.sendAndConfirm(txn, [admin]);
      logSignature("Create Pool", signature);

      const pool = await program.account.pool.fetch(poolPda);
      expect(pool.organizationPubkey).to.eql(organization.publicKey);
      expect(pool.speciesName).to.eql(SPECIES_NAME);
      expect(pool.speciesId).to.eql(SPECIES_ID);
      expect(pool.isActive).to.be.true;
      expect(pool.totalDeposits.toNumber()).to.eql(0);

      logData(`Organization: ${pool.organizationName}`);
      logData(`Species Name: ${pool.speciesName}`);
      logData(`Species ID: ${pool.speciesId}`);

      logDone(`${pool.organizationName} ${pool.speciesName} Pool Created!`);
    });

    it("should initialize pool with correct default values", async () => {
      const pool = await program.account.pool.fetch(poolPda);

      expect(pool.isActive).to.be.true;
      expect(pool.isCrankScheduled).to.be.false;
      expect(pool.totalDeposits.toNumber()).to.equal(0);
      expect(pool.totalShares.toNumber()).to.equal(0);
      expect(pool.organizationYieldBps).to.equal(20);
      expect(pool.lastStreamedVaultSol.toNumber()).to.equal(0);
      expect(pool.lastStreamTs.toNumber()).to.equal(0);

      logDone("Pool initialized with correct default values!");
    });

    it("should fail when non-admin tries to create pool", async () => {
      const newOrg = Keypair.generate();
      const newSpeciesBytes = stringToBytes("test_species", 32);
      const newPdas = getPoolPdas(newOrg.publicKey, newSpeciesBytes);

      try {
        await program.methods
          .createPool(
            "Unauthorized Pool",
            newOrg.publicKey,
            "Test Species",
            newSpeciesBytes
          )
          .accountsStrict({
            admin: supporter.publicKey,
            msolMint: MSOL_MINT,
            pool: newPdas.poolPda,
            poolMint: newPdas.poolMintPda,
            poolVault: newPdas.poolVaultPda,
            organizationVault: newPdas.orgVaultPda,
            poolMsolAccount: getAssociatedTokenAddressSync(
              MSOL_MINT,
              newPdas.poolVaultPda,
              true
            ),
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([supporter])
          .rpc();

        expect.fail("Transaction should have failed");
      } catch (err) {
        const anchorErr = err as AnchorError;
        expect(anchorErr.error.errorCode.code).to.equal(
          "CreatePoolUnauthorized"
        );
        logDone("Non-admin correctly prevented from creating pool!");
      }
    });
  });

  /* DEPOSIT */
  describe("deposit", () => {
    it("should deposit SOL into the pool", async () => {
      const poolBefore = await program.account.pool.fetch(poolPda);

      const txn = await program.methods
        .deposit(new BN(DEPOSIT_AMOUNT * LAMPORTS_PER_SOL))
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
        .transaction();

      const signature = await provider.sendAndConfirm(txn, [supporter]);
      logSignature("Deposit", signature);

      const supporterBalance = await fetchBalance(
        provider,
        supporter.publicKey
      );
      const poolVaultBalance = await fetchBalance(provider, poolVaultPda);
      const supporterPoolTokenBal = await fetchTokenBalance(
        provider,
        supporterPoolTokenAccount
      );

      const poolAfter = await program.account.pool.fetch(poolPda);
      expect(poolAfter.totalDeposits.toNumber()).to.above(
        poolBefore.totalDeposits.toNumber()
      );
      expect(poolAfter.totalShares.toNumber()).to.above(
        poolBefore.totalShares.toNumber()
      );

      logData(`Supporter Balance: ${supporterBalance} SOL`);
      logData(`Pool Vault Balance: ${poolVaultBalance} SOL`);
      logData(`Supporter Token Balance: ${supporterPoolTokenBal} ARFI`);
      logData(
        `Pool Total Deposits: ${lamportsToSol(
          poolAfter.totalDeposits.toNumber()
        )} SOL`
      );
      logData(
        `Pool Total Shares: ${lamportsToSol(
          poolAfter.totalShares.toNumber()
        )} ARFI`
      );

      logDone(`Supporter deposited ${DEPOSIT_AMOUNT} SOL into pool!`);
    });

    it("should handle minimum deposit amount", async () => {
      const minDeposit = 0.001;

      const txn = await program.methods
        .deposit(new BN(minDeposit * LAMPORTS_PER_SOL))
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
        .transaction();

      const signature = await provider.sendAndConfirm(txn, [supporter]);
      expect(signature).to.exist;

      logDone(`Minimum deposit of ${minDeposit} SOL successful!`);
    });

    it("should allow multiple supporters to deposit", async () => {
      const deposit2Amount = 0.05;

      const poolBefore = await program.account.pool.fetch(poolPda);

      const txn = await program.methods
        .deposit(new BN(deposit2Amount * LAMPORTS_PER_SOL))
        .accountsStrict({
          supporter: supporter2.publicKey,
          pool: poolPda,
          poolMint: poolMintPda,
          poolVault: poolVaultPda,
          supporterPoolTokenAccount: supporter2PoolTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .transaction();

      await provider.sendAndConfirm(txn, [supporter2]);

      const poolAfter = await program.account.pool.fetch(poolPda);
      const totalDeposited =
        lamportsToSol(poolAfter.totalDeposits.toNumber()) -
        lamportsToSol(poolBefore.totalDeposits.toNumber());

      expect(totalDeposited).to.be.closeTo(deposit2Amount, 0.001);

      logDone(`Second supporter deposited ${deposit2Amount} SOL!`);
    });

    it("should maintain accurate total deposits after multiple operations", async () => {
      const poolBefore = await program.account.pool.fetch(poolPda);
      const additionalDeposit = 0.02;

      await program.methods
        .deposit(new BN(additionalDeposit * LAMPORTS_PER_SOL))
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

      const poolAfter = await program.account.pool.fetch(poolPda);
      const expectedDeposits =
        poolBefore.totalDeposits.toNumber() +
        additionalDeposit * LAMPORTS_PER_SOL;

      expect(poolAfter.totalDeposits.toNumber()).to.equal(expectedDeposits);

      logDone("Total deposits tracked accurately!");
    });

    it("should fail when depositing 0 SOL", async () => {
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

        expect.fail("Transaction should have failed");
      } catch (err) {
        const anchorErr = err as AnchorError;
        expect(anchorErr.error.errorCode.code).to.equal("InvalidAmount");
        logDone("Zero deposit correctly rejected!");
      }
    });

    it("should maintain correct share ratios", async () => {
      const pool = await program.account.pool.fetch(poolPda);
      const supporterShares = await fetchTokenBalance(
        provider,
        supporterPoolTokenAccount
      );

      expect(supporterShares).to.be.at.most(
        lamportsToSol(pool.totalShares.toNumber())
      );
    });
  });

  /* STAKE */
  describe("stake", () => {
    it("should stake SOL on Marinade and receives mSOL", async () => {
      const txn = await program.methods
        .stake(new BN(STAKE_AMOUNT * LAMPORTS_PER_SOL))
        .accountsStrict({
          pool: poolPda,
          marinadeState: MAR_STATE,
          msolMint: MSOL_MINT,
          liqPoolSolLeg: LIQ_POOL_SOL_LEG,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          liqPoolMsolLegAuthority: MSOL_LEG_AUTH,
          reservePda: RESERVE_PDA,
          poolVault: poolVaultPda,
          poolMsolAccount,
          msolMintAuthority: MSOL_MINT_AUTH,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          marinadeProgram: MAR_PROGRAM_ID,
        })
        .transaction();

      const signature = await provider.sendAndConfirm(txn, []);
      logSignature("Stake", signature);

      const poolVaultBalance = await fetchBalance(provider, poolVaultPda);
      const poolMsolBalance = await fetchTokenBalance(
        provider,
        poolMsolAccount
      );

      expect(poolMsolBalance).to.be.greaterThan(0);

      logData(`Pool Vault Balance: ${poolVaultBalance} SOL`);
      logData(`Pool mSOL Balance: ${poolMsolBalance.toFixed(2)} mSOL`);

      logDone(
        `Pool staked ${STAKE_AMOUNT} SOL and received ${poolMsolBalance.toFixed(
          2
        )} mSOL!`
      );
    });

    it("should handle multiple staking operations", async () => {
      const additionalStake = 0.02;
      const msolBalanceBefore = await fetchTokenBalance(
        provider,
        poolMsolAccount
      );

      await program.methods
        .stake(new BN(additionalStake * LAMPORTS_PER_SOL))
        .accountsStrict({
          pool: poolPda,
          marinadeState: MAR_STATE,
          msolMint: MSOL_MINT,
          liqPoolSolLeg: LIQ_POOL_SOL_LEG,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          liqPoolMsolLegAuthority: MSOL_LEG_AUTH,
          reservePda: RESERVE_PDA,
          poolVault: poolVaultPda,
          poolMsolAccount,
          msolMintAuthority: MSOL_MINT_AUTH,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          marinadeProgram: MAR_PROGRAM_ID,
        })
        .rpc();

      const msolBalanceAfter = await fetchTokenBalance(
        provider,
        poolMsolAccount
      );

      expect(msolBalanceAfter).to.be.greaterThan(msolBalanceBefore);

      logDone(
        `Additional ${additionalStake} SOL staked, mSOL balance increased!`
      );
    });
  });

  /* STREAM */
  describe("stream", () => {
    it("should stream yield to organization vault", async () => {
      const orgBalanceBefore = await fetchBalance(provider, orgVaultPda);

      const txn = await program.methods
        .stream()
        .accountsStrict({
          pool: poolPda,
          organizationVault: orgVaultPda,
          marinadeState: MAR_STATE,
          msolMint: MSOL_MINT,
          liqPoolSolLeg: LIQ_POOL_SOL_LEG,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          treasuryMsolAccount: TREASURY_MSOL,
          poolMsolAccount,
          poolVault: poolVaultPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          marinadeProgram: MAR_PROGRAM_ID,
        })
        .transaction();

      const signature = await provider.sendAndConfirm(txn, []);
      logSignature("Stream", signature);

      const orgBalanceAfter = await fetchBalance(provider, orgVaultPda);
      const streamed = orgBalanceAfter - orgBalanceBefore;

      expect(streamed).to.be.greaterThan(0);

      logData(
        `Organization Vault Balance: ${lamportsToSol(orgBalanceAfter)} SOL`
      );
      logData(`Streamed Amount: ${streamed.toFixed(4)} SOL`);

      logDone(`Streamed ${streamed.toFixed(4)} SOL to organization vault!`);
    });
  });

  /* DELEGATE */
  describe("delegate", () => {
    it("delegates pool to ephemeral rollups", async () => {
      const txn = await program.methods
        .delegate()
        .accountsPartial({
          payer: admin.publicKey,
          pool: poolPda,
        })
        .transaction();

      const signature = await provider.sendAndConfirm(txn, [admin]);
      logSignature("Delegate", signature);

      await new Promise((resolve) => setTimeout(resolve, DELEGATION_WAIT_MS));

      logDone("Pool delegated to ephemeral rollups!");
    });
  });

  /* SCHEDULE STREAMS */
  describe("schedule streams", () => {
    it("schedules automatic yield streaming", async () => {
      const orgBalanceBefore = await provider.connection.getBalance(
        orgVaultPda
      );

      const txn = await program.methods
        .scheduleStreams({
          taskId: new BN(1),
          executionIntervalMillis: new BN(STREAM_TEST_INTERVAL_MS),
          iterations: new BN(SCHEDULE_ITERATIONS),
        })
        .accountsStrict({
          authority: admin.publicKey,
          pool: poolPda,
          poolVault: poolVaultPda,
          organizationVault: orgVaultPda,
          marinadeState: MAR_STATE,
          msolMint: MSOL_MINT,
          liqPoolSolLeg: LIQ_POOL_SOL_LEG,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          treasuryMsolAccount: TREASURY_MSOL,
          tokenProgram: TOKEN_PROGRAM_ID,
          marinadeProgram: MAR_PROGRAM_ID,
          magicProgram: MAGIC_PROGRAM_ID,
          program: program.programId,
          systemProgram: SystemProgram.programId,
        })
        .transaction();

      txn.feePayer = admin.publicKey;
      txn.recentBlockhash = (
        await providerER.connection.getLatestBlockhash()
      ).blockhash;
      txn.sign(admin);

      const signature = await providerER.connection.sendRawTransaction(
        txn.serialize(),
        { skipPreflight: true }
      );
      await providerER.connection.confirmTransaction(signature);
      logSignature("Schedule Streams", signature);

      await new Promise((resolve) => setTimeout(resolve, STREAM_WAIT_MS));

      const orgBalanceAfterBase = await provider.connection.getBalance(
        orgVaultPda
      );
      const orgBalanceAfterER = await providerER.connection.getBalance(
        orgVaultPda
      );
      const streamedBase = orgBalanceAfterBase - orgBalanceBefore;
      const streamedER = orgBalanceAfterER - orgBalanceBefore;

      logData(`Streamed (Base Layer): ${lamportsToSol(streamedBase)} SOL`);
      logData(`Streamed (ER): ${lamportsToSol(streamedER)} SOL`);
      logData(
        `Organization Vault Balance (Base): ${lamportsToSol(
          orgBalanceAfterBase
        )} SOL`
      );
      logData(
        `Organization Vault Balance (ER): ${lamportsToSol(
          orgBalanceAfterER
        )} SOL`
      );

      logDone(
        `Scheduled ${SCHEDULE_ITERATIONS} streaming iterations on ephemeral rollup!`
      );
    });
  });

  /* UNDELEGATE */
  describe("undelegate", () => {
    it("undelegates pool from ephemeral rollups", async () => {
      const txn = await program.methods
        .undelegate()
        .accounts({
          payer: admin.publicKey,
          pool: poolPda,
        })
        .transaction();

      txn.feePayer = admin.publicKey;
      txn.recentBlockhash = (
        await providerER.connection.getLatestBlockhash()
      ).blockhash;
      txn.sign(admin);

      const signature = await providerER.connection.sendRawTransaction(
        txn.serialize(),
        { skipPreflight: true }
      );
      await providerER.connection.confirmTransaction(signature);
      logSignature("Undelegate", signature);

      await new Promise((resolve) => setTimeout(resolve, UNDELEGATION_WAIT_MS));

      const pool = await program.account.pool.fetch(poolPda);
      expect(pool).to.exist;

      logDone("Pool undelegated from ephemeral rollups!");
    });
  });

  /* UNSTAKE */
  describe("unstake", () => {
    it("unstakes mSOL on Marinade and receives SOL", async () => {
      const poolVaultBalanceBefore = await fetchBalance(provider, poolVaultPda);
      const poolMsolBalanceBefore = await fetchTokenBalance(
        provider,
        poolMsolAccount
      );

      const txn = await program.methods
        .unstake(new BN(UNSTAKE_AMOUNT * LAMPORTS_PER_SOL))
        .accountsStrict({
          pool: poolPda,
          marinadeState: MAR_STATE,
          msolMint: MSOL_MINT,
          liqPoolSolLeg: LIQ_POOL_SOL_LEG,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          treasuryMsolAccount: TREASURY_MSOL,
          poolVault: poolVaultPda,
          poolMsolAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          marinadeProgram: MAR_PROGRAM_ID,
        })
        .transaction();

      const signature = await provider.sendAndConfirm(txn, []);
      logSignature("Unstake", signature);

      const poolVaultBalanceAfter = await fetchBalance(provider, poolVaultPda);
      const poolMsolBalanceAfter = await fetchTokenBalance(
        provider,
        poolMsolAccount
      );

      logData(`Pool Vault Balance: ${poolVaultBalanceAfter} SOL`);
      logData(`Pool mSOL Balance: ${poolMsolBalanceAfter.toFixed(2)} mSOL`);
      logData(
        `SOL Received: ${(
          poolVaultBalanceAfter - poolVaultBalanceBefore
        ).toFixed(4)} SOL`
      );
      logData(
        `mSOL Burned: ${(poolMsolBalanceBefore - poolMsolBalanceAfter).toFixed(
          2
        )} mSOL`
      );

      logDone(
        `Unstaked ${UNSTAKE_AMOUNT} mSOL and received ${(
          poolVaultBalanceAfter - poolVaultBalanceBefore
        ).toFixed(4)} SOL!`
      );
    });

    it("should handle multiple unstaking operations", async () => {
      const additionalUnstake = 0.01;
      const vaultBalanceBefore = await fetchBalance(provider, poolVaultPda);

      await program.methods
        .unstake(new BN(additionalUnstake * LAMPORTS_PER_SOL))
        .accountsStrict({
          pool: poolPda,
          marinadeState: MAR_STATE,
          msolMint: MSOL_MINT,
          liqPoolSolLeg: LIQ_POOL_SOL_LEG,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          treasuryMsolAccount: TREASURY_MSOL,
          poolVault: poolVaultPda,
          poolMsolAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          marinadeProgram: MAR_PROGRAM_ID,
        })
        .rpc();

      const vaultBalance = await fetchBalance(provider, poolVaultPda);

      expect(vaultBalance).to.be.greaterThan(vaultBalanceBefore);

      logDone(
        `Additional ${additionalUnstake} SOL unstaked, mSOL balance decreased!`
      );
    });
  });

  /* ORGANIZATION WITHDRAW */
  describe("organization withdraw", () => {
    it("withdraws organization yields", async () => {
      const orgBalanceBefore = await fetchBalance(provider, orgVaultPda);

      const txn = await program.methods
        .organizationWithdraw(new BN(ORG_WITHDRAW_AMOUNT * LAMPORTS_PER_SOL))
        .accountsStrict({
          organization: organization.publicKey,
          pool: poolPda,
          orgVault: orgVaultPda,
          systemProgram: SystemProgram.programId,
        })
        .transaction();

      const signature = await provider.sendAndConfirm(txn, [organization]);
      logSignature("Organization Withdraw", signature);

      const orgBalanceAfter = await fetchBalance(provider, orgVaultPda);

      logData(`Organization Vault Balance: ${orgBalanceAfter} SOL`);
      logData(`Withdrawn Amount: ${ORG_WITHDRAW_AMOUNT} SOL`);

      logDone(`Organization withdrew ${ORG_WITHDRAW_AMOUNT} SOL from yields!`);
    });

    it("should fail when withdrawing more than balance", async () => {
      try {
        await program.methods
          .organizationWithdraw(new BN(1000 * LAMPORTS_PER_SOL))
          .accountsStrict({
            organization: organization.publicKey,
            pool: poolPda,
            orgVault: orgVaultPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([organization])
          .rpc();

        expect.fail("Transaction should have failed");
      } catch (err) {
        const anchorErr = err as AnchorError;
        expect(anchorErr.error.errorCode.code).to.equal(
          "InsufficientWithdrawFunds"
        );
        expect(anchorErr.error.errorMessage).to.equal(
          "Not enough SOL to withdraw"
        );
      }
    });

    it("should fail when non-organization tries to withdraw", async () => {
      try {
        await program.methods
          .organizationWithdraw(new BN(0.001 * LAMPORTS_PER_SOL))
          .accountsStrict({
            organization: supporter.publicKey, // wrong authority
            pool: poolPda,
            orgVault: orgVaultPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([supporter])
          .rpc();

        expect.fail("Transaction should have failed");
      } catch (err) {
        const anchorErr = err as AnchorError;

        expect(anchorErr.error.errorCode.code).to.equal(
          "UnauthorizedOrganization"
        );
        expect(anchorErr.error.errorMessage).to.equal(
          "Only the organization can withdraw"
        );
      }
    });
  });

  /* SUPPORTER WITHDRAW */
  describe("supporter withdraw", () => {
    it("withdraws supporter stake and yields", async () => {
      const supporterBalanceBefore = await fetchBalance(
        provider,
        supporter.publicKey
      );
      const poolVaultBalanceBefore = await fetchBalance(provider, poolVaultPda);

      const txn = await program.methods
        .supporterWithdraw(new BN(SUPPORTER_WITHDRAW_AMOUNT * LAMPORTS_PER_SOL))
        .accountsStrict({
          supporter: supporter.publicKey,
          pool: poolPda,
          poolMint: poolMintPda,
          supporterPoolTokenAccount,
          marinadeState: MAR_STATE,
          msolMint: MSOL_MINT,
          liqPoolSolLeg: LIQ_POOL_SOL_LEG,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          treasuryMsolAccount: TREASURY_MSOL,
          poolMsolAccount,
          poolVault: poolVaultPda,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          marinadeProgram: MAR_PROGRAM_ID,
        })
        .transaction();

      const signature = await provider.sendAndConfirm(txn, [supporter]);
      logSignature("Supporter Withdraw", signature);

      const supporterBalanceAfter = await fetchBalance(
        provider,
        supporter.publicKey
      );
      const poolVaultBalanceAfter = await fetchBalance(provider, poolVaultPda);

      logData(`Supporter Balance: ${supporterBalanceAfter} SOL`);
      logData(`Pool Vault Balance: ${poolVaultBalanceAfter} SOL`);
      logData(
        `Received: ${(supporterBalanceAfter - supporterBalanceBefore).toFixed(
          4
        )} SOL`
      );

      logDone(`Supporter withdrew ${SUPPORTER_WITHDRAW_AMOUNT} SOL from pool!`);
    });
  });

  describe("complete lifecycle", () => {
    it("should handle full deposit-stake-stream-withdraw cycle", async () => {
      const newSupporter = Keypair.generate();
      await fundAccount(
        provider.connection,
        provider.wallet.payer,
        newSupporter.publicKey,
        1
      );

      const newSupporterTokenAccount = getAssociatedTokenAddressSync(
        poolMintPda,
        newSupporter.publicKey
      );

      // Deposit
      const depositAmount = 0.5;
      await program.methods
        .deposit(new BN(depositAmount * LAMPORTS_PER_SOL))
        .accountsStrict({
          supporter: newSupporter.publicKey,
          pool: poolPda,
          poolMint: poolMintPda,
          poolVault: poolVaultPda,
          supporterPoolTokenAccount: newSupporterTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([newSupporter])
        .rpc();

      // Stake
      await program.methods
        .stake(new BN(0.3 * LAMPORTS_PER_SOL))
        .accountsStrict({
          pool: poolPda,
          marinadeState: MAR_STATE,
          msolMint: MSOL_MINT,
          liqPoolSolLeg: LIQ_POOL_SOL_LEG,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          liqPoolMsolLegAuthority: MSOL_LEG_AUTH,
          reservePda: RESERVE_PDA,
          poolVault: poolVaultPda,
          poolMsolAccount,
          msolMintAuthority: MSOL_MINT_AUTH,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          marinadeProgram: MAR_PROGRAM_ID,
        })
        .rpc();

      // Stream yields
      await program.methods
        .stream()
        .accountsStrict({
          pool: poolPda,
          organizationVault: orgVaultPda,
          marinadeState: MAR_STATE,
          msolMint: MSOL_MINT,
          liqPoolSolLeg: LIQ_POOL_SOL_LEG,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          treasuryMsolAccount: TREASURY_MSOL,
          poolMsolAccount,
          poolVault: poolVaultPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          marinadeProgram: MAR_PROGRAM_ID,
        })
        .rpc();

      // Withdraw
      const balanceBefore = await fetchBalance(
        provider,
        newSupporter.publicKey
      );

      await program.methods
        .supporterWithdraw(new BN(0.1 * LAMPORTS_PER_SOL))
        .accountsStrict({
          supporter: newSupporter.publicKey,
          pool: poolPda,
          poolMint: poolMintPda,
          supporterPoolTokenAccount: newSupporterTokenAccount,
          marinadeState: MAR_STATE,
          msolMint: MSOL_MINT,
          liqPoolSolLeg: LIQ_POOL_SOL_LEG,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          treasuryMsolAccount: TREASURY_MSOL,
          poolMsolAccount,
          poolVault: poolVaultPda,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          marinadeProgram: MAR_PROGRAM_ID,
        })
        .signers([newSupporter])
        .rpc();

      const balanceAfter = await fetchBalance(provider, newSupporter.publicKey);
      expect(balanceAfter).to.be.above(balanceBefore);
    });
  });
});
