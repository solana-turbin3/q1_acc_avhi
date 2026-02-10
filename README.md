## Programs

### [Whitelist Transfer Hook](https://github.com/solana-turbin3/q1_acc_avhi/tree/main/whitelist-transfer-hook)

A Solana program that enforces whitelist-based access control on token transfers using the SPL Token 2022 Transfer Hook interface. Only addresses added to the whitelist by the admin can transfer tokens. Each whitelisted address gets its own PDA account for O(1) lookup during transfer validation.

### [Escrow](https://github.com/solana-turbin3/q1_acc_avhi/tree/main/escrow-litesvm)

A two-party token escrow program built with Anchor. The maker deposits Token A into a PDA-controlled vault and specifies how much of Token B they want in return. A 5-day time lock is enforced after escrow creation before a taker can accept the offer, atomically swapping Token B for the vault's Token A. The maker can also cancel and reclaim their tokens via a refund instruction. Tests use [LiteSVM](https://github.com/LiteSVM/litesvm) with time travel (warp) for fast, in-process Solana program testing without a local validator.

### [Transfer Hook Vault](https://github.com/solana-turbin3/q1_acc_avhi/tree/main/transfer-hook-vault)

A whitelisted token vault built with Anchor and Token-2022. Only admin-approved users can hold and transfer the vault token - every `transfer_checked` triggers the on-chain transfer hook, which validates the sender against a PDA-based whitelist for O(1) lookup. Deposit and withdraw use a paired-instruction pattern (ledger update + `transfer_checked` in one atomic transaction) to work around Solana's reentrancy restriction. Withdrawals use a delegate approval so the hook always checks the user's whitelist rather than the vault PDA. The mint includes TransferHook, MetadataPointer, and TokenMetadata extensions. Tests use [LiteSVM](https://github.com/LiteSVM/litesvm) for fast, in-process testing without a local validator.

### [MagicBlock VRF](https://github.com/solana-turbin3/q1_acc_avhi/tree/main/magicblock-vrf)

A Solana program that integrates MagicBlock's Verifiable Random Function (VRF) to update on-chain user state with verifiable randomness. Implements VRF in two contexts: on the Solana base layer using the standard oracle queue, and inside a MagicBlock ephemeral rollup for faster and cheaper execution using the ephemeral oracle queue. The VRF request and callback follow a two-transaction pattern where the program CPIs into the VRF program to queue the request, and the oracle triggers the consume callback to write the random value into the user account.


