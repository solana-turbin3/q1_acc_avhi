import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { GptOracle } from "../target/types/gpt_oracle";
import { init as initTuktuk, taskQueueAuthorityKey } from "@helium/tuktuk-sdk";

describe("gpt-oracle", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = provider.wallet as anchor.Wallet;
  const program = anchor.workspace.gptOracle as Program<GptOracle>;

  const ORACLE_PROGRAM_ID = new PublicKey(
    "LLMrieZMpbJFwN52WgmBNMxYojrpRVYXdC1RCweEbab"
  );

  const TUKTUK_PROGRAM_ID = new PublicKey(
    "tuktukUrfhXT6ZT77QTU8RQtvgL967uRuVagWF57zVA"
  );
  const TASK_QUEUE = new PublicKey(
    "UwdRmurFA11isBpDNY9HNcoL95Pnt4zNYE2cd1SQwn2"
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

  const getQueueAuthorityPda = () =>
    PublicKey.findProgramAddressSync(
      [Buffer.from("queue_authority")],
      program.programId
    );

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

  describe("Schedule", () => {
    it("Schedules interact_with_llm via TukTuk", async () => {
      const tuktukProgram = await initTuktuk(provider);
      const [agentPda] = getAgentPda();
      const [queueAuthority] = getQueueAuthorityPda();

      const agentAccount = await program.account.agent.fetch(agentPda);
      const llmContextPda = agentAccount.context;
      const [interactionPda] = getInteractionPda(llmContextPda);

      // register queue authority if not already
      const tqAuthPda = taskQueueAuthorityKey(TASK_QUEUE, queueAuthority)[0];
      const tqAuthInfo = await provider.connection.getAccountInfo(tqAuthPda);
      if (!tqAuthInfo) {
        console.log("Registering queue authority...");
        const regTx = await tuktukProgram.methods
          .addQueueAuthorityV0()
          .accounts({
            payer: wallet.publicKey,
            queueAuthority,
            taskQueue: TASK_QUEUE,
          })
          .rpc({ skipPreflight: true, commitment: "confirmed" });
        console.log("Registered:", regTx);
      } else {
        console.log("Queue authority already registered.");
      }

      // find free task id
      const tqRaw = (await tuktukProgram.account.taskQueueV0.fetch(
        TASK_QUEUE
      )) as any;
      let taskId = 0;
      for (let i = 0; i < tqRaw.taskBitmap.length; i++) {
        if (tqRaw.taskBitmap[i] !== 0xff) {
          for (let bit = 0; bit < 8; bit++) {
            if ((tqRaw.taskBitmap[i] & (1 << bit)) === 0) {
              taskId = i * 8 + bit;
              break;
            }
          }
          break;
        }
      }

      const taskIdBuf = Buffer.alloc(2);
      taskIdBuf.writeUInt16LE(taskId);
      const [taskAccount] = PublicKey.findProgramAddressSync(
        [Buffer.from("task"), TASK_QUEUE.toBuffer(), taskIdBuf],
        TUKTUK_PROGRAM_ID
      );
      const [tqAuthorityPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("task_queue_authority"),
          TASK_QUEUE.toBuffer(),
          queueAuthority.toBuffer(),
        ],
        TUKTUK_PROGRAM_ID
      );

      console.log("task_id:", taskId);
      console.log("task:", taskAccount.toBase58());

      const tx = await program.methods
        .schedule(taskId)
        .accountsPartial({
          payer: wallet.publicKey,
          interaction: interactionPda,
          agent: agentPda,
          contextAccount: llmContextPda,
          taskQueue: TASK_QUEUE,
          taskQueueAuthority: tqAuthorityPda,
          task: taskAccount,
          queueAuthority,
          tuktukProgram: TUKTUK_PROGRAM_ID,
          systemProgram: SYSTEM_PROGRAM_ID,
        })
        .rpc({ skipPreflight: true, commitment: "confirmed" });

      console.log("Schedule tx:", tx);
      console.log(
        `\nhttps://explorer.solana.com/address/${program.programId.toBase58()}?cluster=devnet`
      );
    });
  });
});
