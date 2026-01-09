# Atharva ReFi Protocol

**Stake SOL. Earn yields. Fund conservation. All on-chain.**

[![Solana](https://img.shields.io/badge/Solana-14F195?style=flat-square&logo=solana&logoColor=black)](https://solana.com)
[![Anchor](https://img.shields.io/badge/Anchor-v0.32-purple?style=flat-square)](https://anchor-lang.com)
[![Status](https://img.shields.io/badge/Status-MVP-orange?style=flat-square)]()

---

## What is Atharva ReFi?

Atharva ReFi turns your SOL into a force for conservation; without giving up your capital.

Stake SOL into species-specific pools (like Ranthambore Tigers). Your SOL earns mSOL yields through Marinade Finance. You keep 80% of yields and can withdraw anytime. The other 20% automatically streams to verified conservation organizations—transparently, continuously, on-chain.

No donations. No intermediaries. No trust required. Just verifiable impact.

---

## How It Works

### For Supporters

**Simple steps:**

1. Choose a species pool (e.g., Ranthambore Tigers)
2. Stake SOL through the protocol
3. Receive ARFI LP tokens (proof of your stake)
4. Earn mSOL yields—80% yours, 20% streams to conservation
5. Withdraw whenever you want

**Example:** Stake 10 SOL at 6.5% APY
- You keep: 0.52 mSOL/year (80%)
- Conservation: 0.13 mSOL/year (20%, auto-streamed)

### For Conservation Organizations

1. Submit application with verification documents
2. Get approved and receive your dedicated pool
3. Your verified wallet receives continuous yield streams
4. All funding is publicly auditable on Solana Explorer

---

## Key Features

| Feature | What It Means |
|---------|---------------|
| **Non-Custodial** | You control your funds, always |
| **Auto-Streaming** | Conservation funding happens automatically |
| **Transparent** | Every transaction visible on-chain |
| **No Lock-ups** | Withdraw anytime, no penalties |
| **Liquid Staking** | Your SOL earns yield via Marinade |
| **Verifiable Impact** | Track exactly where your yields go |

---

## Architecture

### System Overview

![Scoped MVP](./docs/scoped-mvp.png)

*The complete protocol flow—from user deposits to conservation funding*

### Core Flows

**Pool Creation**

![Pool Creation](./docs/create-pool.png)

How conservation organizations get onboarded and receive their dedicated species pool.

**Supporter Deposits**

![Supporter Deposits](./docs/deposit.png)

How supporters stake SOL and receive LP tokens representing their pool ownership.

**Yield Streaming**

![Yield Streaming](./docs/stream.png)

Automated 20% yield distribution to conservation wallets, 80% remains for supporters.

---

## Tech Stack

- Solana
- Rust
- Anchor Framework
- Marinade Finance (Liquid Staking)
- MagicBlock (Stream Automation)

---

## Getting Started

### Quick Setup
```bash
# Clone and install
git clone https://github.com/yourusername/atharva-refi.git
cd atharva-refi
npm install

# Build the program
anchor build

# Run tests
anchor test
```

### Local Development
```bash
# Start local Solana validator
solana-test-validator

# Deploy to localnet
anchor deploy

# Run tests against localnet
anchor test --skip-local-validator
```

---

## Project Structure
```
atharva-refi/
├── programs/
│   └── atharva-refi/            # Anchor program
│       ├── src/
│       │   ├── lib.rs           # Program entry point
│       │   ├── constants.rs     # Global constants
│       │   ├── errors.rs        # Custom program error definitions
│       │   ├── events.rs        # Anchor events for indexing & analytics
│       │   ├── utilities.rs     # Shared helpers and validation logic
│       │   ├── marinade/        # Marinade Finance CPI helpers
│       │   ├── instructions/    # Instruction handlers (deposit, withdraw, stream)
│       │   └── state/           # Account structs (Pool)
├── tests/                       # Anchor integration tests
├── app/                         # Frontend (coming soon)
└── docs/                        # Documentation & architecture diagrams
```

---

## Documentation

| Resource | Purpose |
|----------|---------|
| [User Stories](./docs/user-story.md) | Real-world usage scenarios |
| [Letter of Intent](./docs/letter-of-intent.md) | Project vision and mission |
| [Architecture Diagrams](./docs/) | Technical system design |

---

## Security

> ⚠️ **This is an MVP under active development. Not audited. Do not use with real funds.**

### Known Issues

| Priority | Issue | Mitigation Plan |
|----------|-------|-----------------|
| 🔴 Critical | Hardcoded admin pubkey | Moving to multisig config |
| 🟠 High | Reentrancy protection | Implementing CEI pattern |
| 🟠 High | Organization wallet validation | Adding withdrawal constraints |
| 🟡 Medium | Input validation | Adding comprehensive checks |

---

## Contributing

We welcome contributions from developers, conservationists, and anyone passionate about transparent impact.

### How to Contribute
```bash
1. Fork the repository
2. Create your feature branch (git checkout -b feature/amazing-feature)
3. Commit your changes (git commit -m 'feat: add amazing feature')
4. Push to the branch (git push origin feature/amazing-feature)
5. Open a Pull Request
```

---

**🌿 Building transparent conservation finance, one stake at a time**
