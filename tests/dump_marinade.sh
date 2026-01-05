#!/bin/bash

# ----------------------------
# Configuration
# ----------------------------
MAINNET_RPC="https://api.mainnet-beta.solana.com"
DEVNET_RPC="https://api.devnet.solana.com"

MARINADE_DIR="marinade"
MAGIC_DIR="magic_block"

# Program IDs
MARINADE_PROG="MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD"
# MAGIC_PROG="Magic11111111111111111111111111111111111111"
DELEGATION_PROG="DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh"

# ----------------------------
# Init directories
# ----------------------------
mkdir -p "$MARINADE_DIR" "$MAGIC_DIR"

dump_program() {
    echo "Dumping Program: $2..."
    solana program dump -u "$MAINNET_RPC" "$1" "$2" \
      || echo "⚠️ Failed to dump program $1 from mainnet"
}

# ----------------------------
# 1. Dump Programs (mainnet only)
# ----------------------------
dump_program "$MARINADE_PROG" "$MARINADE_DIR/marinade.so"
# dump_program "$MAGIC_PROG" "$MAGIC_DIR/magic_block.so"
dump_program "$DELEGATION_PROG" "$MAGIC_DIR/delegation_program.so"

# ----------------------------
# 2. Dump Marinade State Accounts
# ----------------------------
# Format: "address:filename"
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

echo "Dumping accounts with mainnet → devnet fallback..."

for item in "${ACCOUNTS[@]}"; do
  ADDR="${item%%:*}"
  NAME="${item#*:}"
  OUT="$MARINADE_DIR/$NAME.json"

  echo "Processing $NAME ($ADDR)..."

  # Try mainnet first
  if solana account -u "$MAINNET_RPC" "$ADDR" --output json > "$OUT" 2>/dev/null; then
    echo "  ✅ Found on mainnet"
  else
    echo "  ⚠️ Not found on mainnet, trying devnet..."

    if solana account -u "$DEVNET_RPC" "$ADDR" --output json > "$OUT" 2>/dev/null; then
      echo "  ✅ Found on devnet"
    else
      echo "  ❌ Not found on mainnet or devnet"
      rm -f "$OUT"
    fi
  fi

  sleep 0.5
done

echo "✅ Done! Files organized in $MARINADE_DIR and $MAGIC_DIR"
