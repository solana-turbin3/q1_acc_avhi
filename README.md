## Programs

### [Whitelist Transfer Hook](https://github.com/solana-turbin3/q1_acc_avhi/tree/main/whitelist-transfer-hook)

A Solana program that enforces whitelist-based access control on token transfers using the SPL Token 2022 Transfer Hook interface. Only addresses added to the whitelist by the admin can transfer tokens. Each whitelisted address gets its own PDA account for O(1) lookup during transfer validation.
