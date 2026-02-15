import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { GptOracle } from "../target/types/gpt_oracle";

describe("gpt-oracle", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = provider.wallet as anchor.Wallet;
  const program = anchor.workspace.gptOracle as Program<GptOracle>;

  const ORACLE_PROGRAM_ID = new PublicKey(
    "LLMrieZMpbJFwN52WgmBNMxYojrpRVYXdC1RCweEbab"
  );

  const getCounterPda = () =>
    PublicKey.findProgramAddressSync(
      [Buffer.from("counter")],
      ORACLE_PROGRAM_ID
    );

  const getAgentPda = () =>
    PublicKey.findProgramAddressSync(
      [Buffer.from("agent"), wallet.publicKey.toBuffer()],
      program.programId
    );

  const getLlmContextPda = (count: number) =>
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("test-context"),
        new Uint8Array(new Uint32Array([count]).buffer),
      ],
      ORACLE_PROGRAM_ID
    );

  const getInteractionPda = (context: PublicKey) =>
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("interaction"),
        wallet.publicKey.toBuffer(),
        context.toBuffer(),
      ],
      ORACLE_PROGRAM_ID
    );

  describe("Initialization", () => {
    it("Initializes agent if not already created", async () => {
      const [counterPda] = getCounterPda();
      const [agentPda] = getAgentPda();

      const agentInfo = await provider.connection.getAccountInfo(agentPda);
      if (agentInfo) {
        console.log("Agent already initialized, skipping...");
        return;
      }

      const counterInfo = await provider.connection.getAccountInfo(counterPda);
      const count = counterInfo!.data.readUInt32LE(8);

      const [llmContextPda] = getLlmContextPda(count);

      const tx = await program.methods
        .initialize()
        .accountsPartial({
          payer: wallet.publicKey,
          agent: agentPda,
          counter: counterPda,
          llmContext: llmContextPda,
          oracleProgram: ORACLE_PROGRAM_ID,
          systemProgram: SYSTEM_PROGRAM_ID,
        })
        .rpc();

      console.log("Initialize tx:", tx);
    });
  });

  describe("Interaction", () => {
    it("Interacts with LLM", async () => {
      const [agentPda] = getAgentPda();
      const agentAccount = await program.account.agent.fetch(agentPda);

      const llmContextPda = agentAccount.context;
      const [interactionPda] = getInteractionPda(llmContextPda);

      const tx = await program.methods
        .interactWithLlm()
        .accountsPartial({
          interaction: interactionPda,
          payer: wallet.publicKey,
          systemProgram: SYSTEM_PROGRAM_ID,
          oracleProgram: ORACLE_PROGRAM_ID,
          agent: agentPda,
          contextAccount: llmContextPda,
        })
        .rpc();

      console.log("Interaction tx:", tx);
    });
  });
});
