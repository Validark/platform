curl -i -X POST -H "Content-Type: application/json; indent=4" \
-d '{
"jsonrpc": "2.0",
"method": "getBlockchainUser",
"params": {"name": "andy"},
"id": "1"
}' http://localhost:5001;echo
