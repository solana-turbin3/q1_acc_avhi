# NFT Staking Core — Challenge

Build on top of the existing Core NFT staking program or create your own from scratch.
You are free to modify existing methods, state accounts, or logic.

---

## Task 1: Core Plugins

### 1. Claim Rewards Without Unstaking
Create a `claim_rewards` instruction that lets users collect accumulated rewards without unstaking their NFT.

**Requirements:**
- Mint reward tokens to the user's ATA
- Keep the NFT staked and frozen


### 2. Burn-to-Earn with BurnDelegate

Create a `burn_staked_nft` instruction that lets users permanently burn their staked NFT for a massive one-time reward bonus.

**Requirements:**
- Mint reward tokens to the user's ATA
- Burn the NFT


### 3. Collection-Level Staking Stats (Attributes on Collection)

Track staking statistics at the collection level using Attributes on the Collection account itself.

**Requirements:**
- Add a `"total_staked"` counter as an Attribute on the Collection account
- Increment on stake, decrement on unstake

---

## Task 2: Oracle Plugin

Implement an external plugin (Oracle).

### Time-Based Transfer

NFTs can only be transferred during specific hours (e.g., 9AM-5PM UTC). Outside these hours, transferring is blocked.

**Requirements:**
- Create an Oracle Account to store the validation state (Approved/Rejected/Pass) per lifecycle event
- Create a method that reads the current on-chain time and updates the Transfer validation state accordingly
- Make the update instruction permissionless and reward the caller for cranking it at the right time
- Add the Oracle Plugin adapter to your Collection (on creation or later)
- Create a method in your program to transfer the NFT

**Tips:**
- The Oracle account should be a static address (PDA)
- The Oracle plugin should be set to check the lifecycle `Transfer` only and give it `REJECT` capability
- For the crank reward, store lamports in a vault PDA and only pay out when the update is called close to the open/close boundary
- The Oracle account should be added as a remaining account on the transfer CPI
