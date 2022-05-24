use crate::*;


#[near_bindgen]
impl Contract {
    pub fn burn_nft(
        &mut self,
        token_id: TokenId
    ){
        //only can be run by collection owner 
        assert_eq!(self.owner_id, env::signer_account_id());
        //remove the NFT from the collection 
        //find who owns it. then delete it. 
        let token = self.tokens_by_id.get(&token_id);
        assert!(token.is_some());
        let x = token.unwrap();
        self.tokens_per_owner.remove(&x.owner_id);
        
        self.token_metadata_by_id.remove(&token_id);
        self.tokens_by_id.remove(&token_id);
        
    }

}