use crate::*;
use near_sdk::{CryptoHash};
use std::mem::size_of;

//calculate how many bytes the account ID is taking up
pub(crate) fn bytes_for_approved_account_id(account_id: &AccountId) -> u64 {
    // The extra 4 bytes are coming from Borsh serialization to store the length of the string.
    account_id.as_str().len() as u64 + 4 + size_of::<u64>() as u64
}
//refund the storage taken up by passed in approved account IDs and send the funds to the passed in account ID. 
pub(crate) fn refund_approved_account_ids_iter<'a, I> (
    account_id: AccountId,
    approved_account_ids: I,
)-> Promise where I:Iterator<Item = &'a AccountId>,{
   //get the storage total by going through and summing all the bytes for each approved account IDs
   let storage_released: u64 = approved_account_ids.map(bytes_for_approved_account_id).sum();
   Promise::new(account_id).transfer(Balance::from(storage_released) * env::storage_byte_cost()) 
}
//refund a map of approved account IDs and send the funds to the passed in account ID
pub(crate) fn refund_approved_account_ids (
    account_id:AccountId,
    approved_account_ids: &HashMap<AccountId, u64>,
) -> Promise {
    refund_approved_account_ids_iter(account_id, approved_account_ids.keys())
}

//create a unique prefix to avoid data collisions
pub(crate) fn hash_account_id(account_id: &AccountId)->CryptoHash{
    //defualt hash
    let mut hash = CryptoHash::default();
    // we has the account id and return it
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

pub(crate) fn assert_one_yocto(){
    assert_eq!(env::attached_deposit(),1,"Requires attached deposit of exactly 1 yoctoNEAR")
}
pub(crate) fn assert_at_least_one_yocto() {
    assert!(
        env::attached_deposit() >= 1,
        "Requires attached deposit of at least 1 yoctoNEAR",
    )
}

pub(crate) fn refund_deposit(storage_used: u64) {
    // how much storage will cost 
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    // attached depostit
    let attached_deposit = env::attached_deposit();

    assert!(required_cost <= attached_deposit, "Must attach {} yoctoNEAR to cover storage", required_cost);

    let refund = attached_deposit - required_cost;

    //if the refund is greater than 1 yocto NEAR, we refund the predecessor that amount
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}
//convert the royalty percentage and amount to pay into a payout (U128)
// we use 100% to be 10,000 so that we can have percentages of less then 1
pub(crate) fn royalty_to_payout(royalty_percentage: u32, amount_to_pay: Balance) -> U128 {
    U128(royalty_percentage as u128 * amount_to_pay /10_000u128)
}

impl Contract {
    pub(crate) fn internal_add_token_to_owner (&mut self, account_id: &AccountId, token_id: &TokenId,) {
        let mut tokens_set = self.tokens_per_owner.get(account_id).unwrap_or_else(||{
            // if account has no NFTS yet we make a new set
            UnorderedSet::new(
                StorageKey::TokenPerOwnerInner{
                    account_id_hash: hash_account_id(&account_id),
                }.try_to_vec().unwrap()
            )
        });
        tokens_set.insert(token_id);
        self.tokens_per_owner.insert(account_id, &tokens_set);
    }

    pub(crate) fn internal_remove_token_from_owner(
        &mut self,
        account_id: &AccountId,
        token_id: &TokenId,
    ){
        //get tokens owner has 
        let mut tokens_set = self.tokens_per_owner.get(account_id).expect("Token should be owned by the sender");
        //remove the token from set
        tokens_set.remove(token_id);
        //if set now empty the we can remove the owner from the collection too
        if tokens_set.is_empty() {
            self.tokens_per_owner.remove(account_id);
        }else{
            // of sety not empty we can insert the upsated version back in 
            self.tokens_per_owner.insert(account_id, &tokens_set);
        }

    }

    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        token_id: &TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) -> Token {
        let token = self.tokens_by_id.get(token_id).expect("No token");
        //panic if seller is not the owner
        if sender_id != &token.owner_id {
            if !token.approved_account_ids.contains_key(sender_id){
                env::panic_str("Unauthorised");
            }
            if let Some(enforced_approval_id) = approval_id {
                let actual_approval_id = token
                        .approved_account_ids
                        .get(sender_id)
                        .expect("Sender is not approved account");
                assert_eq!(actual_approval_id, &enforced_approval_id,"The actual approval_id {} is different from the given approval id {}", actual_approval_id, enforced_approval_id);

            }
        }
        //check not trying to send token to themselves
        assert_ne!(&token.owner_id, receiver_id, "The token owner and receiver must be different");
        //remove tokem from current owner set 
        self.internal_remove_token_from_owner(&token.owner_id, token_id);
        //add token to receivers set
        self.internal_add_token_to_owner(receiver_id, token_id);
        //create new token struct
        let new_token = Token {
            owner_id: receiver_id.clone(),
            //reset approved id's 
            approved_account_ids: Default::default(),
            next_approval_id: token.next_approval_id,
            //copy from previous toekn 
            royalty: token.royalty.clone()
        };
        //insert the new token into tokens_by_id, replaing old entry.
        self.tokens_by_id.insert(token_id, &new_token);
        // log any attached memo
        if let Some(memo) = memo.as_ref() {
            env::log_str(&format!("Memo: {}", memo).to_string());
        }

        // default authorised DI to be none for the logs 
        let mut authorized_id = None;
        //if approval id supplied 
        if approval_id.is_some(){
            authorized_id = Some(sender_id.to_string());
        }

        let nft_transfer_log: EventLog = EventLog {
            standard: NFT_STANDARD_NAME.to_string(),
            version: NFT_METADATA_SPEC.to_string(),
            event: EventLogVariant::NftTransfer(vec![NftTransferLog{
                authorized_id,
                old_owner_id: token.owner_id.to_string(),
                new_owner_id: receiver_id.to_string(),
                token_ids: vec![token_id.to_string()],
                memo,
            }]),
        };
        // Log the serialized json.
        env::log_str(&nft_transfer_log.to_string());

        //return the preivous token object that was transferred.
        token
    }
}