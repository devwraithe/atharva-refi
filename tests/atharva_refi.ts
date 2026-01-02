import { Keypair, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import { AtharvaRefi } from "../target/types/atharva_refi";
import idl from "../target/idl/atharva_refi.json";
import { fromWorkspace, LiteSVMProvider } from "anchor-litesvm";
import {
  bytesToString,
  getOrCreateAdminWallet,
  getPoolPdas,
  stringToBytes,
} from "./utilities";
import { expect } from "chai";

describe("atharva refi tests", () => {
  const svm = fromWorkspace("./");

  let provider: LiteSVMProvider;
  let program: Program<AtharvaRefi>;

  let admin: Keypair;
  let supporter: Keypair;
  let organization: Keypair;

  before(async () => {
    provider = new LiteSVMProvider(svm);
    program = new Program<AtharvaRefi>(idl, provider);

    admin = getOrCreateAdminWallet();
    supporter = Keypair.generate();
    organization = Keypair.generate();

    svm.airdrop(admin.publicKey, BigInt(10 * LAMPORTS_PER_SOL));
    svm.airdrop(supporter.publicKey, BigInt(10 * LAMPORTS_PER_SOL));
    svm.airdrop(organization.publicKey, BigInt(10 * LAMPORTS_PER_SOL));
  });

  it("creates a lion pool for londolozi fund", async () => {
    const organizationPubkey = organization.publicKey;
    const organizationName = "Londolozi Reserve";
    const speciesName = "African Lion";
    const speciesId = "panthera_leo"; // Max 32 chars

    // Convert strings to byte arrays
    const speciesIdBytes = stringToBytes(speciesId, 32);

    const { poolPda, poolVaultPda, orgVaultPda } = getPoolPdas(
      organizationPubkey,
      speciesIdBytes
    );

    console.log("--- ALL ACCOUNTS ---");
    console.log("Admin: ", admin.publicKey.toBase58());
    console.log("Supporter: ", supporter.publicKey.toBase58());
    console.log("Organization: ", organization.publicKey.toBase58());
    console.log("Pool PDA: ", poolPda.toBase58());
    console.log("Pool Vault PDA: ", poolVaultPda.toBase58());
    console.log("Org Vault PDA: ", orgVaultPda.toBase58());

    await program.methods
      .createPool(
        organizationName,
        organizationPubkey,
        speciesName,
        speciesIdBytes
      )
      .accountsStrict({
        admin: admin.publicKey,
        pool: poolPda,
        poolVault: poolVaultPda,
        organizationVault: orgVaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

    // Fetch and verify
    const poolAccount = await program.account.pool.fetch(poolPda);

    console.log("Pool created successfully!");
    console.log("Organization:", poolAccount.organizationName);
    console.log("Species:", poolAccount.speciesName);
    console.log("Species ID:", poolAccount.speciesId);
    console.log("Active:", poolAccount.isActive);
    console.log("Yield BPS:", poolAccount.organizationYieldBps);

    // Assertions
    expect(poolAccount.organizationName).to.equal(organizationName);
    expect(poolAccount.speciesName).to.equal(speciesName);
    expect(poolAccount.speciesId).to.equal(speciesId);
    expect(poolAccount.isActive).to.be.true;
  });
});
