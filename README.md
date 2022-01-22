# Terra NFT Marketplace Smart Contract

The marketplace smart contracts provides a generic platform used for selling and buying CW721 tokens with CW20 tokens. It maintains a list of all current offerings, including the seller's address, the token ID put up for sale, the list price of the token and the contract address the offerings originated from. This ensures maximum visibility on a per-sale instead of a per-contract basis, allowing users to browse through list of offerings in one central place.

## Messages

## Contract Addresses

Testnet

| Contract        | Address                                       |
|:----------------|:----------------------------------------------|
| marketplace     | terra1708j7n4l2909z2afex09727mp22zl0ur6gtrn9  |

### Sell CW721 Token

Puts an NFT token up for sale.

> :warning: The seller needs to be the owner of the token to be able to sell it.

```shell
# Execute send_nft action to put token up for sale for specified list_price on the marketplace
terrad tx wasm execute <CW721_CONTRACT_ADDR> '{
  "send_nft": {
    "contract": "<MARKETPLACE_CONTRACT_ADDR>",
    "token_id": "<TOKEN_ID>",
    "msg": "BASE64_ENCODED_JSON --> { "list_price": { "address": "<INSERT_CW20_CONTRACT_ADDR>", "amount": "<INSERT_AMOUNT_WITHOUT_DENOM>" }} <--"
  }
}'  --from test1 --gas="auto" --gas=auto --fees=50000uluna --broadcast-mode=block --chain-id=localterra
```

### Withdraw CW721 Token Offering

Withdraws an NFT token offering from the global offerings list and returns the NFT token back to its owner.

> :warning: Only the token's owner/seller can withdraw the offering. This will only work after having used `sell_nft` on a token.

```shell
# Execute withdraw_nft action to withdraw the token with the specified offering_id from the marketplace
terrad tx wasm execute <MARKETPLACE_CONTRACT_ADDR> '{
  "withdraw_nft": {
    "offering_id": "<INSERT_OFFERING_ID>"
  }
}' --from test1 --gas="auto" --gas=auto --fees=50000uluna --broadcast-mode=block --chain-id=localterra
```

### Buy CW721 Token

Buys an NFT token, transferring funds to the seller and the token to the buyer.

> :warning: This will only work after having used `sell_nft` on a token.

```shell
# Execute send action to buy token with the specified offering_id from the marketplace
terrad tx wasm execute <CW_20_CONTRACT_ADDR> '{
  "send": {
    "contract": "<MARKETPLACE_CONTRACT_ADDR>",
    "amount": "<INSERT_AMOUNT>",
    "msg": "BASE64_ENCODED_JSON --> { "offering_id": "<INSERT_OFFERING_ID>" } <--"
  }
}'  --from test1 --gas="auto" --gas=auto --fees=50000uluna --broadcast-mode=block --chain-id=localterra
```

### Query Offerings

```shell
# Lists offerings sorted by lowest price, sort_listing can be one of following values (price_lowest, price_highest, newest_listed, oldest_listed), index > size (next chunk) size is defined as max chunk size

terrad query wasm contract-store <MARKETPLACE_CONTRACT_ADDR> '{"get_offerings":{"sort_listing":"price_lowest", "index": 0, "size": 5 }}' --chain-id=localterra
```

###

### Building Artifacts for Deployment

```shell
# Build artifacts

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer-arm64:0.12.4
```

### Deploy Artifacts

```shell
# Deploy artifacts

terrad tx wasm store ./artifacts/cw_marketplace-aarch64.wasm --from test1 --chain-id=localterra --gas=auto --fees=50000uluna --broadcast-mode=block
```

### Instantiate Contract
```shell
# Instantiate contract

terrad tx wasm instantiate 1 '{"name":"<MARKETPLACE_NAME>"}' --from test1 --chain-id=localterra --fees=50000uluna --gas=auto --broadcast-mode=block
```