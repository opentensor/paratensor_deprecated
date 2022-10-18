#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
use frame_system::{
	self as system,
	ensure_signed
};

use frame_support::{dispatch, ensure, traits::{
	Currency, 
	ExistenceRequirement,
	IsSubType, 
	tokens::{
		WithdrawReasons
	}
}, weights::{
	//DispatchInfo, 
	//PostDispatchInfo
}
};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// ************************************************************
///	-Paratensor-Imports
/// ************************************************************
mod registration;
mod epoch;
mod utils;
mod staking;

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

		#[pallet::constant]
		type InitialGlobalN: Get<u16>;

		#[pallet::constant]
		type InitialBlocksPerStep: Get<u64>;

		/// --- Hyperparams
		#[pallet::constant]
		type InitialMinAllowedWeights: Get<u16>;

		#[pallet::constant]
		type InitialMaxAllowedMaxMinRatio: Get<u16>;

		// Tempo for each network that multiplies in blockPerStep and sets a different blocksPerStep for each network
		#[pallet::constant]
		type InitialTempo: Get<u16>;

		/// Initial Difficulty.
		#[pallet::constant]
		type InitialDifficulty: Get<u64>;

		/// Initial adjustment interval.
		#[pallet::constant]
		type InitialAdjustmentInterval: Get<u64>;

		/// Initial target registrations per interval.
		#[pallet::constant]
		type InitialTargetRegistrationsPerInterval: Get<u64>;

		/// Rho constant
		#[pallet::constant]
		type InitialRho: Get<u16>;

		/// Kappa constant
		#[pallet::constant]
		type InitialKappa: Get<u16>;

		/// Max UID constant.
		#[pallet::constant]
		type InitialMaxAllowedUids: Get<u16>;

		/// Default Batch size.
		#[pallet::constant]
		type InitialValidatorBatchSize: Get<u16>;

		/// Default Batch size.
		#[pallet::constant]
		type InitialValidatorSequenceLen: Get<u16>;

		/// Default Epoch length.
		#[pallet::constant]
		type InitialValidatorEpochLen: Get<u16>;

		/// Default Reset length.
		#[pallet::constant]
		type InitialValidatorEpochsPerReset: Get<u16>;

		/// Initial incentive pruning denominator
		#[pallet::constant]
		type InitialIncentivePruningDenominator: Get<u16>;

		/// Initial stake pruning denominator
		#[pallet::constant]
		type InitialStakePruningDenominator: Get<u16>;

		/// Initial stake pruning min
		#[pallet::constant]
		type InitialStakePruningMin: Get<u16>;

		/// Immunity Period Constant.
		#[pallet::constant]
		type InitialImmunityPeriod: Get<u16>;

		/// Activity constant
		#[pallet::constant]
		type InitialActivityCutoff: Get<u16>;

		/// Initial max registrations per block.
		#[pallet::constant]
		type InitialMaxRegistrationsPerBlock: Get<u16>;

		// Initial prunning score for each neuron
		#[pallet::constant]
		type InitialPrunningScore: Get<u16>;		
	}

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	//pub type NeuronMetadataOf<T> = NeuronMetadata<AccountIdOf<T>>;

	/// ===============================
	/// ==== Global Params Storage ====
	/// ===============================
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// ---- StorageItem Global Total N
	#[pallet::type_value] 
	pub fn DefaultGlobalN<T: Config>() -> u16 { T::InitialGlobalN::get() }
	#[pallet::storage]
	pub type GlobalN<T> = StorageValue<_, u16, ValueQuery>;

	/// ---- StorageItem Global Total Stake
	#[pallet::storage]
	pub type TotalStake<T> = StorageValue<_, u64, ValueQuery>;

	/// ---- StorageItem Hotkey --> Global Stake
	#[pallet::type_value] 
	pub fn DefaultTotalIssuance<T: Config>() -> u64 { T::InitialIssuance::get() }
	#[pallet::storage]
	pub type TotalIssuance<T> = StorageValue<_, u64, ValueQuery, DefaultTotalIssuance<T>>;

	/// ---- StorageItem BlocksPerSteps
	#[pallet::type_value]
	pub fn DefaultBlocksPerStep<T: Config>() -> u64 {T::InitialBlocksPerStep::get()}
	#[pallet::storage]
	pub type BlocksPerStep<T> = StorageValue<_, u64, ValueQuery, DefaultBlocksPerStep<T>>; 

	/// ---- SingleMap Network UID --> EmissionRatio
	#[pallet::type_value]
	pub fn DefaultEmissionRatio<T: Config>() ->  u16 { 0}
	#[pallet::storage]
	pub(super) type EmissionRatio<T:Config> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultEmissionRatio<T>>;

	/// ---- Maps from uid to neuron.
	//#[pallet::storage]
    //#[pallet::getter(fn uid)]
    //pub(super) type Neurons<T:Config> = StorageMap<_, Identity, u64, NeuronMetadataOf<T>, OptionQuery>;

	/// ---- StorageItem Global Adjustment Interval
	#[pallet::type_value]
	pub fn DefaultAdjustmentInterval<T: Config>() -> u64 { T::InitialAdjustmentInterval::get() }
	#[pallet::storage]
	pub type AdjustmentInterval<T> = StorageValue<_, u64, ValueQuery, DefaultAdjustmentInterval<T> >;

	#[pallet::type_value] 
	pub fn DefaultTargetRegistrationsPerInterval<T: Config>() -> u64 { T::InitialTargetRegistrationsPerInterval::get() }
	#[pallet::storage]
	pub type TargetRegistrationsPerInterval<T> = StorageValue<_, u64, ValueQuery, DefaultTargetRegistrationsPerInterval<T> >;

	/// ---- StorageItem Global Max Registration Per Block
	#[pallet::type_value] 
	pub fn DefaultMaxRegistrationsPerBlock<T: Config>() -> u16 { T::InitialMaxRegistrationsPerBlock::get() }
	#[pallet::storage]
	pub type MaxRegistrationsPerBlock<T> = StorageValue<_, u16, ValueQuery, DefaultMaxRegistrationsPerBlock<T> >;

	#[pallet::storage]
	pub type RegistrationsThisBlock<T> = StorageValue<_, u16, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn usedwork)]
    pub(super) type UsedWork<T:Config> = StorageMap<_, Identity, Vec<u8>, u64, ValueQuery>;

	#[pallet::type_value] 
	pub fn DefaultBlockAtRegistration<T: Config>() -> u64 { 0 }
	#[pallet::storage]
	#[pallet::getter(fn block_at_registration)]
    pub(super) type BlockAtRegistration<T:Config> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBlockAtRegistration<T> >;


	/// ==============================
	/// ==== Accounts Storage ====
	/// ==============================

	/// ---- SingleMap Hotkey --> Global Stake
	#[pallet::storage]
    pub(super) type Stake<T:Config> = StorageMap<_, Identity, T::AccountId, u64, ValueQuery>;

	/// ---- SingleMap Hotkey --> Coldkey
	#[pallet::type_value] 
	pub fn DefaultHotkeyAccount<T: Config>() -> T::AccountId { T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap()}
	#[pallet::storage]
    pub(super) type Coldkeys<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery, DefaultHotkeyAccount<T> >;

	/// ---- SingleMap Coldkey --> Hotkey
	#[pallet::type_value] 
	pub fn DefaultColdkeyAccount<T: Config>() -> T::AccountId { T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap()}
	#[pallet::storage]
	pub(super) type Hotkeys<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery, DefaultColdkeyAccount<T> >;

	/// --- SingleMap Hotkey --> Network UID // a list of subnets that each hotkey is registered on
	#[pallet::type_value] 
	pub fn DefaultHotkeys<T:Config>() -> Vec<u16> { vec![] }
	#[pallet::storage]
	pub(super) type Subnets<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<u16>, ValueQuery, DefaultHotkeys<T> >;

	/// =======================================
	/// ==== Subnetwork Hyperparam stroage  ====
	/// =======================================
	/// ---- SingleMap Network UID --> Hyper-parameter MinAllowedWeights
	#[pallet::type_value] 
	pub fn DefaultMinAllowedWeights<T: Config>() -> u16 { T::InitialMinAllowedWeights::get() }
	#[pallet::storage]
	pub type MinAllowedWeights<T> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultMinAllowedWeights<T> >;

	/// ---- SingleMap Network UID --> MaxAllowedMaxMinRatio
	/// TODO(const): should be moved to max clip ratio.
	#[pallet::type_value] 
	pub fn DefaultMaxAllowedMaxMinRatio<T: Config>() -> u16 { T::InitialMaxAllowedMaxMinRatio::get() }
	#[pallet::storage]
	pub type MaxAllowedMaxMinRatio<T> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultMaxAllowedMaxMinRatio<T> >;

	/// ---- SingleMap Network UID --> Tempo
	#[pallet::type_value]
	pub fn DefaultTempo<T: Config>() -> u16 {T::InitialTempo::get()}
	#[pallet::storage]
	pub type Tempo<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultTempo<T> >;

	/// ---- SingleMap Network UID --> Difficulty
	#[pallet::type_value]
	pub fn DefaultDifficulty<T: Config>() -> u64 {T::InitialDifficulty::get()}
	#[pallet::storage]
	pub type Difficulty<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultDifficulty<T> >;

	/// ---- SingleMap Network UID --> Rho
	#[pallet::type_value]
	pub fn DefaultRho<T: Config>() -> u16 {T::InitialRho::get()}
	#[pallet::storage]
	pub type Rho<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultRho<T> >;

	/// --- SingleMap Network UID ---> Kappa
	#[pallet::type_value]
	pub fn DefaultKappa<T: Config>() -> u16 {T::InitialKappa::get()}
	#[pallet::storage]
	pub type Kappa<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultKappa<T> >;

	/// --- SingleMap Network UID ---> Max Allowed Uids
	#[pallet::type_value] 
	pub fn DefaultMaxAllowedUids<T: Config>() -> u16 { T::InitialMaxAllowedUids::get() }
	#[pallet::storage]
	pub type MaxAllowedUids<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxAllowedUids<T> >;

	/// --- SingleMap Network UID ---> Validator Batch Size
	#[pallet::type_value] 
	pub fn DefaultValidatorBatchSize<T: Config>() -> u16 { T::InitialValidatorBatchSize::get() }
	#[pallet::storage]
	pub type ValidatorBatchSize<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorBatchSize<T> >;

	/// --- SingleMap Network UID ---> Validaotr Sequence Length
	#[pallet::type_value] 
	pub fn DefaultValidatorSequenceLen<T: Config>() -> u16 { T::InitialValidatorSequenceLen::get() }
	#[pallet::storage]
	pub type ValidatorSequenceLength<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorSequenceLen<T> >;

	/// --- SingleMap Network UID ---> Validator Epoch Length
	#[pallet::type_value] 
	pub fn DefaultValidatorEpochLen<T: Config>() -> u16 { T::InitialValidatorEpochLen::get() }
	#[pallet::storage]
	pub type ValidatorEpochLen<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorEpochLen<T> >; 

	/// ---- SingleMap Network UID ---> Valdiator Epochs Per Reset
	#[pallet::type_value] 
	pub fn DefaultValidatorEpochsPerReset<T: Config>() -> u16 { T::InitialValidatorEpochsPerReset::get() }
	#[pallet::storage]
	pub type ValidatorEpochsPerReset<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorEpochsPerReset<T> >;

	/// ---- SingleMap Network UID ---> Incentive Pruning Denominator
	#[pallet::type_value] 
	pub fn DefaultIncentivePruningDenominator<T: Config>() -> u16 { T::InitialIncentivePruningDenominator::get() }
	#[pallet::storage]
	pub type IncentivePruningDenominator<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultIncentivePruningDenominator<T> >;

	/// --- SingleMap Network UID ---> Stake Pruning Denominator
	#[pallet::type_value] 
	pub fn DefaultStakePruningDenominator<T: Config>() -> u16 { T::InitialStakePruningDenominator::get() }
	#[pallet::storage]
	pub type StakePruningDenominator<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultStakePruningDenominator<T> >;

	/// --- SingleMap Network UID ---> Stake Pruning Min
	#[pallet::type_value] 
	pub fn DefaultStakePruningMin<T: Config>() -> u16 { T::InitialStakePruningMin::get() }
	#[pallet::storage]
	pub type StakePruningMin<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultStakePruningMin<T> >;

	/// --- SingleMap Network UID ---> Immunity Period
	#[pallet::type_value] 
	pub fn DefaultImmunityPeriod<T: Config>() -> u16 { T::InitialImmunityPeriod::get() }
	#[pallet::storage]
	pub type ImmunityPeriod<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultImmunityPeriod<T> >;

	/// --- SingleMap Network UID --> Activity Cutoff
	#[pallet::type_value] 
	pub fn DefaultActivityCutoff<T: Config>() -> u16 { T::InitialActivityCutoff::get() }
	#[pallet::storage]
	pub type ActivityCutoff<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultActivityCutoff<T> >;

	/// ---- SingleMap Network UID --> Neuron UID, we use to record uids to prune at next epoch.
	#[pallet::storage]
	#[pallet::getter(fn uid_to_prune)]
    pub(super) type NeuronsToPruneAtNextEpoch<T:Config> = StorageMap<_, Identity, u16, u16, ValueQuery>;

	// ---- SingleMap Network UID --> Registration This Interval
	#[pallet::storage]
	pub type RegistrationsThisInterval<T:Config> = StorageMap<_, Identity, u16, u16, ValueQuery>;

	/// =======================================
	/// ==== Subnetwork Consensus Storage  ====
	/// =======================================
	/// --- SingleMap Network UID --> SubNetwork Size (Number of UIDs in the network)
	#[pallet::type_value] 
	pub fn DefaultN<T:Config>() -> u16 { 0 }
	#[pallet::storage]
	pub(super) type SubnetworkN<T:Config> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultN<T> >;

	/// ---- DoubleMap Network UID --> Neuron UID --> Hotkey
	#[pallet::type_value] 
	pub fn DefaultKey<T:Config>() -> T::AccountId { T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap() }
	#[pallet::storage]
	pub(super) type Keys<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, T::AccountId, ValueQuery, DefaultKey<T> >;

	/// ---- DoubleMap Network UID --> Hotkey --> Neuron UID
	#[pallet::type_value] 
	pub fn DefaultUid<T:Config>() -> u16 { 0 }
	#[pallet::storage]
	pub(super) type Uids<T:Config> = StorageDoubleMap<_, Identity, u16, Blake2_128Concat, T::AccountId, u16, ValueQuery, DefaultUid<T> >;

	/// ---- DoubleMap Network UID --> Neuron UID --> Row Weights
	#[pallet::type_value] 
	pub fn DefaultWeights<T:Config>() -> Vec<(u16, u16)> { vec![] }
	#[pallet::storage]
    pub(super) type Weights<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, Vec<(u16, u16)>, ValueQuery, DefaultWeights<T> >;

	/// ---- DoubleMap Network UID --> Neuron UID --> Row Bonds
	#[pallet::type_value] 
	pub fn DefaultBonds<T:Config>() -> Vec<(u16, u16)> { vec![] }
	#[pallet::storage]
    pub(super) type Bonds<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, Vec<(u16, u16)>, ValueQuery, DefaultBonds<T> >;

	/// ---- SingleMap Network UID --> Network Activity Vector
	#[pallet::type_value] 
	pub fn DefaultActive<T:Config>() -> Vec<bool> { vec![] }
	#[pallet::storage]
	pub(super) type Active<T:Config> = StorageMap< _, Identity, u16, Vec<bool>, ValueQuery, DefaultActive<T> >;

	/// ---- DoubleMap Network UID --> Neuron UID --> Neuron Stake
	#[pallet::type_value] 
	pub fn DefaultStake<T:Config>() -> u64 {0 }
	#[pallet::storage]
    pub(super) type S<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u64, ValueQuery, DefaultStake<T> >;

	/// ---- DoubleMap Network UID -->  Neuron UID --> Neuron Rank
	#[pallet::type_value] 
	pub fn DefaultRank<T:Config>() -> u16 {0 }
	#[pallet::storage]
	pub(super) type Rank<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u16, ValueQuery, DefaultRank<T> >;

	/// ---- DoubleMap Network UID --> Neuron UID --> Neuron Trust
	#[pallet::type_value] 
	pub fn DefaultTrust<T:Config>() -> u16 {0 }
	#[pallet::storage]
	pub(super) type Trust<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u16, ValueQuery, DefaultTrust<T> >;

	/// ---- DoubleMap Network UID --> Neuron UID --> Neuron Incentive
	#[pallet::type_value] 
	pub fn DefaultIncentive<T:Config>() -> u16 { 0}
	#[pallet::storage]
	pub(super) type Incentive<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u16, ValueQuery, DefaultIncentive<T> >;

	/// ---- DoubleMap Network UID --> Neuron UID --> Neuron Consensus
	#[pallet::type_value] 
	pub fn DefaultConsensus<T:Config>() -> u16 {0 }
	#[pallet::storage]
	pub(super) type Consensus<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u16, ValueQuery, DefaultConsensus<T> >;

	/// ---- DoubleMap Network UID --> Neuron UID --> Neuron Dividends
	#[pallet::type_value] 
	pub fn DefaultDividends<T: Config>() -> u16 {0 }
	#[pallet::storage]
	pub(super) type Dividends<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u16, ValueQuery, DefaultDividends<T> >;

	/// ---- DoubleMap Network UID --> Neuron UID --> Neuron Emission 
	#[pallet::type_value] 
	pub fn DefaultEmission<T:Config>() -> u64 {0 }
	#[pallet::storage]
	pub(super) type Emission<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u64, ValueQuery, DefaultEmission<T> >;

	/// ---- DoubleMap Network UID -->  Neuron UID --> Prunning Score
	#[pallet::type_value] 
	pub fn DefaultPrunningScore<T: Config>() -> u16 { T::InitialPrunningScore::get() }
	#[pallet::storage]
	pub(super) type PrunningScores<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u16, ValueQuery, DefaultPrunningScore<T> >;
	
	/// ************************************************************
	///	-Genesis-Configuration  
	/// ************************************************************
	/// ---- Genesis Configuration (Mostly used for testing.)
	/// TO DO (If we need it) 

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

		/// ---- Event created when a caller successfully set's their weights on a subnetwork.
		WeightsSet(u16, u16),

		/// ---- Event created when default blocks per step has been set.
		BlocksPerStepSet(u64),

		/// ---- Event created when Tempo is set
		TempoSet(u16),
		/* TO DO: more events for setting hyper parameters should be defined here */

		/// --- Event created when a new neuron account has been registered to 
		/// the chain.
		NeuronRegistered(u16),
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

		// --- Error for setting blocksPerStep
		
		// --- Error for setting Tempo /* TO DO: more Errors for different hyper parameters should be defined here */

		/// ---- Thrown when registrations this block exceeds allowed number.
		ToManyRegistrationsThisBlock,

		/// ---- Thrown when the caller requests registering a neuron which 
		/// already exists in the active set.
		AlreadyRegistered,

		/// ---- Thrown if the supplied pow hash block is in the future or negative
		InvalidWorkBlock,

		/// ---- Thrown when the caller attempts to use a repeated work.
		WorkRepeated,

		/// ---- Thrown if the supplied pow hash block does not meet the network difficulty.
		InvalidDifficulty,

		/// ---- Thrown if the supplied pow hash seal does not match the supplied work.
		InvalidSeal,
	}

	/// ================
	/// ==== Hooks =====
	/// ================
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> { /*TO DO */
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
            Ok(()) /*TO DO */
			//Self::do_remove_stake(origin, hotkey, ammount_unstaked)
		}

		/// ---- Serves or updates axon information for the neuron associated with the caller. If the caller
		/// already registered the metadata is updated. If the caller is not registered this call throws NotRegsitered.
		///
		/// # Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, a hotkey associated of the registered neuron.
		///
		/// 	* 'ip' (u128):
		/// 		- The u64 encoded IP address of type 6 or 4.
		///
		/// 	* 'port' (u16):
		/// 		- The port number where this neuron receives RPC requests.
		///
		/// 	* 'ip_type' (u8):
		/// 		- The ip type one of (4,6).
		///
		/// 	* 'modality' (u8):
		/// 		- The neuron modality type.
		///
		/// # Event:
		/// 	* 'AxonServed':
		/// 		- On subscription of a new neuron to the active set.
		///
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn serve_axon (
			_origin:OriginFor<T>, 
			_version: u32, 
			_ip: u128, 
			_port: u16, 
			_ip_type: u8, 
			_modality: u8 
		) -> DispatchResult {  /*TO DO */
			Ok(()) 
		}
		/// ---- Registers a new neuron to the subnetwork. 
		///
		/// # Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, registration key as found in RegistrationKey::get(0);
		///
		/// 	* 'block_number' (u64):
		/// 		- Block number of hash to attempt.
		///
		/// 	* 'nonce' (u64):
		/// 		- Hashing nonce as a u64.
		///
		/// 	* 'work' (Vec<u8>):
		/// 		- Work hash as list of bytes.
		/// 
		/// 	* 'hotkey' (T::AccountId,):
		/// 		- Hotkey to register.
		/// 
		/// 	* 'coldkey' (T::AccountId,):
		/// 		- Coldkey to register.
		/// 	* 'netuid' (u16):
		///			- subnetwork registering on
		/// # Event:
		/// 	* 'NeuronRegistered':
		/// 		- On subscription of a new neuron to the active set.
		///
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn register( 
				origin:OriginFor<T>, 
				netuid: u16,
				block_number: u64, 
				nonce: u64, 
				work: Vec<u8>,
				hotkey: T::AccountId, 
				coldkey: T::AccountId,
		) -> DispatchResult { 
				Self::do_registration(origin, netuid, block_number, nonce, work, hotkey, coldkey)
		}

		/// ---- SUDO ONLY FUNCTIONS ------
		/// Set blocks per Step
		/// #Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		/// 		
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn sudo_set_blocks_per_step(
			_origin: OriginFor<T>,
			_blocks_per_step: u64
		) -> DispatchResult { /*TO DO */
			Ok(())
		}

		/// ---- Set emission ratio for each subnetwork
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		/// 	* `netuid` (u16):
		/// 		- The network uid we are setting emission ratio on.
		/// 
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn sudo_set_emission_ratio (
			_origin: OriginFor<T>,
			_netuid: u16,
			_subnet_emission_ratio: u16
		) -> DispatchResult{
			ensure_root( _origin )?;
				if Self::calculate_emission_ratio_sum() + _subnet_emission_ratio > 1 { 
						 //we should return error /*To DO */
				}
				else{
					EmissionRatio::<T>::insert(_netuid, _subnet_emission_ratio);
				}
			Ok(())
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_bonds_moving_average ( 
			_origin:OriginFor<T>, 
			_bonds_moving_average: u64 
		) -> DispatchResult {  
			ensure_root( _origin )?; /*TO DO */
			Ok(())
		}

		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_difficulty ( 
			_origin:OriginFor<T>, 
			_difficulty: u64 
		) -> DispatchResult {
			ensure_root( _origin )?;
			Ok(())  /*TO DO */
		}

		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_adjustment_interval ( 
			_origin:OriginFor<T>, 
			_adjustment_interval: u64 
		) -> DispatchResult {
			ensure_root( _origin )?;
			Ok(())  /*TO DO */
		}

		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_target_registrations_per_interval ( 
			_origin:OriginFor<T>, 
			_target_registrations_per_interval: u64 
		) -> DispatchResult {
			ensure_root( _origin )?; /* TO DO */
			Ok(())
		}
		
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_activity_cutoff ( 
			_origin:OriginFor<T>, 
			_activity_cutoff: u16 
		) -> DispatchResult {
			ensure_root( _origin )?;/*TO DO */
			Ok(())
		}

		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_rho ( 
			_origin:OriginFor<T>, 
			_rho: u64 
		) -> DispatchResult {
			ensure_root( _origin )?; /*TO DO */
			Ok(())
		}

		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_kappa ( 
			_origin:OriginFor<T>, 
			_kappa: u16 
		) -> DispatchResult {
			ensure_root( _origin )?; /*TO DO */
			Ok(())
		}

		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_max_allowed_uids ( 
			_origin:OriginFor<T>, 
			_max_allowed_uids: u16 
		) -> DispatchResult {
			ensure_root( _origin )?; /*TO DO */
			Ok(())
		}

		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_min_allowed_weights ( 
			_origin:OriginFor<T>, 
			_min_allowed_weights: u64 
		) -> DispatchResult {
			ensure_root( _origin )?; /* TO DO */
			Ok(())
		}

		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_max_allowed_max_min_ratio ( 
			_origin:OriginFor<T>, 
			_max_allowed_max_min_ratio: u16 
		) -> DispatchResult {
			ensure_root( _origin )?; /*TO DO */
			Ok(())
		}

		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_validator_batch_size ( 
			_origin:OriginFor<T>, 
			_validator_batch_size: u16 
		) -> DispatchResult {
			ensure_root( _origin )?; /*TO DO */
			Ok(())
		}

		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_validator_sequence_length ( 
			_origin:OriginFor<T>, 
			_validator_sequence_length: u16 
		) -> DispatchResult {
			ensure_root( _origin )?;  /*TO DO */
			Ok(())
		}

		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_validator_epochs_per_reset ( 
			_origin:OriginFor<T>, 
			_validator_epochs_per_reset : u16 
		) -> DispatchResult {
			ensure_root( _origin )?; /*TO DO */
			Ok(())
		}

		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_incentive_pruning_denominator( 
			_origin:OriginFor<T>, 
			_incentive_pruning_denominator: u16 
		) -> DispatchResult {
			ensure_root( _origin )?; /*TO DO */
			Ok(())
		}

		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_stake_pruning_denominator( 
			_origin:OriginFor<T>, 
			_stake_pruning_denominator: u16 
		) -> DispatchResult {
			ensure_root( _origin )?; /*TO DO */
			Ok(())
		}

		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_immunity_period ( 
			_origin:OriginFor<T>, 
			_immunity_period: u16 
		) -> DispatchResult {
			ensure_root( _origin )?; /*TO DO */
			Ok(())
		}
		/*TO DO: reset_bonds function  */ 
	}
	/// ---- Paratensor helper functions.
	impl<T: Config> Pallet<T> {
	/// ---- returns the sum of emission ratios for defined subnetworks
		pub fn calculate_emission_ratio_sum() -> u16 {
			let sum : u16 = 0; /*TO DO */
			sum
		}
	}	
}
