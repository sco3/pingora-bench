#!/usr/bin/env -S bash

set -ueo pipefail

source ./url.sh

# Source token from token file
TOKEN_FILE="$HOME/.local/mcpgateway-bearer-token.txt"
if [ ! -f "$TOKEN_FILE" ]; then
	echo "Error: Token file not found at $TOKEN_FILE" >&2
	exit 1
fi

AUTH="Bearer $(tr -d '\r\n' <"$TOKEN_FILE")"

./target/release/pingora-bench \
  --url $URL \
  --method POST \
  --duration 10 \
  --insecure \
  -H "Authorization: $AUTH" \
  -H "Content-Type: application/json" \
  --body '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_system_time","arguments":{"timezone":"UTC"}}}'
