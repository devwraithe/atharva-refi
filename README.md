## Atharva ReFi Protocol

Atharva ReFi Protocol merges conservation with Solana, using Regenerative Finance to make protection of real-world endangered species come alive on the blockchain through transparent, incentivized impact.

### User Story

You can view the User Story document [here](./docs/USER-STORY.md)

### Architecture Diagrams

#### 1. Scoped

![Atharva ReFi Scoped Architecture](./docs/scoped-v1.png)

#### 2. Detailed

![Atharva ReFi Detailed Architecture](./docs/detailed-v1.png)

### Possible Bugs

- ADMIN_PUBKEY is in code, what if an attacker decides to change it to theirs?
- Make sure species is lowercase to avoid errors
- Implement checks for all inputs and avoid duplication