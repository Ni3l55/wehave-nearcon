use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen, ext_contract, require, env, AccountId, BorshStorageKey, Balance, CryptoHash, PanicOnDefault, Promise, Gas, PromiseError, PromiseOrValue};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;

const TGAS: u64 = 1_000_000_000_000;

// Define the state of the smart contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    // The NEP-141 item account that this DAO is about
    item_ft: AccountId,

    // The proposals that can be voted on [should we sell, should we lend, ...], mapped to an index
    proposals: UnorderedMap<u64, String>,

    // Per proposal the possible options [0 -> yes, no; 1 -> ok, maybe, idk]
    options: LookupMap<u64, Vector<String>>,

    // Votes that were cast for each proposal [0 -> niels.near -> 1, 1 -> root.near -> 0]
    votes: LookupMap<u64, UnorderedMap<AccountId, u64>>,

    // Calculated outcome of the votes, ordered by option index
    // TODO
}

// Define storage keys for lists
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Proposals,
    Options,
    Votes,
    ProposalVote { proposal_index_hash: CryptoHash}
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(item_ft: AccountId) -> Self {
        require!(!env::state_exists(), "Already initialized");

        Self{
            item_ft: item_ft,
            proposals: UnorderedMap::new(StorageKeys::Proposals),
            options: LookupMap::new(StorageKeys::Options),
            votes: LookupMap::new(StorageKeys::Votes),
        }
    }

    // Add a new proposal to vote upon
    pub fn new_proposal(&mut self, question: String, options: Vec<String>) {
        require!(question.chars().count() > 0); // Question cannot be empty
        require!(options.len() > 1); // At least 2 options to choose from

        log!("Creating new proposal: {} with options - {:?}", question, options);

        let amt = u128::from(self.proposals.len());

        // Save proposal
        self.proposals.insert(&amt, &question);

        // Save the options
        self.options.insert(&amt, Vector::from(options));

        // Make space for the votes on this proposal
        self.votes.insert(&amt, &UnorderedMap::new(StorageKeys::ProposalVote {
                                proposal_index_hash: env::sha256_array(&amt.to_be_bytes()),
                            })
                         );
    }

    // Get all proposals
    pub fn get_proposals(&self) -> UnorderedMap<u64, String> {
        self.proposals
    }

    // Get all votes on a certain proposal
    pub fn get_proposal_votes(&self, &proposal_index: u64) {
        self.votes.get(&proposal_index).expect("Incorrect proposal index!")
    }

    // Cast a vote
    pub fn cast_vote(&mut self, proposal_index: u64, answer_index: u64) {
        log!("Casting vote {} for proposal {}", answer_index, proposal_index);

        // If there already exists a vote -> overwrite
        let mut proposal_votes = self.get_proposal_votes(&proposal_index);
        proposal_votes.insert(&env::predecessor_account_id(), &answer_index);
        self.votes.insert(&proposal_index, &proposal_votes);
    }

    // Return weighted votes for a certain proposal
    // TODO figure out how to get weights atomically
    //pub fn calculate_votes(&self, proposal_index: u64) -> Vec<u64> {
    //    let proposal_votes = self.votes.get(&proposal_index).expect("Incorrect proposal index!");

        // Calculate by iterating over votes, add weight to vote based on user's amount of item tokens

    //}
}


/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new();
        testing_env!(context.is_view(true).build());
    }

    #[test]
    fn test_new_item() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new();
        let sample_item_name = String::from("rolex");
        contract.new_item(sample_item_name.clone());
        assert_eq!(contract.items.get(&0), Some(sample_item_name));
    }
}