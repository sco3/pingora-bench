#!/usr/bin/env -S bash

set -ueo pipefail

# Source token from token file
TOKEN_FILE="$HOME/.local/mcpgateway-bearer-token.txt"
if [ ! -f "$TOKEN_FILE" ]; then
	echo "Error: Token file not found at $TOKEN_FILE" >&2
	exit 1
fi

AUTH="Bearer $(tr -d '\r\n' <"$TOKEN_FILE")"

./target/release/pingora-bench \
  --url http://localhost:3000/mcp/ \
  --method POST \
  --duration 10 \
  -n 1 \
  -H "Authorization: $AUTH" \
  -H "Content-Type: application/json" \
  --body '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_system_time","arguments":{"timezone":"UTC"}}}'
