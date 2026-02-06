

set -ueo pipefail

source ./url.sh

# Source token from token-from-file.sh
TOKEN_FILE="$HOME/.local/mcpgateway-bearer-token.txt"
if [ ! -f "$TOKEN_FILE" ]; then
        echo "Error: Token file not found at $TOKEN_FILE" >&2
        exit 1
fi

AUTH="Bearer $(tr -d '\r\n' <"$TOKEN_FILE")"

hey -n 300 -c 1 \
  -m POST \
  -H "Authorization: $AUTH" \
  -T "application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_system_time","arguments":{"timezone":"UTC"}}}' \
  $URL
