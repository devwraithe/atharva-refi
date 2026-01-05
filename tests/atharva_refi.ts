import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { AtharvaRefi } from "../target/types/atharva_refi";
import { Keypair, LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";
import {
  fundAccount,
  getBalance,
  getOrCreateAdminWallet,
  getPoolPdas,
  getTokenBalance,
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
  let organization: Keypair;
  let supporterPoolTokenAccount: anchor.web3.PublicKey;
  let poolMsolAccount: anchor.web3.PublicKey;

  const ORGANIZATION_NAME = "Londolozi Reserve";
  const SPECIES_NAME = "African Lion";
  const SPECIES_ID = "panthera_leo";

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
    organization = Keypair.generate();

    await fundAccount(
      provider.connection,
      provider.wallet.payer,
      admin.publicKey,
      0.05
    );
    await fundAccount(
      provider.connection,
      provider.wallet.payer,
      organization.publicKey,
      0.01
    );
    await fundAccount(
      provider.connection,
      provider.wallet.payer,
      supporter.publicKey,
      0.15
    );
  });

  it("creates a lion conservation pool", async () => {
    const speciesIdBytes = stringToBytes(SPECIES_ID, 32);
    const { poolPda, poolMintPda, poolVaultPda, orgVaultPda } = getPoolPdas(
      organization.publicKey,
      speciesIdBytes
    );

    poolMsolAccount = getAssociatedTokenAddressSync(
      MSOL_MINT,
      poolVaultPda,
      true
    );

    const tx = await program.methods
      .createPool(
        ORGANIZATION_NAME,
        organization.publicKey,
        SPECIES_NAME,
        speciesIdBytes
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

    await provider.sendAndConfirm(tx, [admin]);
  });

  it("deposits SOL into the pool", async () => {
    const speciesIdBytes = stringToBytes(SPECIES_ID, 32);
    const { poolPda, poolMintPda, poolVaultPda } = getPoolPdas(
      organization.publicKey,
      speciesIdBytes
    );

    supporterPoolTokenAccount = getAssociatedTokenAddressSync(
      poolMintPda,
      supporter.publicKey
    );

    const tx = await program.methods
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

    await provider.sendAndConfirm(tx, [supporter]);

    getBalance(provider, "Pool Vault", poolVaultPda);
  });

  it("stakes SOL on Marinade and receives mSOL", async () => {
    const speciesIdBytes = stringToBytes(SPECIES_ID, 32);
    const { poolPda, poolVaultPda } = getPoolPdas(
      organization.publicKey,
      speciesIdBytes
    );

    const tx = await program.methods
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

    await provider.sendAndConfirm(tx, []);

    getTokenBalance(provider, "Pool mSOL", poolMsolAccount);
  });

  it("streams yield to organization vault", async () => {
    const speciesIdBytes = stringToBytes(SPECIES_ID, 32);
    const { poolPda, poolVaultPda, orgVaultPda } = getPoolPdas(
      organization.publicKey,
      speciesIdBytes
    );

    const tx = await program.methods
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

    await provider.sendAndConfirm(tx, []);

    getBalance(provider, "Organization Vault", orgVaultPda);
  });

  it("delegates pool to ephemeral rollups", async () => {
    const speciesIdBytes = stringToBytes(SPECIES_ID, 32);
    const { poolPda } = getPoolPdas(organization.publicKey, speciesIdBytes);

    const tx = await program.methods
      .delegate()
      .accountsPartial({
        payer: admin.publicKey,
        pool: poolPda,
      })
      .transaction();

    await provider.sendAndConfirm(tx, [admin]);
    await new Promise((resolve) => setTimeout(resolve, DELEGATION_WAIT_MS));
  });

  it("schedules automatic yield streaming", async () => {
    const speciesIdBytes = stringToBytes(SPECIES_ID, 32);
    const { poolPda, poolVaultPda, orgVaultPda } = getPoolPdas(
      organization.publicKey,
      speciesIdBytes
    );

    const orgBalanceBefore = await provider.connection.getBalance(orgVaultPda);

    const tx = await program.methods
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

    tx.feePayer = admin.publicKey;
    tx.recentBlockhash = (
      await providerER.connection.getLatestBlockhash()
    ).blockhash;
    tx.sign(admin);

    const signature = await providerER.connection.sendRawTransaction(
      tx.serialize(),
      { skipPreflight: true }
    );
    await providerER.connection.confirmTransaction(signature);

    await new Promise((resolve) => setTimeout(resolve, STREAM_WAIT_MS));

    const orgBalanceAfter = await provider.connection.getBalance(orgVaultPda);
    const streamed = orgBalanceAfter - orgBalanceBefore;

    console.log(`1st Streamed: ${streamed / LAMPORTS_PER_SOL} SOL`);
    getBalance(provider, "Organization Vault", orgVaultPda);

    const orgBalanceAfter2 = await providerER.connection.getBalance(
      orgVaultPda
    );
    console.log(
      `2nd Org Balance After: ${orgBalanceAfter2 / LAMPORTS_PER_SOL} SOL`
    );
  });

  it("undelegates pool from ephemeral rollups", async () => {
    const speciesIdBytes = stringToBytes(SPECIES_ID, 32);
    const { poolPda } = getPoolPdas(organization.publicKey, speciesIdBytes);

    const tx = await program.methods
      .undelegate()
      .accounts({
        payer: admin.publicKey,
        pool: poolPda,
      })
      .transaction();

    tx.feePayer = admin.publicKey;
    tx.recentBlockhash = (
      await providerER.connection.getLatestBlockhash()
    ).blockhash;
    tx.sign(admin);

    const signature = await providerER.connection.sendRawTransaction(
      tx.serialize(),
      { skipPreflight: true }
    );
    await providerER.connection.confirmTransaction(signature);

    await new Promise((resolve) => setTimeout(resolve, UNDELEGATION_WAIT_MS));

    await program.account.pool.fetch(poolPda);
  });

  it("unstakes mSOL on Marinade and receives SOL", async () => {
    const speciesIdBytes = stringToBytes(SPECIES_ID, 32);
    const { poolPda, poolVaultPda } = getPoolPdas(
      organization.publicKey,
      speciesIdBytes
    );

    const tx = await program.methods
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

    await provider.sendAndConfirm(tx, []);

    getBalance(provider, "Pool Vault", poolVaultPda);
  });

  it("withdraws organization yields", async () => {
    const speciesIdBytes = stringToBytes(SPECIES_ID, 32);
    const { poolPda, orgVaultPda } = getPoolPdas(
      organization.publicKey,
      speciesIdBytes
    );

    const tx = await program.methods
      .organizationWithdraw(new BN(ORG_WITHDRAW_AMOUNT * LAMPORTS_PER_SOL))
      .accountsStrict({
        organization: organization.publicKey,
        pool: poolPda,
        orgVault: orgVaultPda,
        systemProgram: SystemProgram.programId,
      })
      .transaction();

    await provider.sendAndConfirm(tx, [organization]);

    getBalance(provider, "Organization Vault", orgVaultPda);
  });

  it("withdraws supporter stake and yields", async () => {
    const speciesIdBytes = stringToBytes(SPECIES_ID, 32);
    const { poolPda, poolMintPda, poolVaultPda } = getPoolPdas(
      organization.publicKey,
      speciesIdBytes
    );

    const tx = await program.methods
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

    await provider.sendAndConfirm(tx, [supporter]);

    getBalance(provider, "Supporter", supporter.publicKey);
    getBalance(provider, "Pool Vault", poolVaultPda);
  });
});
