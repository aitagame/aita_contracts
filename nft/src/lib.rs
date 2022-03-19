use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{NonFungibleToken, Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedSet};
use near_sdk::json_types::ValidAccountId;
use near_sdk::{
    env, ext_contract, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, Promise,
    PromiseOrValue,
};

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    TokensPerOwner { account_hash: Vec<u8> },
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_default_meta(owner_id: ValidAccountId) -> Self {
        let aita_icon = include_str!("aita_icon.svg.dat");

        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "AITA Game NFT".to_string(),
                symbol: "AITANFT".to_string(),
                icon: Some(aita_icon.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    #[init]
    pub fn new(owner_id: ValidAccountId, metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        }
    }

    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        receiver_id: ValidAccountId,
        token_metadata: TokenMetadata,
    ) -> Token {
        self.tokens.mint(token_id, receiver_id, Some(token_metadata))
    }

    fn refund_deposit_to_account(storage_used: u64, account_id: AccountId) {
        let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
        let attached_deposit = env::attached_deposit();
        assert!(
            required_cost <= attached_deposit,
            "Must attach {} yoctoNEAR to cover storage",
            required_cost,
        );
        let refund = attached_deposit - required_cost;
        if refund > 1 {
            Promise::new(account_id).transfer(refund);
        }
    }

    #[payable]
    pub fn nft_mint_purchase(
        &mut self,
        token_id: TokenId,
        receiver_id: &AccountId,
        token_metadata: Option<TokenMetadata>,
    ) {
        let current_account_id = env::current_account_id();
        let predecessor_account_id = Some(env::predecessor_account_id());

        let initial_storage_usage =
            predecessor_account_id.map(|account_id| (account_id, env::storage_usage()));

        if token_metadata.is_none() {
            env::panic("Must provide metadata".as_ref());
        }
        if self.tokens.owner_by_id.get(&token_id).is_some() {
            env::panic("token_id must be unique".as_ref());
        }

        self.tokens.owner_by_id.insert(&token_id, &current_account_id);

        self.tokens
            .token_metadata_by_id
            .as_mut()
            .and_then(|by_id| by_id.insert(&token_id, token_metadata.as_ref().unwrap()));

        if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
            let mut token_ids = tokens_per_owner.get(&current_account_id).unwrap_or_else(|| {
                UnorderedSet::new(StorageKey::TokensPerOwner {
                    account_hash: env::sha256(current_account_id.as_bytes()),
                })
            });
            token_ids.insert(&token_id);
            tokens_per_owner.insert(&current_account_id, &token_ids);
        }

        if let Some((id, storage_usage)) = initial_storage_usage {
            Contract::refund_deposit_to_account(env::storage_usage() - storage_usage, id)
        }

        // get contract-level LookupMap of token_id to approvals HashMap
        let approvals_by_id = self.tokens.approvals_by_id.as_mut().unwrap();

        // update HashMap of approvals for this token
        let approved_account_ids =
            &mut approvals_by_id.get(&token_id).unwrap_or_else(|| HashMap::new());

        let approval_id: u64 = self
            .tokens
            .next_approval_id_by_id
            .as_ref()
            .unwrap()
            .get(&token_id)
            .unwrap_or_else(|| 1u64);
        approved_account_ids.insert(receiver_id.clone(), approval_id);

        // save updated approvals HashMap to contract's LookupMap
        approvals_by_id.insert(&token_id, &approved_account_ids);

        // increment next_approval_id for this token
        self.tokens.next_approval_id_by_id.as_mut().unwrap().insert(&token_id, &(approval_id + 1));

        self.tokens.internal_transfer(&current_account_id, &receiver_id, &token_id, None, None);
    }
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[ext_contract(ext_approval_receiver)]
pub trait NonFungibleTokenReceiver {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}
