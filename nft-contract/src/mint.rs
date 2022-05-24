use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        metadata: TokenMetadata,
        receiver_id: AccountId,
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
    ) {
        let initial_storage_usage = env::storage_usage();

        // create a royalty map to store in the token
        let mut royalty = HashMap::new();
        // iwe ahve perpetual royaties 
        if let Some(perpetual_royalties) = perpetual_royalties {
            // check bleow gas limit of 6 royalties 
            assert!(perpetual_royalties.len() < 7, "Cannot add more then 6 perpetyual royalties");
            // loop through and add acounts t the list 
            for (account, amount) in perpetual_royalties {
                royalty.insert(account, amount);
            }
        }
        let token = Token {
            owner_id: receiver_id,
            // an empty map to start with
            approved_account_ids: Default::default(),
            // approval id starts and 0 and increase by 1 each time it is used
            next_approval_id: 0,
            royalty,
        };
        assert!(self.tokens_by_id.insert(&token_id, &token).is_none(), "Token already exists!!");
        self.token_metadata_by_id.insert(&token_id, &metadata);
        self.internal_add_token_to_owner(&token.owner_id, &token_id);

        //event log 
        let nft_mint_log: EventLog = EventLog {
            standard: NFT_STANDARD_NAME.to_string(),
            version: NFT_METADATA_SPEC.to_string(),
            event: EventLogVariant::NftMint(vec![NftMintLog{
                owner_id: token.owner_id.to_string(),
                token_ids: vec![token_id.to_string()],
                //optional memo
                memo: None,
            }]),
        };
        // Log the serialized json.
        env::log_str(&nft_mint_log.to_string());

        //calculate the storage needed 
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;
        //refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover the required.
        refund_deposit(required_storage_in_bytes);
    }

}