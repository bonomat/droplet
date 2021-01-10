docker-compose -f docker-compose-e2e.yml up -d

sleep 5

native_asset_id=$(docker exec liquid-e2e-test elements-cli -rpcport=18884 -rpcuser=admin1 -rpcpassword=123 dumpassetlabels | jq -r '.bitcoin')

response=$(docker exec liquid-e2e-test elements-cli -rpcport=18884 -rpcuser=admin1 -rpcpassword=123 issueasset 10000000 1)
usdt_asset_id=$(echo $response | jq -r '.asset')

echo $native_asset_id
echo $usdt_asset_id

yarn install
export NATIVE_ASSET_ID=$native_asset_id
export USDT_ASSET_ID=$usdt_asset_id
export CHAIN="ELEMENTS"
export ESPLORA_API_URL="http://localhost:3012"
export REACT_APP_BLOCKEXPLORER_URL="http://localhost:5001"

yarn build

RUST_LOG=info,bobtimus=debug cargo run --bin fake_bobtimus -- \
        --elementsd http://admin1:123@127.0.0.1:7041 \
        --usdt $usdt_asset_id
