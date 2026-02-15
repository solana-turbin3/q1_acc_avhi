import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { PythScheduler } from "../target/types/pyth_scheduler";
import { init as initTuktuk, taskQueueAuthorityKey } from "@helium/tuktuk-sdk";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";
import { HermesClient } from "@pythnetwork/hermes-client";

describe("pyth-scheduler", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = provider.wallet as anchor.Wallet;
  const program = anchor.workspace.PythScheduler as Program<PythScheduler>;

  const TUKTUK_PROGRAM_ID = new PublicKey(
    "tuktukUrfhXT6ZT77QTU8RQtvgL967uRuVagWF57zVA"
  );
  const TASK_QUEUE = new PublicKey(
    "UwdRmurFA11isBpDNY9HNcoL95Pnt4zNYE2cd1SQwn2"
  );

  const SOL_USD_FEED_ID =
    "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";

  const [priceStorePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("price")],
    program.programId
  );

  const [queueAuthorityPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("queue_authority")],
    program.programId
  );

  const pythSolanaReceiver = new PythSolanaReceiver({
    connection: provider.connection,
    wallet,
  });

  const priceUpdateAccount = pythSolanaReceiver.getPriceFeedAccountAddress(
    0,
    SOL_USD_FEED_ID
  );

  describe("Update Price", () => {
    it("Fetches SOL/USD from Pyth and stores on-chain", async () => {
      const hermes = new HermesClient("https://hermes.pyth.network", {});
      const priceUpdateData = (
        await hermes.getLatestPriceUpdates([SOL_USD_FEED_ID], {
          encoding: "base64",
        })
      ).binary.data;

      const txBuilder = pythSolanaReceiver.newTransactionBuilder({
        closeUpdateAccounts: false,
      });
      await txBuilder.addPostPriceUpdates(priceUpdateData);

      console.log(
        "Posting price update, account:",
        priceUpdateAccount.toBase58()
      );
      const sigs = await pythSolanaReceiver.provider.sendAll(
        await txBuilder.buildVersionedTransactions({
          computeUnitPriceMicroLamports: 50000,
        }),
        { skipPreflight: true }
      );
      for (const sig of sigs) {
        const latest = await provider.connection.getLatestBlockhash();
        await provider.connection.confirmTransaction(
          { signature: sig, ...latest },
          "confirmed"
        );
      }
      console.log("Price update confirmed:", sigs);

      const tx = await program.methods
        .updatePrice()
        .accountsPartial({
          payer: wallet.publicKey,
          priceStore: priceStorePda,
          priceFeed: priceUpdateAccount,
          systemProgram: SYSTEM_PROGRAM_ID,
        })
        .rpc({ commitment: "confirmed" });

      console.log("Update price tx:", tx);

      const priceStore = await program.account.priceStore.fetch(priceStorePda);
      const actualPrice =
        priceStore.price.toNumber() * Math.pow(10, priceStore.exponent);
      console.log(`SOL/USD: $${actualPrice.toFixed(4)}`);
      console.log(`Confidence: ±${priceStore.confidence}`);
      console.log(
        `Published at: ${new Date(
          priceStore.publishedAt.toNumber() * 1000
        ).toISOString()}`
      );
    });
  });

  describe("Schedule", () => {
    it("Schedules update_price via TukTuk", async () => {
      const tuktukProgram = await initTuktuk(provider);

      const tqAccount = await tuktukProgram.account.taskQueueV0.fetch(
        TASK_QUEUE
      );
      const occupied = tqAccount.taskBitmap.reduce(
        (acc: number, b: number) => acc + bin(b),
        0
      );
      function bin(n: number) {
        let c = 0;
        for (let i = 0; i < 8; i++) if (n & (1 << i)) c++;
        return c;
      }
      if (occupied >= tqAccount.capacity) {
        console.log(
          `Queue full (${occupied}/${tqAccount.capacity}), increasing capacity...`
        );
        await tuktukProgram.methods
          .updateTaskQueueV0({
            capacity: tqAccount.capacity + 10,
            minCrankReward: null,
            lookupTables: null,
            updateAuthority: null,
            staleTaskAge: null,
          })
          .accounts({
            payer: wallet.publicKey,
            updateAuthority: wallet.publicKey,
            taskQueue: TASK_QUEUE,
          })
          .rpc({ commitment: "confirmed" });
        console.log("Capacity increased to", tqAccount.capacity + 10);
      }

      // register queue authority if not already
      const tqAuthPda = taskQueueAuthorityKey(TASK_QUEUE, queueAuthorityPda)[0];
      const tqAuthInfo = await provider.connection.getAccountInfo(tqAuthPda);
      if (!tqAuthInfo) {
        console.log("Registering queue authority...");
        const regTx = await tuktukProgram.methods
          .addQueueAuthorityV0()
          .accounts({
            payer: wallet.publicKey,
            queueAuthority: queueAuthorityPda,
            taskQueue: TASK_QUEUE,
          })
          .rpc({ commitment: "confirmed" });
        console.log("Registered:", regTx);
      } else {
        console.log("Queue authority already registered.");
      }

      let taskId = 0;
      let taskAccount: PublicKey = PublicKey.default;
      for (let candidate = 0; candidate < 256; candidate++) {
        const buf = Buffer.alloc(2);
        buf.writeUInt16LE(candidate);
        const [acc] = PublicKey.findProgramAddressSync(
          [Buffer.from("task"), TASK_QUEUE.toBuffer(), buf],
          TUKTUK_PROGRAM_ID
        );
        const info = await provider.connection.getAccountInfo(acc);
        if (!info) {
          taskId = candidate;
          taskAccount = acc;
          break;
        }
      }
      const [tqAuthorityPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("task_queue_authority"),
          TASK_QUEUE.toBuffer(),
          queueAuthorityPda.toBuffer(),
        ],
        TUKTUK_PROGRAM_ID
      );

      console.log("task_id:", taskId);
      console.log("task:", taskAccount.toBase58());

      const tx = await program.methods
        .schedule(taskId)
        .accountsPartial({
          payer: wallet.publicKey,
          priceStore: priceStorePda,
          priceFeed: priceUpdateAccount,
          taskQueue: TASK_QUEUE,
          taskQueueAuthority: tqAuthorityPda,
          task: taskAccount,
          queueAuthority: queueAuthorityPda,
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
