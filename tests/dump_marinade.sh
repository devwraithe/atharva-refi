#!/bin/bash
MAINNET_RPC="https://api.mainnet-beta.solana.com"
DEVNET_RPC="https://api.devnet.solana.com"
DIR="marinade"
MAGIC_BLOCK_DIR="magic_block"

mkdir -p $DIR
mkdir -p $MAGIC_BLOCK_DIR

echo "Dumping Program..."
solana program dump -u $DEVNET_RPC MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD $DIR/marinade.so

echo "Dumping MagicBlock Program..."
solana program dump -u $DEVNET_RPC Magic11111111111111111111111111111111111111 $MAGIC_BLOCK_DIR/magic_block.so


# Define mapping: "address:filename"
ACCOUNTS=(
  "8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC:marinade_state"
  "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So:msol_mint"
  "Du3Ysj1wKbxPKkuPPnvzQLQh8oMSVifs3jGZjJWXFmHN:reserve_pda"
  "3JLPCS1qM2zRw3Dp6V4hZnYHd4toMNPkNesXdX9tg6KM:msol_mint_auth"
  "UefNb6z6yvArqe4cJHTXCqStRsKmWhGxnZzuHbikP5Q:liq_pool_sol_leg"
  "7GgPYjS5Dza89wV6FpZ23kUJRG5vbQ1GM25ezspYFSoE:liq_pool_msol_leg"
  "8ZUcztoAEhpAeC2ixWewJKQJsSUGYSGPVAjkhDJYf5Gd:treasury_msol"
  "EyaSjUtSgo9aRD1f8LWXwdvkpDTmXAW54yoSHZRF14WL:msol_leg_auth"
)

for item in "${ACCOUNTS[@]}"; do
  # Split the string by the colon
  ADDR="${item%%:*}"
  NAME="${item#*:}"
  
  echo "Dumping $NAME ($ADDR)..."
  solana account -u $DEVNET_RPC $ADDR --commitment confirmed --output json > "$DIR/$NAME.json"
  sleep 1 
done

echo "Done! Files created in $DIR"