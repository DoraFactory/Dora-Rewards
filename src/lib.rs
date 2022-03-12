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
	#[pallet::without_storage_info]
	// The crowdloan rewards pallet
	pub struct Pallet<T>(PhantomData<T>);

	pub const PALLET_ID: PalletId = PalletId(*b"DoraRewa");

	pub const VESTAMOUNT: u128 = 1000000000000;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		//Set VestPercentage of contributions (1 KSM / DOT : xxx DORA)
		// #[pallet::constant]
		// type VestPercentage: Get<u32>;

		// One-time distribution ratio after the auction, and the left will be distribute by linear
		// #[pallet::constant]
		// type VestRatioOnce: Get<u32>;

		// max contributors number at once
		// #[pallet::constant]
		// type MaxContributrsNum: Get<u32>;
	}

	// single beneficiary account for test the auto distribute
	#[pallet::storage]
	#[pallet::getter(fn contrbutor)]
	pub type Contributor<T: Config> = StorageValue<_, T::AccountId>;

	//
	// 	contributors list:{  accountId  =>  contributions amount  }
	//
	#[pallet::storage]
	#[pallet::getter(fn contributor_list)]
	pub type ContributorList<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

	//
	// contributors rewards list : contributor account => how much money(this is the total rewards)
	//
	#[pallet::storage]
	#[pallet::getter(fn contributors_rewards)]
	pub type ContributorRewards<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

	// Errors.
	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// update the contributors list
		UpdateContributorsList(T::AccountId, BalanceOf<T>),
		// distribute Vest <source account, destination account, amount>
		DistributeVest(T::AccountId, T::AccountId, BalanceOf<T>),
		// 
		LeftRewards(T::AccountId, BalanceOf<T>),
		//
	}

	//
	// Question: how many accounts can be distributed in one block, that is very important.
	// i think we can set four accounts in one block to test.
	//
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: T::BlockNumber) {
			let pallet_acc = T::Currency::free_balance(&Self::account_id());
			let pallet_balance: u128 = pallet_acc.clone().saturated_into();
			log::info!("current pallet account balance is : {:?}", pallet_balance);
			// if the auction complete at 40th block, we can distribute 20% rewards to contributors
			log::info!("current block number is {:?}", n);
			if n == 1u32.into() {
				log::info!("This is hahaha!!!!!!!!!!!!!")
			}
			if n == 15u32.into() {
				let rewards_list_iter = <ContributorRewards<T>>::iter();
				for (acc, total_rewards) in rewards_list_iter {
					Self::distribute_to_contributors(
						acc.clone(),
						total_rewards.saturated_into::<u128>() * 20 / 100,
					);
					Self::deposit_event(<Event<T>>::DistributeVest(Self::account_id(),acc.clone(), (total_rewards.saturated_into::<u128>() * 20 / 100).saturated_into::<BalanceOf<T>>()));
					let left_rewards:u128 = total_rewards.saturated_into::<u128>() - total_rewards.saturated_into::<u128>() * 20 / 100;
					// update the total rewards to left rewards
					<ContributorRewards<T>>::insert(
						acc.clone(),
						left_rewards.saturated_into::<BalanceOf<T>>(),
					);
				}
			}

			if n > 20u32.into() {
				let rewards_iter = <ContributorRewards<T>>::iter();
				for (acc, left_rewards) in rewards_iter {
					// if the left reward < 10 ,则不进行分发了
					if left_rewards.saturated_into::<u128>() < 10 {
						break;
					}
					let rewards_per_block = left_rewards.saturated_into::<u128>() * 2 / 100;
					// distribute rewards by linear with block
					Self::distribute_to_contributors(
						acc.clone(),
						// each block distribute 2% left rewards
						rewards_per_block,
					);
					Self::deposit_event(<Event<T>>::DistributeVest(Self::account_id(), acc, rewards_per_block.saturated_into::<BalanceOf<T>>()));
				}
			}
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// The amount of funds this pallet controls
		pub funded_amount: BalanceOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { funded_amount: 1u32.into() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		// This sets the funds of the crowdloan pallet
		fn build(&self) {
			T::Currency::deposit_creating(&Pallet::<T>::account_id(), self.funded_amount);
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

		// set a contributors list
		#[pallet::weight(0)]
		pub fn set_contributors_list(
			origin: OriginFor<T>,
			acc: T::AccountId,
			// contribute XXX KSM/DOT
			#[pallet::compact] value: BalanceOf<T>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			// update the contributors list
			<ContributorList<T>>::insert(acc.clone(), value);
			Self::deposit_event(Event::UpdateContributorsList(acc, value));
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
		pub fn distribute_to_contributors(contributor_account: T::AccountId, value: u128) {
			let pallet_acc = &Self::account_id();
			T::Currency::transfer(
				pallet_acc,
				&contributor_account,
				value.saturated_into(),
				ExistenceRequirement::AllowDeath,
			);
		}

		// compute total vest amount of every contributor
		pub fn compute_vest_first() {
			let contributors_iter = <ContributorList<T>>::iter();
			// compute the total rewards of contributor's account by his amount of contributions
			for (acc, contrinutions) in contributors_iter {
				// contribute one KSM/DOT can get some DORA => rewards
				let total_rewards: u128 = contrinutions.saturated_into::<u128>() * 3;
				<ContributorRewards<T>>::insert(
					acc,
					total_rewards.saturated_into::<BalanceOf<T>>(),
				);
			}
		}
	}
}