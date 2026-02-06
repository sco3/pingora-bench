#!/usr/bin/env -S bash

set -ueo pipefail

# Source token from token-from-file.sh
TOKEN_FILE="$HOME/.local/mcpgateway-bearer-token.txt"
if [ ! -f "$TOKEN_FILE" ]; then
	echo "Error: Token file not found at $TOKEN_FILE" >&2
	exit 1
fi

AUTH="Bearer $(tr -d '\r\n' <"$TOKEN_FILE")"

curl -k \
  -X POST \
  -H "Authorization: $AUTH" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_system_time","arguments":{"timezone":"UTC"}}}' \
  https://localhost:3000/mcp/
