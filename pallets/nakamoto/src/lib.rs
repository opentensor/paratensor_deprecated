#![cfg_attr(not(feature = "std"), no_std)]

/// ===============================
/// ==== Yuma Consensus Pallet ====
/// ===============================
pub use pallet::*;
use frame_support::{dispatch, ensure};
use frame_system::ensure_signed;

mod weights;
mod utils;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::inherent::Vec;
	use frame_support::sp_std::vec;

	/// ================
	/// ==== Config ====
	/// ================
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Hyperparams.
		#[pallet::constant]
		type NakamotoInitialMinAllowedWeights: Get<u16>;
		#[pallet::constant]
		type NakamotoInitialMaxAllowedMaxMinRatio: Get<u16>;

	}

	/// =================
	/// ==== Storage ====
	/// =================
	#[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// =============================
	/// ==== Hyperparam stroage  ====
	/// ============================)
	#[pallet::type_value] 
	pub fn DefaultMinAllowedWeights<T: Config>() -> u16 { T::NakamotoInitialMinAllowedWeights::get() }
	#[pallet::storage]
	pub type MinAllowedWeights<T> = StorageValue<_, u16, ValueQuery, DefaultMinAllowedWeights<T> >;

	#[pallet::type_value] 
	pub fn DefaultMaxAllowedMaxMinRatio<T: Config>() -> u16 { T::NakamotoInitialMaxAllowedMaxMinRatio::get() }
	#[pallet::storage]
	pub type MaxAllowedMaxMinRatio<T> = StorageValue<_, u16, ValueQuery, DefaultMaxAllowedMaxMinRatio<T> >;


	/// ============================
	/// ==== Consensus Storage  ====
	/// ============================
	#[pallet::type_value] 
	pub fn DefaultN<T:Config>() -> u16 { 0 }
	#[pallet::storage]
	#[pallet::getter(fn n)]
	pub(super) type N<T:Config> = StorageValue<	_, u16, ValueQuery, DefaultN<T> >;

	#[pallet::type_value] 
	pub fn DefaultUid<T:Config>() -> u16 { u16::MAX }
	#[pallet::storage]
	pub(super) type Uids<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u16, ValueQuery, DefaultUid<T> >;

	#[pallet::type_value] 
	pub fn DefaultColdkeyAccount<T: Config>() -> T::AccountId { T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap() }
	#[pallet::storage]
	#[pallet::getter(fn coldkey)]
    pub(super) type Coldkeys<T:Config> = StorageMap<_, Identity, u16, T::AccountId, ValueQuery, DefaultColdkeyAccount<T> >;

	#[pallet::type_value] 
	pub fn DefaultHotkeyAccount<T: Config>() -> T::AccountId { T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap() }
	#[pallet::storage]
	#[pallet::getter(fn uids)]
	pub(super) type Hotkeys<T:Config> = StorageMap<_, Identity, u16, T::AccountId, ValueQuery, DefaultHotkeyAccount<T> >;

	#[pallet::type_value] 
	pub fn DefaultWeights<T:Config>() -> Vec<(u16, u16)> { vec![] }
	#[pallet::storage]
    pub(super) type Weights<T:Config> = StorageMap<_, Identity, u16, Vec<(u16, u16)>, ValueQuery, DefaultWeights<T> >;

	#[pallet::type_value] 
	pub fn DefaultBonds<T:Config>() -> Vec<(u16, u16)> { vec![] }
	#[pallet::storage]
    pub(super) type Bonds<T:Config> = StorageMap<_, Identity, u16, Vec<(u16, u16)>, ValueQuery, DefaultBonds<T> >;

	#[pallet::type_value] 
	pub fn DefaultActive<T:Config>() -> Vec<bool> { vec![] }
	#[pallet::storage]
	#[pallet::getter(fn active)]
	pub(super) type Active<T:Config> = StorageValue< _, Vec<bool>, ValueQuery, DefaultActive<T> >;

	#[pallet::type_value] 
	pub fn DefaultStake<T:Config>() -> Vec<u64> { vec![] }
	#[pallet::storage]
    #[pallet::getter(fn stake)]
    pub(super) type Stake<T:Config> = StorageValue<	_, Vec<u64>, ValueQuery, DefaultStake<T> >;

	#[pallet::type_value] 
	pub fn DefaultRank<T:Config>() -> Vec<u16> { vec![] }
	#[pallet::storage]
	#[pallet::getter(fn rank)]
	pub(super) type Rank<T:Config> = StorageValue<	_, Vec<u16>, ValueQuery, DefaultRank<T> >;

	#[pallet::type_value] 
	pub fn DefaultTrust<T:Config>() -> Vec<u16> { vec![] }
	#[pallet::storage]
	#[pallet::getter(fn trust)]
	pub(super) type Trust<T:Config> = StorageValue<	_, Vec<u16>, ValueQuery, DefaultTrust<T> >;

	#[pallet::type_value] 
	pub fn DefaultIncentive<T:Config>() -> Vec<u16> { vec![] }
	#[pallet::storage]
	#[pallet::getter(fn incentive)]
	pub(super) type Incentive<T:Config> = StorageValue<	_, Vec<u16>, ValueQuery, DefaultIncentive<T> >;

	#[pallet::type_value] 
	pub fn DefaultConsensus<T:Config>() -> Vec<u16> { vec![] }
	#[pallet::storage]
	#[pallet::getter(fn consensus)]
	pub(super) type Consensus<T:Config> = StorageValue<	_, Vec<u16>, ValueQuery, DefaultConsensus<T> >;

	#[pallet::type_value] 
	pub fn DefaultDividends<T: Config>() -> Vec<u16> { vec![] }
	#[pallet::storage]
	#[pallet::getter(fn dividends)]
	pub(super) type Dividends<T:Config> = StorageValue<	_, Vec<u16>, ValueQuery, DefaultDividends<T> >;

	#[pallet::type_value] 
	pub fn DefaultEmission<T:Config>() -> Vec<u64> { vec![] }
	#[pallet::storage]
	#[pallet::getter(fn emission)]
	pub(super) type Emission<T:Config> = StorageValue<	_, Vec<u64>, ValueQuery, DefaultEmission<T> >;
	
	/// ===============
	/// ==== Events ===
	/// ===============
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// ---- Event created when a caller successfully set's their weights on the chain.
		WeightsSet(T::AccountId),
	}

	/// ================
	/// ==== Errors ====
	/// ================	
	#[pallet::error]
	pub enum Error<T> {

		/// ---- Thrown when the caller requests setting or removing data from
		/// a neuron which does not exist in the active set.
		NotRegistered,
		
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
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

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
			origin:OriginFor<T>, 
			dests: Vec<u16>, 
			weights: Vec<u16>
		) -> DispatchResult {
			Self::do_set_weights(origin, dests, weights)
		}
	}
}
