// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { AtharvaRefi } from "../target/types/atharva_refi";
// import { LiteSVM } from "litesvm";
// import { createTestContext } from "./utilities/test_context";

// describe("atharva_refi test", () => {
//   // Configure the client to use the local cluster.
//   anchor.setProvider(anchor.AnchorProvider.env());

//   let litesvm: LiteSVM;
//   let program: Program<AtharvaRefi>;

//   // const program = anchor.workspace.atharvaRefi as Program<AtharvaRefi>;

//   before(async () => {
//     const ctx = await createTestContext();
//     litesvm = ctx.litesvm;
//     program = ctx.program;
//   });

//   it("Is initialized!", async () => {
//     // Add your test here.
//     const tx = await program.methods.initialize().rpc();
//     console.log("Your transaction signature", tx);
//   });
// });
