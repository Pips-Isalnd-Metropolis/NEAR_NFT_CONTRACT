use crate::*;

#[near_bindgen]
impl Contract {

    //Query for nft tokens on the contract regardless of the owner using pagination
    pub fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<JsonToken> {
        //get a vector of the keys in the token_metadata_by_id collection.
        let keys = self.token_metadata_by_id.keys_as_vector();
        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = u128::from(from_index.unwrap_or(U128(0)));
        //loop through the keys vector
        keys.iter()
            .skip(start as usize) // skip to start var
            .take(limit.unwrap_or(0) as usize)
            .map(|token_id| self.nft_token(token_id.clone()).unwrap())
            .collect() //iter back to vector
    }

    //get the total supply of NFTs for a given owner
    pub fn nft_supply_for_owner(
        &self,
        account_id: AccountId,
    ) -> U128 {
        //get the set of tokens for the passed in owner
        let tokens_for_owner_set = self.tokens_per_owner.get(&account_id);
        //if there is some set of tokens, we'll return the length as a U128
        if let Some(tokens_for_owner_set) = tokens_for_owner_set {
            U128(tokens_for_owner_set.len() as u128)
        }else{
            U128(0)
        }
    }

    //Query for all the tokens for an owner
    pub fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonToken> {
        //get the set of tokens for the account passed into the fn
        let tokens_for_owner_set = self.tokens_per_owner.get(&account_id);
        // if there are ay we put them in a var
        let tokens = if let Some(tokens_for_owner_set) = tokens_for_owner_set {
            tokens_for_owner_set
        }else{
            // if no set of tokens we return empty vec
            return vec![];
        };
        //convert set into vector
        let keys = tokens.as_vector();
        // if there is a from use that as start otherwise start at 0
        let start = u128::from(from_index.unwrap_or(U128(0)));
        //loop through the keys vector
        keys.iter()
        .skip(start as usize) //skip to start possition
        .take(limit.unwrap_or(0) as usize) //take the first "limit" elements in the vector. If we didn't specify a limit, use 0
        .map(|token_id| self.nft_token(token_id.clone()).unwrap()) //we'll map the token IDs which are strings into Json Tokens
        .collect() //since we turned the keys into an iterator, we need to turn it back into a vector to return
    }   
}