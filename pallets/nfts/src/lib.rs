
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::{Currency, Get, ReservableCurrency},
	PalletId, BoundedVec,
};
use primitives::{TokenId, TokenIndex};
use sp_runtime::{RuntimeDebug, traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, One}};
use sp_std::{convert::TryInto, prelude::*};

use pallet_utils;

pub use pallet::*;


pub type CollectionId = u64;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen)]
pub struct Token<AccountId,CollectionId,BoundedString> {
	owner: AccountId,
	collection:CollectionId,
	name: BoundedString,
	symbol: BoundedString,
	base_uri: BoundedString,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum NftType {
	NonFungibleToken,
	MultiToken,
}

/// Collection info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Collection<AccountId> {
	/// Class owner
	pub owner: AccountId,
	// The type of nft
	pub nft_type: NftType,
	/// The account of nft
	pub nft_account: AccountId,
	/// Metadata from ipfs
	pub metadata: Vec<u8>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	use sp_runtime::traits::AccountIdConversion;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type PalletId: Get<PalletId>;

		/// Identifier for the class of token.
		type NonFungibleTokenId: Member  + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// The maximum length of base uri stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		/// The minimum balance to create token
		#[pallet::constant]
		type CreateTokenDeposit: Get<BalanceOf<Self>>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// The minimum balance to create collection
		#[pallet::constant]
		type CreateCollectionDeposit: Get<BalanceOf<Self>>;

		type CarbonZeroId : Get<u64>;

		type CarbonZeroRareId : Get<u64>;

		type CarbonZeroEpicId : Get<u64>;

		type CarbonZeroLegendaryId : Get<u64>;

		
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Tokens<T: Config> =
		StorageMap<_, Blake2_128Concat, T::NonFungibleTokenId, Token<T::AccountId, CollectionId,BoundedVec<u8, T::StringLimit>>>;

	#[pallet::storage]
	#[pallet::getter(fn next_token_id)]
	pub(super) type NextTokenId<T: Config> = StorageValue<_, T::NonFungibleTokenId, ValueQuery>;

	#[pallet::storage]
	pub type Collections<T: Config> =
		StorageMap<_, Blake2_128Concat, CollectionId, Collection<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn next_collection_id)]
	pub(super) type NextCollectionId<T: Config> = StorageValue<_, CollectionId, ValueQuery>;

	/// The NftMaster Account similar to treasury vault
    #[pallet::storage]
    #[pallet::getter(fn nft_master)]
    pub type NftMaster<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::storage]
	pub type IsLocked<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::NonFungibleTokenId,
		Blake2_128Concat, TokenId,
		u32,ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	pub type Owners<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		TokenId,
		T::AccountId,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		T::AccountId,
		u32,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_approved)]
	pub(super) type TokenApprovals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		TokenId,
		T::AccountId,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn is_approved_for_all)]
	pub(super) type OperatorApprovals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		// (owner, operator)
		(T::AccountId, T::AccountId),
		bool,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn total_supply)]
	pub(super) type TotalSupply<T: Config> =
		StorageMap<_, Blake2_128Concat, T::NonFungibleTokenId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn token_by_index)]
	pub(super) type AllTokens<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		TokenIndex,
		TokenId,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type AllTokensIndex<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		TokenId,
		TokenIndex,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn token_of_owner_by_index)]
	pub(super) type OwnedTokens<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		(T::AccountId, TokenIndex),
		TokenId,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type OwnedTokensIndex<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		TokenId,
		TokenIndex,
		ValueQuery,
	>;




    #[pallet::genesis_config]
    pub struct GenesisConfig<T:Config>{
		pub nft_master: T::AccountId,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			Self {
				nft_master: Default::default(),
			}
		}
	}

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
               // create a NTF class
			let treasury_acc = self.nft_master.clone();

			NftMaster::<T>::put(treasury_acc.clone());
			
			let col_id = Pallet::<T>::do_create_collection(&treasury_acc.clone(), NftType::NonFungibleToken, &treasury_acc.clone(), "yoyo".into()).unwrap();
			Pallet::<T>::do_create_token(&treasury_acc.clone(),col_id,  "first token".into(), "first token".into(), "first token".into());
			
        }
        
    }


	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TokenCreated(T::NonFungibleTokenId, T::AccountId),
		Transfer(T::NonFungibleTokenId, T::AccountId, T::AccountId, TokenId),
		Approval(T::NonFungibleTokenId, T::AccountId, T::AccountId, TokenId),
		ApprovalForAll(T::NonFungibleTokenId, T::AccountId, T::AccountId, bool),
		CollectionCreated(CollectionId, T::AccountId),
		CollectionDestroyed(CollectionId, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoAvailableTokenId,
		Overflow,
		Underflow,
		TokenAlreadyMinted,
		InvalidId,
		NoPermission,
		NotTokenOwner,
		TokenNonExistent,
		ApproveToCurrentOwner,
		NotOwnerOrApproved,
		ApproveToCaller,
		BadMetadata,
		LockedAsset,
		NoAvailableCollectionId,
		CollectionNotFound,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		

	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {


		#[pallet::weight(10_000)]
		pub fn create_collection(
			origin: OriginFor<T>,
			nft_type: NftType,

			nft_account: T::AccountId,
			metadata: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_create_collection(&who, nft_type, &nft_account, metadata)?;
			
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn destroy_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_destroy_collection(&who, collection_id)?;

			Ok(().into())
		}


		#[pallet::weight(10_000)]
		pub fn create_token(
			origin: OriginFor<T>,
			collection:CollectionId,
			name: Vec<u8>,
			symbol: Vec<u8>,
			base_uri: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_create_token(&who, collection,name, symbol, base_uri)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn approve(
			origin: OriginFor<T>,
			id: T::NonFungibleTokenId,
			to: T::AccountId,
			token_id: TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let owner = Self::owner_of(id, token_id);
			ensure!(
				owner != T::AccountId::default(),
				Error::<T>::TokenNonExistent
			);
	
			ensure!(to != owner, Error::<T>::ApproveToCurrentOwner);
			ensure!(
				who == owner || Self::is_approved_for_all(id, (&owner, &who)),
				Error::<T>::NotOwnerOrApproved
			);
	
			TokenApprovals::<T>::insert(id, token_id, &to);
	
			Self::deposit_event(Event::Approval(
				id,
				owner,
				to,
				token_id,
			));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn set_approve_for_all(
			origin: OriginFor<T>,
			id: T::NonFungibleTokenId,
			operator: T::AccountId,
			approved: bool,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(operator != who, Error::<T>::ApproveToCaller);

			OperatorApprovals::<T>::insert(id, (&who, &operator), approved);
	
			Self::deposit_event(Event::ApprovalForAll(
				id,
				who,
				operator,
				approved,
			));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			id: T::NonFungibleTokenId,
			from: T::AccountId,
			to: T::AccountId,
			token_id: TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				Self::is_approved_or_owner(id, &who, token_id),
				Error::<T>::NotOwnerOrApproved
			);

			Self::do_transfer_from(id, &from, &to, token_id)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn mint(
			origin: OriginFor<T>,
			id: T::NonFungibleTokenId,
			to: T::AccountId,
			token_id: TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::has_permission(id, &who), Error::<T>::NoPermission);

			Self::do_mint(id, &to, token_id)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn burn(
			origin: OriginFor<T>,
			id: T::NonFungibleTokenId,
			token_id: TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_burn(id, &who, token_id)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Returns the `AccountId` of the treasury account.
	pub fn treasury_account() -> T::AccountId {
			AccountIdConversion::into_account(&T::PalletId::get())
	}

	pub fn exists(id: T::NonFungibleTokenId, token_id: TokenId) -> bool {
		Owners::<T>::contains_key(id, token_id)
	}

	pub fn do_create_collection(
		who: &T::AccountId,
		nft_type: NftType,
		nft_account: &T::AccountId,
		metadata: Vec<u8>,
	) -> Result<CollectionId, DispatchError> {
		let collection_id =
			NextCollectionId::<T>::try_mutate(|id| -> Result<CollectionId, DispatchError> {
				let current_id = *id;
				*id = id
					.checked_add(One::one())
					.ok_or(Error::<T>::NoAvailableCollectionId)?;
				Ok(current_id)
			})?;

		let deposit = T::CreateCollectionDeposit::get();
		T::Currency::reserve(who, deposit.clone())?;

		let collection = Collection {
			owner: who.clone(),
			nft_type,
			nft_account: nft_account.clone(),
			metadata,
		};

	
		Collections::<T>::insert(collection_id, collection);

		Self::deposit_event(Event::CollectionCreated(collection_id, who.clone()));
		Ok(collection_id)
	} 

	pub fn do_destroy_collection(
		who: &T::AccountId,
		collection_id: CollectionId,
	) -> DispatchResult {
		Collections::<T>::try_mutate_exists(collection_id, |collection| -> DispatchResult {
			let c = collection.take().ok_or(Error::<T>::CollectionNotFound)?;
			ensure!(c.owner == *who, Error::<T>::NoPermission);

			let deposit = T::CreateCollectionDeposit::get();
			T::Currency::unreserve(who, deposit);

			Self::deposit_event(Event::CollectionDestroyed(collection_id, who.clone()));

			Ok(())
		})
	}

	pub fn do_create_token(
		who: &T::AccountId,
		collection:CollectionId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		base_uri: Vec<u8>,
	) -> Result<T::NonFungibleTokenId, DispatchError> {
		let deposit = T::CreateTokenDeposit::get();
		T::Currency::reserve(&who, deposit.clone())?;

		let bounded_name: BoundedVec<u8, T::StringLimit> =
			name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
		let bounded_symbol: BoundedVec<u8, T::StringLimit> =
			symbol.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
		let bounded_base_uri: BoundedVec<u8, T::StringLimit> =
			base_uri.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

		let id = NextTokenId::<T>::try_mutate(|id| -> Result<T::NonFungibleTokenId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableTokenId)?;
			Ok(current_id)
		})?;

		let token = Token {
			owner: who.clone(),
			collection:collection.clone(),
			name: bounded_name,
			symbol: bounded_symbol,
			base_uri: bounded_base_uri,
		};

		Tokens::<T>::insert(id, token);

		

		Self::deposit_event(Event::TokenCreated(id, who.clone()));

		Ok(id)
	}

	pub fn do_transfer_from(
		id: T::NonFungibleTokenId,
		from: &T::AccountId,
		to: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
		let owner = Self::owner_of(id, token_id);
		ensure!(
			owner != T::AccountId::default(),
			Error::<T>::TokenNonExistent
		);

		ensure!(IsLocked::<T>::get(id,token_id)==1,Error::<T>::LockedAsset);

		ensure!(owner == *from, Error::<T>::NotTokenOwner);

		let balance_from = Self::balance_of(id, from);
		let balance_to = Self::balance_of(id, to);

		let new_balance_from = match balance_from.checked_sub(1) {
			Some(c) => c,
			None => return Err(Error::<T>::Underflow.into()),
		};

		let new_balance_to = match balance_to.checked_add(1) {
			Some(c) => c,
			None => return Err(Error::<T>::Overflow.into()),
		};

		Self::remove_token_from_owner_enumeration(id, from, token_id)?;
		Self::add_token_to_owner_enumeration(id, to, token_id)?;

		Self::clear_approval(id, token_id)?;

		Balances::<T>::insert(id, from, new_balance_from);
		Balances::<T>::insert(id, to, new_balance_to);
		Owners::<T>::insert(id, token_id, to);

		Self::deposit_event(Event::Transfer(
			id.clone(),
			from.clone(),
			to.clone(),
			token_id,
		));

		Ok(())
	}

	pub fn do_mint(
		id: T::NonFungibleTokenId,
		to: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
		ensure!(
			!Self::exists(id, token_id),
			Error::<T>::TokenAlreadyMinted
		);

		let balance = Self::balance_of(id, to);

		let new_balance = match balance.checked_add(One::one()) {
			Some(c) => c,
			None => return Err(Error::<T>::Overflow.into()),
		};

		Self::add_token_to_all_tokens_enumeration(id, token_id)?;
		Self::add_token_to_owner_enumeration(id, to, token_id)?;

		Balances::<T>::insert(id, to, new_balance);
		Owners::<T>::insert(id, token_id, to);

		IsLocked::<T>::insert(id,token_id,0);

		Self::deposit_event(Event::Transfer(
			id.clone(),
			T::AccountId::default(),
			to.clone(),
			token_id,
		));

		Ok(())
	}

	pub fn do_burn(
		id: T::NonFungibleTokenId,
		account: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
		let owner = Self::owner_of(id, token_id);
		ensure!(
			owner != T::AccountId::default(),
			Error::<T>::TokenNonExistent
		);
		ensure!(*account == owner, Error::<T>::NotTokenOwner);

		let balance = Self::balance_of(id, &owner);

		let new_balance = match balance.checked_sub(One::one()) {
			Some(c) => c,
			None => return Err(Error::<T>::Underflow.into()),
		};

		Self::remove_token_from_all_tokens_enumeration(id, token_id)?;
		Self::remove_token_from_owner_enumeration(id, &owner, token_id)?;

		Self::clear_approval(id, token_id)?;

		Balances::<T>::insert(id, &owner, new_balance);
		Owners::<T>::remove(id, token_id);

		Self::deposit_event(Event::Transfer(
			id.clone(),
			owner.clone(),
			T::AccountId::default(),
			token_id,
		));

		Ok(())
	}

	fn is_approved_or_owner(
		id: T::NonFungibleTokenId,
		spender: &T::AccountId,
		token_id: TokenId,
	) -> bool {
		let owner = Self::owner_of(id, token_id);

		*spender == owner
			|| Self::get_approved(id, token_id) == *spender
			|| Self::is_approved_for_all(id, (&owner, spender))
	}

	fn has_permission(id: T::NonFungibleTokenId, who: &T::AccountId) -> bool {
		let token = Tokens::<T>::get(id).unwrap();
		*who == token.owner
	}

	fn clear_approval(id: T::NonFungibleTokenId, token_id: TokenId) -> DispatchResult {
		TokenApprovals::<T>::remove(id, token_id);
		Ok(())
	}

	fn add_token_to_owner_enumeration(
		id: T::NonFungibleTokenId,
		to: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
		let new_token_index = Self::balance_of(id, to);

		OwnedTokensIndex::<T>::insert(id, token_id, new_token_index);
		OwnedTokens::<T>::insert(id, (to, new_token_index), token_id);

		Ok(())
	}

	fn add_token_to_all_tokens_enumeration(
		id: T::NonFungibleTokenId,
		token_id: TokenId,
	) -> DispatchResult {
		TotalSupply::<T>::try_mutate(id, |total_supply| -> DispatchResult {
			let new_token_index = *total_supply;
			*total_supply = total_supply
				.checked_add(One::one())
				.ok_or(Error::<T>::Overflow)?;

			AllTokensIndex::<T>::insert(id, token_id, new_token_index);
			AllTokens::<T>::insert(id, new_token_index, token_id);

			Ok(())
		})?;

		Ok(())
	}

	fn remove_token_from_owner_enumeration(
		id: T::NonFungibleTokenId,
		from: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
		let balance_of_from = Self::balance_of(id, from);

		let last_token_index = match balance_of_from.checked_sub(One::one()) {
			Some(c) => c,
			None => return Err(Error::<T>::Overflow.into()),
		};

		let token_index = OwnedTokensIndex::<T>::get(id, token_id);

		if token_index != last_token_index {
			let last_token_id = OwnedTokens::<T>::get(id, (from, last_token_index));
			OwnedTokens::<T>::insert(id, (from, token_index), last_token_id);
			OwnedTokensIndex::<T>::insert(id, last_token_id, token_index);
		}

		OwnedTokensIndex::<T>::remove(id, token_id);
		OwnedTokens::<T>::remove(id, (from, last_token_index));

		Ok(())
	}

	fn remove_token_from_all_tokens_enumeration(
		id: T::NonFungibleTokenId,
		token_id: TokenId,
	) -> DispatchResult {
		let total_supply = Self::total_supply(id);

		let new_total_supply = match total_supply.checked_sub(One::one()) {
			Some(c) => c,
			None => return Err(Error::<T>::Overflow.into()),
		};

		let last_token_index = new_total_supply;

		let token_index = AllTokensIndex::<T>::get(id, token_id);

		let last_token_id = AllTokens::<T>::get(id, last_token_index);

		AllTokens::<T>::insert(id, token_index, last_token_id);
		AllTokensIndex::<T>::insert(id, last_token_id, token_index);

		AllTokens::<T>::remove(id, last_token_index);
		AllTokensIndex::<T>::remove(id, token_id);

		TotalSupply::<T>::insert(id, new_total_supply);

		Ok(())
	}
}