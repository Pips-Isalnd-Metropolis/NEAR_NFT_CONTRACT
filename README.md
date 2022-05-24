
To BUILD and Deploy contract: 

yarn build && near deploy --wasmFile out/main.wasm --accountId **AcountID**

To itlilise contract: 
default setup with minimal data
near call **ACCOUNTID** new_default_meta '{"owner_id": "'**Acountid**'"}' --accountId ***acountid**
with full data
near call **ACCOUNTID** new METADATA_AS_BELOW --accountId ***acountid**

pub struct NFTContractMetadata {
    pub spec: String, // required, is essentially the version number
    pub name: String, // required name for the contract
    pub symbol: String, // required like the EPIC
    pub icon: Option<String>, //data url to represent the icon
    pub base_uri: Option<String>, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
    pub referance: Option<String>, // URL to a JSON file with more info
    pub referance_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

Minting example: 
near call **accountid** nft_mint '{"token_id": "test-token", "metadata": {"title": "Testing Token", "description": "testing nft contract", "media": "https://bafybeiftczwrtyr3k7a2k4vutd3amkwsmaqyhrdzlhvpt33dyjivufqusq.ipfs.dweb.link/goteam-gif.gif"}, "receiver_id": "'**acountid**'"}' --accountId **acountid** --amount 0.1

calling view function example: 
near view ***acountid*** nft_tokens_for_owner '{"account_id": "'**accountid**'", "limit": 10}'

balances passed in should be yoctoNEAR. 

Royalty percentages are bassed on 100% being 10,000 so we can have less then 1% without using decimals. 

Trackable evenst such as minting are tracked using a Json log who's name starts with "EVENT_JSON:"

Burning example: 

near call nftcontractdemo.testnet burn_nft '{"token_id": "test-token"}' --accountId **AccoutnID**