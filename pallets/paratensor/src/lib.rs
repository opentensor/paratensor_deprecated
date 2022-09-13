#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
use frame_system::{self as system};

mod epoch;
mod misc;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::traits::Currency;
	use frame_support::inherent::Vec;
	use frame_support::sp_std::vec;

	/// ================
	/// ==== Config ====
	/// ================
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// --- Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// --- Currency type that will be used to place deposits on neurons
		type Currency: Currency<Self::AccountId> + Send + Sync;

		/// --- Initialization
		#[pallet::constant]
		type InitialIssuance: Get<u64>;

		/// --- Hyperparams
		#[pallet::constant]
		type InitialMinAllowedWeights: Get<u16>;

		#[pallet::constant]
		type InitialMaxAllowedMaxMinRatio: Get<u16>;
	}

	/// ==============================
	/// ==== Super Params Storage ====
	/// ==============================
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type GlobalN<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub type TotalStake<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::type_value] 
	pub fn DefaultTotalIssuance<T: Config>() -> u64 { T::InitialIssuance::get() }
	#[pallet::storage]
	pub type TotalIssuance<T> = StorageValue<_, u64, ValueQuery, DefaultTotalIssuance<T>>;

	/// ==============================
	/// ==== Accounts Storage ====
	/// ==============================
	#[pallet::storage]
    pub(super) type Stake<T:Config> = StorageMap<_, Identity, T::AccountId, u64, ValueQuery>;

	#[pallet::type_value] 
	pub fn DefaultHotkeyAccount<T: Config>() -> T::AccountId { T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap()}
	#[pallet::storage]
    pub(super) type Coldkeys<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery, DefaultHotkeyAccount<T> >;

	#[pallet::type_value] 
	pub fn DefaultColdkeyAccount<T: Config>() -> T::AccountId { T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap()}
	#[pallet::storage]
	pub(super) type Hotkeys<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery, DefaultColdkeyAccount<T> >;

	

	/// =======================================
	/// ==== Subnetork Hyperparam stroage  ====
	/// =======================================
	#[pallet::type_value] 
	pub fn DefaultMinAllowedWeights<T: Config>() -> u16 { T::InitialMinAllowedWeights::get() }
	#[pallet::storage]
	pub type MinAllowedWeights<T> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultMinAllowedWeights<T> >;

	#[pallet::type_value] 
	pub fn DefaultMaxAllowedMaxMinRatio<T: Config>() -> u16 { T::InitialMaxAllowedMaxMinRatio::get() }
	#[pallet::storage]
	pub type MaxAllowedMaxMinRatio<T> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultMaxAllowedMaxMinRatio<T> >;

	/// =======================================
	/// ==== Subnetwork Consensus Storage  ====
	/// =======================================
	#[pallet::type_value] 
	pub fn DefaultN<T:Config>() -> u16 { 0 }
	#[pallet::storage]
	pub(super) type SubnetworkN<T:Config> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultN<T> >;

	#[pallet::type_value] 
	pub fn DefaultKey<T:Config>() -> T::AccountId { T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap() }
	#[pallet::storage]
	pub(super) type Keys<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, T::AccountId, ValueQuery, DefaultKey<T> >;

	#[pallet::type_value] 
	pub fn DefaultUid<T:Config>() -> u16 { 0 }
	#[pallet::storage]
	pub(super) type Uids<T:Config> = StorageDoubleMap<_, Identity, u16, Blake2_128Concat, T::AccountId, u16, ValueQuery, DefaultUid<T> >;

	#[pallet::type_value] 
	pub fn DefaultWeights<T:Config>() -> Vec<(u16, u16)> { vec![] }
	#[pallet::storage]
    pub(super) type Weights<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, Vec<(u16, u16)>, ValueQuery, DefaultWeights<T> >;

	#[pallet::type_value] 
	pub fn DefaultBonds<T:Config>() -> Vec<(u16, u16)> { vec![] }
	#[pallet::storage]
    pub(super) type Bonds<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, Vec<(u16, u16)>, ValueQuery, DefaultBonds<T> >;

	#[pallet::type_value] 
	pub fn DefaultActive<T:Config>() -> Vec<bool> { vec![] }
	#[pallet::storage]
	pub(super) type Active<T:Config> = StorageMap< _, Identity, u16, Vec<bool>, ValueQuery, DefaultActive<T> >;

	#[pallet::type_value] 
	pub fn DefaultStake<T:Config>() -> Vec<u64> { vec![] }
	#[pallet::storage]
    pub(super) type S<T:Config> = StorageMap< _, Identity, u16, Vec<u64>, ValueQuery, DefaultStake<T> >;

	#[pallet::type_value] 
	pub fn DefaultRank<T:Config>() -> Vec<u16> { vec![] }
	#[pallet::storage]
	pub(super) type Rank<T:Config> = StorageMap< _, Identity, u16, Vec<u16>, ValueQuery, DefaultRank<T> >;

	#[pallet::type_value] 
	pub fn DefaultTrust<T:Config>() -> Vec<u16> { vec![] }
	#[pallet::storage]
	pub(super) type Trust<T:Config> = StorageMap< _, Identity, u16, Vec<u16>, ValueQuery, DefaultTrust<T> >;

	#[pallet::type_value] 
	pub fn DefaultIncentive<T:Config>() -> Vec<u16> { vec![] }
	#[pallet::storage]
	pub(super) type Incentive<T:Config> = StorageMap< _, Identity, u16, Vec<u16>, ValueQuery, DefaultIncentive<T> >;

	#[pallet::type_value] 
	pub fn DefaultConsensus<T:Config>() -> Vec<u16> { vec![] }
	#[pallet::storage]
	pub(super) type Consensus<T:Config> = StorageMap< _, Identity, u16, Vec<u16>, ValueQuery, DefaultConsensus<T> >;

	#[pallet::type_value] 
	pub fn DefaultDividends<T: Config>() -> Vec<u16> { vec![] }
	#[pallet::storage]
	pub(super) type Dividends<T:Config> = StorageMap< _, Identity, u16, Vec<u16>, ValueQuery, DefaultDividends<T> >;

	#[pallet::type_value] 
	pub fn DefaultEmission<T:Config>() -> Vec<u64> { vec![] }
	#[pallet::storage]
	pub(super) type Emission<T:Config> = StorageMap< _, Identity, u16, Vec<u64>, ValueQuery, DefaultEmission<T> >;
	
	/// ===============
	/// ==== Events ===
	/// ===============
	#[pallet::event]
	pub enum Event<T: Config> {
		/// --- Event created when stake has been transfered from 
		/// the a coldkey account onto the hotkey staking account.
		StakeAdded(T::AccountId, u64),

		/// --- Event created when stake has been removed from 
		/// the hotkey staking account onto the coldkey account.
		StakeRemoved(T::AccountId, u64),

		/// ---- Event created when a caller successfully set's their weights on a subnetwork.
		WeightsSet(u16, u16),
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
		
		/// ---- Thrown when the caller attempts to set the weight keys
		/// and values but these vectors have different size.
		WeightVecNotEqualSize,

		/// ---- Thrown when the caller attempts to set weights with duplicate uids
		/// in the weight matrix.
		DuplicateUids,

		/// ---- Thrown when a caller attempts to set weight to at least one uid that
		/// does not exist in the metagraph.
		InvalidUid,

		/// ---- Thrown when the dispatch attempts to set weights on chain with fewer elements 
		/// than are allowed.
		NotSettingEnoughWeights,

		/// ---- Thrown when the dispatch attempts to set weights on chain with where the normalized
		/// max value is more than MaxAllowedMaxMinRatio.
		MaxAllowedMaxMinRatioExceeded,
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

		/// --- Sets the caller weights for the incentive mechanism. The call can be
		/// made from the hotkey account so is potentially insecure, however, the damage
		/// of changing weights is minimal if caught early. This function includes all the
		/// checks that the passed weights meet the requirements. Stored as u16s they represent
		/// rational values in the range [0,1] which sum to 1 and can be interpreted as
		/// probabilities. The specific weights determine how inflation propagates outward
		/// from this peer. 
		/// 
		/// Note: The 16 bit integers weights should represent 1.0 as the max u16.
		/// However, the function normalizes all integers to u16_max anyway. This means that if the sum of all
		/// elements is larger or smaller than the amount of elements * u16_max, all elements
		/// will be corrected for this deviation. 
		/// 
		/// # Args:
		/// 	* `origin`: (<T as frame_system::Config>Origin):
		/// 		- The caller, a hotkey who wishes to set their weights.
		///
		/// 	* `netuid` (u16):
		/// 		- The network uid we are setting these weights on.
		/// 
		/// 	* `uids` (Vec<u16>):
		/// 		- The edge endpoint for the weight, i.e. j for w_ij.
		///
		/// 	* 'weights' (Vec<u16>):
		/// 		- The u16 integer encoded weights. Interpreted as rational
		/// 		values in the range [0,1]. They must sum to in32::MAX.
		///
		/// # Event:
		/// 	* WeightsSet;
		/// 		- On successfully setting the weights on chain.
		///
		/// # Raises:
		/// 	* 'WeightVecNotEqualSize':
		/// 		- If the passed weights and uids have unequal size.
		///
		/// 	* 'WeightSumToLarge':
		/// 		- When the calling coldkey is not associated with the hotkey account.
		///
		/// 	* 'InsufficientBalance':
		/// 		- When the amount to stake exceeds the amount of balance in the
		/// 		associated colkey account.
		///
        #[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn set_weights(
			_origin:OriginFor<T>, 
			_netuid: u16,
			_dests: Vec<u16>, 
			_weights: Vec<u16>
		) -> DispatchResult {
            Ok(())
			//Self::do_set_weights(origin, netuid, dests, weights)
		}

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
			_origin: OriginFor<T>, 
			_hotkey: T::AccountId, 
			_ammount_staked: u64
		) -> DispatchResult {
            Ok(())
			//Self::do_add_stake(origin, hotkey, ammount_staked)
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
			_origin: OriginFor<T>, 
			_hotkey: T::AccountId, 
			_ammount_unstaked: u64
		) -> DispatchResult {
            Ok(())
			//Self::do_remove_stake(origin, hotkey, ammount_unstaked)
		}
	}
}
