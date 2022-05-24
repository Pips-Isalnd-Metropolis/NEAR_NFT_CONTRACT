use crate::*;

pub trait NonFungibleTokenCore {
    //calculates the payout for a token given the passed in balance. This is a view method
  	fn nft_payout(&self, token_id: String, balance: U128, max_len_payout: u32) -> Payout;
    
    //transfers the token to the receiver ID and returns the payout object that should be payed given the passed in balance. 
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: String,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout;
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {

    //calculates the payout for a token given the passed in balance. This is a view method
    fn nft_payout(&self, token_id: String, balance: U128, max_len_payout: u32) -> Payout{
		let token = self.tokens_by_id.get(&token_id).expect("No toekn");
        let owner_id = token.owner_id;
        //keepp track of total perpeptual royalaties
        let mut total_perpetual = 0;
        //u128 version of balance
        let balance_u128 = u128::from(balance);
        //Payout object to send back
        let mut payout_object = Payout {
            payout: HashMap::new()
        };
        //roaytility object from the token
        let royalty = token.royalty;
        //check we are not at the limit of people we can payout. (limited by GAS)
        assert!(royalty.len() as u32 <= max_len_payout, "Market cannot payout to that many receivers");
        //loop through the royality object 
        for (k,v) in royalty.iter() {
            let key = k.clone();
            //only insert into the payout if the key isn't the token owner (we add their payout at the end)
            if key != owner_id {
                payout_object.payout.insert(key, royalty_to_payout(*v, balance_u128));
                total_perpetual += *v;
            }
        }
        //pay previous owner who get everything minu royatlties
        payout_object.payout.insert(owner_id, royalty_to_payout(10000 - total_perpetual, balance_u128));
        // return payout
        payout_object
	}

    //transfers the token to the receiver ID and returns the payout object that should be payed given the passed in balance. 
    #[payable]
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: String,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        //transfer the token to the passed in receiver and get the previous token object back
        let previous_token = self.internal_transfer(
            &sender_id,
            &receiver_id, 
            &token_id,
            Some(approval_id),
            Some(memo),
        );
        //refund previois owner any unsed storage
        refund_approved_account_ids(previous_token.owner_id.clone(), &previous_token.approved_account_ids);

        let owner_id = previous_token.owner_id;
        let mut total_perpetual = 0;
        let balance_u128 = u128::from(balance);
        //payout object to send back 
        let mut payout_object = Payout {
            payout: HashMap::new()
        };

        let royalty = previous_token.royalty;
        assert!(royalty.len() as u32 <= max_len_payout, "Market cannot pay that many people");
        // loop throug object to get payouts
        for (k,v) in royalty.iter(){
            let key = k.clone();
            //only add if not owner
            if key != owner_id {
                payout_object.payout.insert(key, royalty_to_payout(*v, balance_u128));
                total_perpetual += *v;
            }
        }
        payout_object.payout.insert(owner_id, royalty_to_payout(10000 - total_perpetual, balance_u128));
        payout_object
    }
}
