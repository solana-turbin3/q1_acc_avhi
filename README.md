## Programs

### [Whitelist Transfer Hook](https://github.com/solana-turbin3/q1_acc_avhi/tree/main/whitelist-transfer-hook)

A Solana program that enforces whitelist-based access control on token transfers using the SPL Token 2022 Transfer Hook interface. Only addresses added to the whitelist by the admin can transfer tokens. Each whitelisted address gets its own PDA account for O(1) lookup during transfer validation.

### [Escrow](https://github.com/solana-turbin3/q1_acc_avhi/tree/main/escrow-litesvm)

A two-party token escrow program built with Anchor. The maker deposits Token A into a PDA-controlled vault and specifies how much of Token B they want in return. A 5-day time lock is enforced after escrow creation before a taker can accept the offer, atomically swapping Token B for the vault's Token A. The maker can also cancel and reclaim their tokens via a refund instruction. Tests use [LiteSVM](https://github.com/LiteSVM/litesvm) with time travel (warp) for fast, in-process Solana program testing without a local validator.

### [Transfer Hook Vault](https://github.com/solana-turbin3/q1_acc_avhi/tree/main/transfer-hook-vault)

A whitelisted token vault built with Anchor and Token-2022. Only admin-approved users can hold and transfer the vault token - every `transfer_checked` triggers the on-chain transfer hook, which validates the sender against a PDA-based whitelist for O(1) lookup. Deposit and withdraw use a paired-instruction pattern (ledger update + `transfer_checked` in one atomic transaction) to work around Solana's reentrancy restriction. Withdrawals use a delegate approval so the hook always checks the user's whitelist rather than the vault PDA. The mint includes TransferHook, MetadataPointer, and TokenMetadata extensions. Tests use [LiteSVM](https://github.com/LiteSVM/litesvm) for fast, in-process testing without a local validator.

### [MagicBlock VRF](https://github.com/solana-turbin3/q1_acc_avhi/tree/main/magicblock-vrf)

A Solana program that integrates MagicBlock's Verifiable Random Function (VRF) to update on-chain user state with verifiable randomness. Implements VRF in two contexts: on the Solana base layer using the standard oracle queue, and inside a MagicBlock ephemeral rollup for faster and cheaper execution using the ephemeral oracle queue. The VRF request and callback follow a two-transaction pattern where the program CPIs into the VRF program to queue the request, and the oracle triggers the consume callback to write the random value into the user account.

### [TukTuk Escrow](https://github.com/solana-turbin3/q1_acc_avhi/tree/main/tuktuk-escrow)

A trustless token escrow program with automated expiry refunds powered by [TukTuk](https://www.tuktuk.fun) - a permissionless crank scheduler on Solana. The maker deposits Token A and specifies how much Token B they want in return. A taker can fulfill the trade before the escrow expires. If no taker shows up, TukTuk's crankers automatically call `auto_refund` at expiry and return the maker's tokens - no manual intervention needed. Deployed and tested end-to-end on devnet.

### [GPT Oracle](https://github.com/solana-turbin3/q1_acc_avhi/tree/main/gpt-oracle)

A Solana program that queries an on-chain AI oracle (powered by [MagicBlock](https://magicblock.gg)) and schedules those queries automatically using [TukTuk](https://www.tuktuk.fun). An agent is initialized with a system prompt via CPI to the oracle's `create_llm_context`. TukTuk's crankers automatically fire `interact_with_llm` on a schedule, the oracle processes the query off-chain via GPT, and calls back into the program with the response via the `callback_from_llm` instruction. Deployed and tested end-to-end on devnet.

### [Pyth Scheduler](https://github.com/solana-turbin3/q1_acc_avhi/tree/main/pyth-scheduler)

A Solana program that fetches the SOL/USD price from [Pyth](https://pyth.network)'s pull oracle and stores it on-chain, with automated recurring updates powered by [TukTuk](https://www.tuktuk.fun). Uses the pull oracle model - the latest signed price is fetched from Hermes and posted to the Pyth Receiver program before being read. TukTuk's crankers automatically call `update_price` on a schedule to keep the stored price fresh. Deployed and tested end-to-end on devnet.
