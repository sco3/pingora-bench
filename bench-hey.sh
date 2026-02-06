hey -n 300000 -c 1 \
  -m POST \
  -H "Authorization: $AUTH" \
  -T "application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_system_time","arguments":{"timezone":"UTC"}}}' \
  "http://localhost:3000/mcp/"