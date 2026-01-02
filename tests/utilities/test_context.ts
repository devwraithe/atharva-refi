// import { Program } from "@coral-xyz/anchor";
// import { Keypair, PublicKey } from "@solana/web3.js";
// import { getEscrowPda, getMarketPda, getEscrowVaultPda } from "./pdas";
// import { fundSetupAccount, setupAccount } from "./setup";
// import { LiteSVM } from "litesvm";
// import { AtharvaRefi } from "../../target/types/atharva_refi";

// export interface TestContext {
//   litesvm: LiteSVM;
//   program: Program<AtharvaRefi>;
//   supporter: Keypair;
//   organization: Keypair;
//   marketPda: PublicKey;
//   escrowPda: PublicKey;
//   escrowVaultPda: PublicKey;
// }

// // Test context global storage (for reusability)
// let globalTestContext: TestContext | null = null;

// export async function createTestContext(): Promise<TestContext> {
//   let litesvm: LiteSVM;
//   let program: Program<AtharvaRefi>;

//   const supporter = Keypair.generate();
//   const organization = Keypair.generate();

//   ({ litesvm, program } = await setupAccount([
//     // create supporter account and fund
//     {
//       pubkey: supporter.publicKey,
//       account: fundSetupAccount(),
//     },

//     // create organization account and fund
//     {
//       pubkey: organization.publicKey,
//       account: fundSetupAccount(),
//     },
//   ]));

//   const [marketPda] = getMarketPda(supporter.publicKey);
//   const [escrowPda] = getEscrowPda(marketPda);
//   const [escrowVaultPda] = getEscrowVaultPda(marketPda);

//   const context = {
//     litesvm,
//     program,
//     supporter,
//     organization,
//     marketPda,
//     escrowPda,
//     escrowVaultPda,
//   };

//   // Store context globally
//   globalTestContext = context;

//   return context;
// }

// export function getTestContext(): TestContext {
//   if (!globalTestContext) {
//     throw new Error(
//       "Test context not initialized. Call `createTestContext()` first."
//     );
//   }
//   return globalTestContext;
// }

// export function resetTestContext(): void {
//   globalTestContext = null;
// }
