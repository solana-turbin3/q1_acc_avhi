## Programs

### [Whitelist Transfer Hook](https://github.com/solana-turbin3/q1_acc_avhi/tree/main/whitelist-transfer-hook)

A Solana program that enforces whitelist-based access control on token transfers using the SPL Token 2022 Transfer Hook interface. Only addresses added to the whitelist by the admin can transfer tokens. Each whitelisted address gets its own PDA account for O(1) lookup during transfer validation.

### [Escrow](https://github.com/solana-turbin3/q1_acc_avhi/tree/main/escrow-litesvm)

A two-party token escrow program built with Anchor. The maker deposits Token A into a PDA-controlled vault and specifies how much of Token B they want in return. A 5-day time lock is enforced after escrow creation before a taker can accept the offer, atomically swapping Token B for the vault's Token A. The maker can also cancel and reclaim their tokens via a refund instruction. Tests use [LiteSVM](https://github.com/LiteSVM/litesvm) with time travel (warp) for fast, in-process Solana program testing without a local validator.
