# Atharva ReFi Protocol

Atharva ReFi Protocol merges conservation with Solana, using Regenerative Finance to make protection of real-world endangered species come alive on the blockchain through transparent, incentivized impact.

**Stake SOL. Earn yields. Fund conservation. All on-chain.**

[![Solana](https://img.shields.io/badge/Solana-14F195?style=flat-square&logo=solana&logoColor=black)](https://solana.com)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)

---

## What This Does

Atharva ReFi lets you stake SOL to earn yields while automatically funding endangered species conservation.

- You stake SOL → Earn mSOL yields via Marinade
- 80% of yields stay with you
- 20% auto-streams to verified conservation organizations
- Withdraw anytime, no lock-ups
- Everything is on-chain and transparent

---

## How It Works

**For Supporters:**
1. Stake SOL in a species pool
2. Earn mSOL yields (you keep 80%)
3. 20% automatically goes to conservation
4. Withdraw principal + yields anytime

**For Organizations:**
1. Apply and get verified
2. Receive continuous yield streams
3. Fully auditable on-chain

**Example:** Stake 10 SOL at 6.5% APY
- You earn: 0.52 mSOL/year
- Conservation gets: 0.13 mSOL/year (automatically)

---

## Documentation

- **[User Stories](./docs/user-story.md)** - See how people use Atharva
- **[Letter of Intent](./docs/letter-of-intent.md)** - Our mission and vision
- **[Architecture Diagrams](./docs/)** - Technical design

---

## Architecture

**System Overview:**

![Scoped MVP](./docs/scoped-mvp.png)

**Detailed Flows:**

![Pool](./docs/create-pool.png) |
![Deposit](./docs/deposit.png) |
![Stream](./docs/stream.png) |

**Tech Stack:**
- Blockchain: Solana
- Smart Contracts: Anchor (Rust)
- Liquid Staking: Marinade Finance

---

## Getting Started

### Install
```bash
git clone https://github.com/yourusername/atharva-refi.git
cd atharva-refi
npm install
```

### Build & Test
```bash
anchor build
anchor test
```

### Local Development
```bash
solana-test-validator
anchor deploy
```

---

## Security Status

⚠️ **MVP in Development - Not production-ready**

**Current Issues:**
- Admin key is hardcoded (moving to multisig)
- Need reentrancy protection (implementing CEI pattern)
- Organization wallet validation needed
- Input validation in progress

**Before Mainnet:**
- Smart contract audit
- Multisig admin controls
- Emergency pause mechanism
- Bug bounty program

---

## Contributing

1. Fork the repo
2. Create your branch (`git checkout -b feature/name`)
3. Commit changes (`git commit -m 'Add feature'`)
4. Push and open a PR

**Need help with:**
- Security audits
- Documentation

---

## Project Structure
```
atharva-refi/
├── programs/atharva-refi/    # Smart contracts
├── tests/                    # Tests
├── app/                      # Frontend (coming soon)
└── docs/                     # Documentation
```

---

## License

MIT License - see [LICENSE](LICENSE)

---

**Built for endangered species and transparent impact 🌿**
