#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use frame_support::{dispatch, ensure, 
	traits::{
		Currency, 
		ExistenceRequirement,
		tokens::{
			WithdrawReasons
		}
	}
};
use frame_system::{
	self as system, 
	ensure_signed
};

pub use pallet_nakamoto;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod staking;
mod accounts;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::traits::Currency;

	/// ================
	/// ==== Config ====
	/// ================
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nakamoto::Config {
		/// --- Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// --- Currency type that will be used to place deposits on neurons
		type Currency: Currency<Self::AccountId> + Send + Sync;

		/// --- Initial token issuance at pallet construction.
		#[pallet::constant]
		type InitialIssuance: Get<u64>;
	}

	/// =================
	/// ==== Storage ====
	/// =================
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// --- Total number of peers across all subnetworks.
	#[pallet::storage]
	#[pallet::getter(fn n)]
	pub type N<T> = StorageValue<_, u64, ValueQuery>;

	// --- Stores total staked tao.
	#[pallet::storage]
	#[pallet::getter(fn total_stake)]
	pub type TotalStake<T> = StorageValue<_, u64, ValueQuery>;

	// --- Stores total tao issuance.
	#[pallet::type_value] 
	pub fn DefaultTotalIssuance<T: Config>() -> u64 { T::InitialIssuance::get() }
	#[pallet::storage]
	#[pallet::getter(fn total_issuance)]
	pub type TotalIssuance<T> = StorageValue<_, u64, ValueQuery, DefaultTotalIssuance<T>>;

	// --- Maps from hotkey account index to staked tokens.
	#[pallet::storage]
	#[pallet::getter(fn stake)]
    pub(super) type Stake<T:Config> = StorageMap<_, Identity, T::AccountId, u64, ValueQuery>;

	// --- Maps from hotkey account index to coldkey account.
	#[pallet::type_value] 
	pub fn DefaultHotkeyAccount<T: Config>() -> T::AccountId { T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap()}
	#[pallet::storage]
	#[pallet::getter(fn coldkey)]
    pub(super) type Coldkeys<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery, DefaultHotkeyAccount<T> >;

	// --- Maps from hotkey to u64 account index.
	#[pallet::type_value] 
	pub fn DefaultColdkeyAccount<T: Config>() -> T::AccountId { T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap()}
	#[pallet::storage]
	#[pallet::getter(fn hotkey)]
	pub(super) type Hotkeys<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery, DefaultColdkeyAccount<T> >;
	
	/// ===============
	/// ==== Events ===
	/// ===============
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// --- Event created when stake has been transfered from 
		/// the a coldkey account onto the hotkey staking account.
		StakeAdded(T::AccountId, u64),

		/// --- Event created when stake has been removed from 
		/// the hotkey staking account onto the coldkey account.
		StakeRemoved(T::AccountId, u64),
	}
	/// ================
	/// ==== Errors ====
	/// ================
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,

		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		/// ---- Thrown when the caller requests setting or removing data from
		/// a neuron which does not exist in the active set.
		NotRegistered,

		/// ---- Thrown when a stake, unstake or subscribe request is made by a coldkey
		/// which is not associated with the hotkey account. 
		/// See: fn add_stake and fn remove_stake.
		NonAssociatedColdKey,

		/// ---- Thrown when the caller requests removing more stake then there exists 
		/// in the staking account. See: fn remove_stake.
		NotEnoughStaketoWithdraw,

		///  ---- Thrown when the caller requests adding more stake than there exists
		/// in the cold key account. See: fn add_stake
		NotEnoughBalanceToStake,

		/// ---- Thrown when the caller tries to add stake, but for some reason the requested
		/// amount could not be withdrawn from the coldkey account
		BalanceWithdrawalError,
	}

	/// ================
	/// ==== Hooks =====
	/// ================
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
	}

	/// ======================
	/// ==== Dispatchables ===
	/// ======================
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// --- Adds stake to a neuron account. The call is made from the
		/// coldkey account linked in the neurons's NeuronMetadata.
		/// Only the associated coldkey is allowed to make staking and
		/// unstaking requests. This protects the neuron against
		/// attacks on its hotkey running in production code.
		///
		/// # Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, a coldkey signature associated with the hotkey account.
		///
		/// 	* 'hotkey' (T::AccountId):
		/// 		- The hotkey account to add stake to.
		///
		/// 	* 'ammount_staked' (u64):
		/// 		- The ammount to transfer from the balances account of the cold key
		/// 		into the staking account of the hotkey.
		///
		/// # Event:
		/// 	* 'StakeAdded':
		/// 		- On the successful staking of funds.
		///
		/// # Raises:
		/// 	* 'NotRegistered':
		/// 		- If the hotkey account is not active (has not subscribed)
		///
		/// 	* 'NonAssociatedColdKey':
		/// 		- When the calling coldkey is not associated with the hotkey account.
		///
		/// 	* 'InsufficientBalance':
		/// 		- When the amount to stake exceeds the amount of balance in the
		/// 		associated colkey account.
		///
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn add_stake(
			origin: OriginFor<T>, 
			hotkey: T::AccountId, 
			ammount_staked: u64
		) -> DispatchResult {
			Self::do_add_stake(origin, hotkey, ammount_staked)
		}

		/// ---- Remove stake from the staking account. The call must be made
		/// from the coldkey account attached to the neuron metadata. Only this key
		/// has permission to make staking and unstaking requests.
		///
		/// # Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, a coldkey signature associated with the hotkey account.
		///
		/// 	* 'hotkey' (T::AccountId):
		/// 		- The hotkey account to withdraw stake from.
		///
		/// 	* 'ammount_unstaked' (u64):
		/// 		- The ammount to transfer from the staking account into the balance
		/// 		of the coldkey.
		///
		/// # Event:
		/// 	* 'StakeRemoved':
		/// 		- On successful withdrawl.
		///
		/// # Raises:
		/// 	* 'NonAssociatedColdKey':
		/// 		- When the calling coldkey is not associated with the hotkey account.
		///
		/// 	* 'NotEnoughStaketoWithdraw':
		/// 		- When the amount to unstake exceeds the quantity staked in the
		/// 		associated hotkey staking account.
		///
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn remove_stake(
			origin: OriginFor<T>, 
			hotkey: T::AccountId, 
			ammount_unstaked: u64
		) -> DispatchResult {
			Self::do_remove_stake(origin, hotkey, ammount_unstaked)
		}
	}
}
