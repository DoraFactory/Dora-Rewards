#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::{fmt::Debug, Codec, DispatchResult},
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, ReservableCurrency},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{traits::AccountIdConversion, SaturatedConversion};
	use sp_std::{prelude::*, vec::Vec};
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// DoraFactory Crowdloan rewards pallet
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	pub const PALLET_ID: PalletId = PalletId(*b"DoraRewa");

	pub const VESTAMOUNT:u128 = 1000000000000;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		//TODO: set VestPercentage

	}

	#[pallet::storage]
	#[pallet::getter(fn contributors_list)]
	pub type ContributorsList<T: Config> = StorageValue<_, Vec<T::AccountId>>;

/* 	#[pallet::storage]
	#[pallet::getter(fn contributors)]
	pub type Contributors<T: Config> = StorageMap<_, T::AccountId, BalanceOf<T>, Option<OptionQuery>>;
 */
	#[pallet::storage]
	#[pallet::getter(fn contrbutor)]
	pub type Contributor<T: Config> = StorageValue<_, T::AccountId>;

	// Errors.
	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		//
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_n: T::BlockNumber) {
			let acc = T::Currency::free_balance(&Self::account_id());
			let acc_balance: u128 = acc.saturated_into();
			log::info!(":{:?}", acc);
			if acc_balance != 0 {
				//TODO:
				// read contributors list and will be distributed
				// let contributors = <ContrbutorsList<T>>::get().unwrap();
				Self::distribut_to_contributors(
					<Contributor<T>>::get().unwrap(),
					// vest amount
					VESTAMOUNT,
				);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn start_distribute(
			origin: OriginFor<T>,
			#[pallet::compact] value: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			T::Currency::transfer(
				&who,
				&Self::account_id(),
				value,
				ExistenceRequirement::KeepAlive,
			);
			Ok(().into())
		}

		// set an account to be disstributed, eg: here, we set a contributor
		#[pallet::weight(0)]
		pub fn set_contributor(origin: OriginFor<T>, contributor: T::AccountId) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			<Contributor<T>>::put(contributor);
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of the treasury pot.
		///
		/// This actually does computation. If you need to keep using it, then make sure you cache
		/// the value and only call this once.
		pub fn account_id() -> T::AccountId {
			PALLET_ID.into_account()
		}

		// distributed by Pallet account
		pub fn distribut_to_contributors(
			contributor_account: T::AccountId,
			value: u128,
		) {
			let pallet_acc = &Self::account_id();
			T::Currency::transfer(
				pallet_acc,
				&contributor_account,
				value.saturated_into(),
				ExistenceRequirement::AllowDeath,
			);
			
		}



		//TODO: compute vest amount
		// pub fn compute_Vest_once() -> u32{

		// }


	}
}
