// import { LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
// import { Program } from "@coral-xyz/anchor";
// import { AccountInfoBytes, LiteSVM } from "litesvm";
// import idl from "../../target/idl/atharva_refi.json";
// import { AtharvaRefi } from "../../target/types/atharva_refi";

// export async function setupAccount(
//   accounts: { pubkey: PublicKey; account: AccountInfoBytes }[] = []
// ) {
//   const litesvm = new LiteSVM();

//   // Set up Pyth price feed
//   //   await litesvm.setAccount(SOL_USD_PRICE_UPDATE_V2, {
//   //     lamports: LAMPORTS_PER_SOL,
//   //     data: Buffer.from(solUsdPriceAcctInfo.account.data[0], "base64"),
//   //     owner: PYTH_SOLANA_RECEIVER_PROGRAM_ID,
//   //     executable: false,
//   //   });

//   // Set up additional test accounts
//   for (const { pubkey, account } of accounts) {
//     await litesvm.setAccount(pubkey, {
//       data: account.data,
//       executable: account.executable,
//       lamports: account.lamports,
//       owner: account.owner,
//     });
//   }

//   const provider = new LiteSVMProvider(litesvm);
//   const program = new Program<AtharvaRefi>(idl, litesvm);

//   return { litesvm, provider, program };
// }

// export function fundSetupAccount(
//   lamports: number = 3 * LAMPORTS_PER_SOL
// ): AccountInfoBytes {
//   return {
//     lamports,
//     data: Buffer.alloc(0),
//     owner: SystemProgram.programId,
//     executable: false,
//   };
// }
