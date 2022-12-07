#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
use frame_system::{
	self as system,
	ensure_signed
};

use frame_support::{dispatch, ensure, traits::{
	Currency, 
	ExistenceRequirement,
	tokens::{
		WithdrawReasons
	}
}};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// ************************************************************
///	-Paratensor-Imports
/// ************************************************************
mod registration;
mod epoch;
mod utils;
mod staking;
mod weights;
mod networks;
mod serving;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::{DispatchResult, StorageMap, StorageValue, ValueQuery};
	use frame_support::{pallet_prelude::*, Identity, IterableStorageMap};
	use frame_system::{pallet_prelude::*};
	use frame_support::traits::{Currency, Get};
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

		/// Initial Emission Ratio
		#[pallet::constant]
		type InitialEmissionValue: Get<u16>;

		/// Initial max weight limit.
		#[pallet::constant]
		type InitialMaxWeightsLimit: Get<u16>;

		#[pallet::constant]
		type InitialMaxAllowedMaxMinRatio: Get<u16>;

		// Tempo for each network
		#[pallet::constant]
		type InitialTempo: Get<u16>;

		/// Initial Difficulty.
		#[pallet::constant]
		type InitialDifficulty: Get<u64>;

		/// Initial adjustment interval.
		#[pallet::constant]
		type InitialAdjustmentInterval: Get<u16>;

		/// Initial bonds moving average.
		#[pallet::constant]
		type InitialBondsMovingAverage: Get<u64>;

		/// Initial target registrations per interval.
		#[pallet::constant]
		type InitialTargetRegistrationsPerInterval: Get<u16>;

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

		// Initial pruning score for each neuron
		#[pallet::constant]
		type InitialPruningScore: Get<u16>;	
		
		/// Initial validator exclude quantile.
		#[pallet::constant]
		type InitialValidatorExcludeQuantile: Get<u16>;
	}

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type NeuronMetadataOf = NeuronMetadata;

	#[derive(Encode, Decode, Default, TypeInfo)]
    pub struct NeuronMetadata {

		/// ---- The endpoint's code version.
        pub version: u32,

        /// ---- The endpoint's u128 encoded ip address of type v6 or v4.
        pub ip: u128,

        /// ---- The endpoint's u16 encoded port.
        pub port: u16,

        /// ---- The endpoint's ip type, 4 for ipv4 and 6 for ipv6.
        pub ip_type: u8,
	}
	/// ===============================
	/// ==== Global Params Storage ====
	/// ===============================
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// ---- StorageItem Global Total Stake
	#[pallet::storage]
	pub type TotalStake<T> = StorageValue<_, u64, ValueQuery>;

	/// --- StorageItem Global Block Emission
	#[pallet::type_value]
	pub fn DefaultBlockEmission<T: Config>() -> u64 {1000000000}
	#[pallet::storage]
	pub type BlockEmission<T> = StorageValue<_, u64, ValueQuery, DefaultBlockEmission<T>>;

	/// ---- Total number of Existing Networks
	#[pallet::storage]
	pub type TotalNetworks<T> = StorageValue<_, u16, ValueQuery>;

	/// --- SingleMap Network UID -> if network is added
	#[pallet::type_value]
	pub fn DefaultNeworksAdded<T: Config>() ->  bool { false }
	#[pallet::storage]
	pub(super) type NetworksAdded<T:Config> = StorageMap<_, Identity, u16, bool, ValueQuery, DefaultNeworksAdded<T>>;

	/// --- SingleMap Network UID -> Pending Emission
	#[pallet::type_value]
	pub fn DefaultPendingEmission<T: Config>() ->  u64 { 0 }
	#[pallet::storage]
	pub(super) type PendingEmission<T:Config> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultPendingEmission<T>>;

	/// ---- StorageItem Hotkey --> Global Stake
	#[pallet::type_value] 
	pub fn DefaultTotalIssuance<T: Config>() -> u64 { T::InitialIssuance::get() }
	#[pallet::storage]
	pub type TotalIssuance<T> = StorageValue<_, u64, ValueQuery, DefaultTotalIssuance<T>>;

	/// ---- SingleMap Network UID --> EmissionValues
	#[pallet::type_value]
	pub fn DefaultEmissionValues<T: Config>() ->  u64 { 0}
	#[pallet::storage]
	pub(super) type EmissionValues<T:Config> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultEmissionValues<T>>;

	/// ---- StorageItem Global Max Registration Per Block
	#[pallet::type_value] 
	pub fn DefaultMaxRegistrationsPerBlock<T: Config>() -> u16 { T::InitialMaxRegistrationsPerBlock::get() }
	#[pallet::storage]
	pub type MaxRegistrationsPerBlock<T> = StorageValue<_, u16, ValueQuery, DefaultMaxRegistrationsPerBlock<T> >;

	/// ----  SingleMap Network UID --> Registration this Block
	#[pallet::type_value]
	pub fn DefaultRegistrationsThisBlock<T: Config>() ->  u16 { 0}
	#[pallet::storage]
	pub type RegistrationsThisBlock<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultRegistrationsThisBlock<T>>;

	/// ---- StorageItem Global Used Work
	#[pallet::storage]
	#[pallet::getter(fn usedwork)]
    pub(super) type UsedWork<T:Config> = StorageMap<_, Identity, Vec<u8>, u64, ValueQuery>;

	#[pallet::type_value] 
	pub fn DefaultBlocksSinceLastStep<T: Config>() -> u64 { 0 }
	#[pallet::storage]
	pub type BlocksSinceLastStep<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBlocksSinceLastStep<T>>;

	#[pallet::storage]
	pub type LastMechansimStepBlock<T> = StorageValue<_, u64, ValueQuery>;

	/// ---- SingleMap Network UID --> validator Exclude Quantile
	#[pallet::type_value]
	pub fn DefaultValidatorExcludeQuantile<T: Config>() -> u16 {T::InitialValidatorExcludeQuantile::get()}
	#[pallet::storage]
	pub type ValidatorExcludeQuantile<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorExcludeQuantile<T> >;
	//	
	/// ---- SingleMap Neuron UID --> Neuron Metadata (veriosn, ip address, port, ip type)
	#[pallet::storage]
	#[pallet::getter(fn uid)]
	pub(super) type NeuronsMetaData<T:Config> = StorageMap<_, Identity, u16, NeuronMetadataOf, OptionQuery>;

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

	/// --- SingleMap Hotkey --> A Vector of Network UIDs // a list of subnets that each hotkey is registered on
	#[pallet::type_value] 
	pub fn DefaultHotkeys<T:Config>() -> Vec<u16> { vec![] }
	#[pallet::storage]
	pub(super) type Subnets<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<u16>, ValueQuery, DefaultHotkeys<T> >;

	/// ---- DoubleMap Network UID --> neuron UID --> last_update
	#[pallet::type_value] 
	pub fn DefaultLastUpdate<T:Config>() -> u64 { 0 }
	#[pallet::storage]
    pub(super) type LastUpdate<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, u64 , ValueQuery, DefaultLastUpdate<T> >;

	/// =======================================
	/// ==== Subnetwork Hyperparam stroage  ====
	/// =======================================
	
	/// ---- SingleMap Network UID --> Hyper-parameter MinAllowedWeights
	#[pallet::type_value] 
	pub fn DefaultMinAllowedWeights<T: Config>() -> u16 { T::InitialMinAllowedWeights::get() }
	#[pallet::storage]
	pub type MinAllowedWeights<T> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultMinAllowedWeights<T> >;

	/// ---- SingleMap Network UID -->  Adjustment Interval
	#[pallet::type_value]
	pub fn DefaultAdjustmentInterval<T: Config>() -> u16 { T::InitialAdjustmentInterval::get() }
	#[pallet::storage]
	pub type AdjustmentInterval<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultAdjustmentInterval<T> >;

	/// ---- SingleMap Network UID -->  Bonds Moving Average
	#[pallet::type_value]
	pub fn DefaultBondsMovingAverage<T: Config>() -> u64 { T::InitialBondsMovingAverage::get() }
	#[pallet::storage]
	pub type BondsMovingAverage<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBondsMovingAverage<T> >;

	/// ---- SingleMap Network UID -->  Target Registration Per Interval
	#[pallet::type_value] 
	pub fn DefaultTargetRegistrationsPerInterval<T: Config>() -> u16 { T::InitialTargetRegistrationsPerInterval::get() }
	#[pallet::storage]
	pub type TargetRegistrationsPerInterval<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultTargetRegistrationsPerInterval<T> >;

	/// ---- SingleMap Network UID --> Hyper-parameter MaxWeightsLimit
	#[pallet::type_value] 
	pub fn DefaultMaxWeightsLimit<T: Config>() -> u16 { T::InitialMaxWeightsLimit::get() }
	#[pallet::storage]
	pub type MaxWeightsLimit<T> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultMaxWeightsLimit<T> >;

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
	pub type Rho<T> =  StorageMap<_, Identity, u16, u16, ValueQuery, DefaultRho<T> >;

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

	//// ---- DoubleMap Network UID --> Neuron UID --> Block Registration
	#[pallet::type_value] 
	pub fn DefaultBlockAtRegistration<T: Config>() -> u64 { 0 }
	
	#[pallet::storage]
	#[pallet::getter(fn block_at_registration)]
	pub(super) type BlockAtRegistration<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, u64, ValueQuery, DefaultBlockAtRegistration<T> >;

	/// =======================================
	/// ==== Subnetwork Consensus Storage  ====
	/// =======================================
	/// --- SingleMap Network UID --> SubNetwork Size (Number of UIDs in the network)
	#[pallet::type_value] 
	pub fn DefaultN<T:Config>() -> u16 { 0 }
	#[pallet::storage]
	pub(super) type SubnetworkN<T:Config> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultN<T> >;

	/// ---- SingleMap Network UID --> Modality   TEXT: 0, IMAGE: 1, TENSOR: 2
	#[pallet::type_value] 
	pub fn DefaultModality<T:Config>() -> u16 { 0 }
	#[pallet::storage]
	pub type NetworkModality<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultModality<T>> ;

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

	/// ---- DoubleMap Network UID --> Neuron UID --> is active
	#[pallet::type_value] 
	pub fn DefaultActive<T:Config>() -> bool { false }
	#[pallet::storage]
	pub(super) type Active<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, bool, ValueQuery, DefaultActive<T> >;

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

	/// ---- DoubleMap Network UID -->  Neuron UID --> Pruning Score
	#[pallet::type_value] 
	pub fn DefaultPruningScore<T: Config>() -> u16 { T::InitialPruningScore::get() }
	#[pallet::storage]
	pub(super) type PruningScores<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u16, ValueQuery, DefaultPruningScore<T> >;
	
	
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
		/// --- Event created when a new network is added
		NetworkAdded(u16, u16),

		/// --- Event created when a network is removed
		NetworkRemoved(u16),

		/// --- Event created when stake has been transfered from 
		/// the a coldkey account onto the hotkey staking account.
		StakeAdded(T::AccountId, u64),

		/// --- Event created when stake has been removed from 
		/// the hotkey staking account onto the coldkey account.
		StakeRemoved(T::AccountId, u64),

		/// ---- Event created when a caller successfully set's their weights on a subnetwork.
		WeightsSet(u16, u16),

		/// ---- Event created when Tempo is set
		TempoSet(u16),

		/// --- Event created when a new neuron account has been registered to 
		/// the chain.
		NeuronRegistered(u16),
		
		/// --- Event created when max allowed uids has been set for a subnetwor.
		MaxAllowedUidsSet(u16, u16),

		/// --- Event created when total stake increased
		TotalStakeIncreased(u64),

		/// --- Event created when the max weight limit has been set.
		MaxWeightLimitSet( u16, u16 ),

		/// --- Event created when the difficulty has been set for a subnet.
		DifficultySet(u16, u64),

		/// --- Event created when the adjustment interval is set for a subnet.
		AdjustmentIntervalSet(u16, u16),

		/// --- Event created when registeration per interval is set for a subnet.
		RegistrationPerIntervalSet(u16, u16),

		/// --- Event created when an activity cutoff is set for a subnet.
		ActivityCutoffSet(u16, u16),

		/// --- Event created when Rho value is set
		RhoSet(u16),

		/// --- Event created when kappa is set for a subnet.
		KappaSet(u16, u16),

		/// --- Event created when minimun aloowed weight is set for a subnet.
		MinAllowedWeightSet(u16, u16),

		/// --- Event created when max allowded max min ratio is set for a subnet.
		MaxAllowedMaxMinRatioSet(u16, u16),

		/// --- Event created when validator batch size is set for a subnet.
		ValidatorBatchSizeSet(u16, u16),

		/// --- Event created when validator sequence length i set for a subnet.
		ValidatorSequenceLengthSet(u16, u16),

		/// --- Event created when validator epoch per reset is set for a subnet.
		ValidatorEpochPerResetSet(u16, u16),

		/// --- Event created when incentive pruning denominator is set for a subnet.
		IncentivePruningDenominatorSet(u16, u16),

		/// --- Event created when stake pruning denominator is set for a subnet.
		StakePruningDenominatorSet(u16, u16),

		/// --- Event created when immunity period is set for a subnet
		ImmunityPeriodSet(u16, u16),

		/// --- Event created when bonds moving average is set for a subnet
		BondsMovingAverageSet(u16, u64),

		/// --- Event thrown when bonds have been reset for a subnet.
		ResetBonds(u16),

		/// --- Event created when the validator exclude quantile has been set for a subnet.
		ValidatorExcludeQuantileSet( u16, u16 ),

		/// --- Event created when the axon server information is added to the network.
		AxonServed(u16),

		/// --- Event created when emission ratios fr all networks is set
		EmissionValuesSet(),
	}
	
	/// ================
	/// ==== Errors ====
	/// ================
	#[pallet::error]
	pub enum Error<T> {
		/// Network does not exist
		NetworkDoesNotExist,

		/// Network already exist
		NetworkExist,

		/// --- Thrown when an invalid modality attempted on serve.
		InvalidModality,

		/// ---- Thrown when the user tries to serve an axon which is not of type
	    /// 4 (IPv4) or 6 (IPv6).
		InvalidIpType,

		/// --- Thrown when an invalid IP address is passed to the serve function.
		InvalidIpAddress,
		
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

		/// ---- Thrown when registrations this block exceeds allowed number.
		TooManyRegistrationsThisBlock,

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

		/// ---  Thrown if the vaule is invalid for MaxAllowedUids
		MaxAllowedUIdsNotAllowed,

		/// ---- Thrown when the dispatch attempts to convert between a u64 and T::balance 
		/// but the call fails.
		CouldNotConvertToBalance,

		/// --- thrown when the caller requests adding stake for a hotkey to the 
		/// total stake which already added
		StakeAlreadyAdded,

		/// ---- Thrown when the dispatch attempts to set weights on chain with where any normalized
		/// weight is more than MaxWeightLimit.
		MaxWeightExceeded,

		/// ---- Thrown when the caller attempts to set a storage value outside of its allowed range.
		StorageValueOutOfRange,

		// --- Thrown when tempo has not set
		TempoHasNotSet,

		// --- Thrown when tempo is not valid
		InvalidTempo,

		// --- Thrown when number or recieved emission rates does not match number of networks
		EmissionValuesDoesNotMatchNetworks,

		// --- Thrown when emission ratios are not valid (did not sum up to 10^9)
		InvalidEmissionValues,
	}

	/// ================
	/// ==== Hooks =====
	/// ================
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> { 
		/// ---- Called on the initialization of this pallet. (the order of on_finalize calls is determined in the runtime)
		///
		/// # Args:
		/// 	* 'n': (T::BlockNumber):
		/// 		- The number of the block we are initializing.
		fn on_initialize( _n: BlockNumberFor<T> ) -> Weight {
			/*TO DO:
			1. calculate pending emission for each network
			2. check if tempo % current_block ==0 for any network, then call epoch with pending emission for this network
			3. if tempo% current_block == 0 then check pending_emission for the network. if pending_emission for the network ==0, 
				we do not need to run epoch for the network */
				let current_block_number = Self::get_current_block_as_u64();

				/* Emissions per networks : net 1 ---> 100,000 ; net 2 --> 3000,000 ; .... ==> sum = 10^9 rao */
				for (netuid_i, _) in <SubnetworkN<T> as IterableStorageMap<u16, u16>>::iter(){ //we gonna distribute 10^9 rao
					let pending_emission = EmissionValues::<T>::get(netuid_i);
					PendingEmission::<T>::mutate(netuid_i, |val| *val += pending_emission);
				}
				for (netuid_i, tempo_i)  in <Tempo<T> as IterableStorageMap<u16, u16>>::iter() {

					if tempo_i as u64 % current_block_number == 0 { 
						let net_emission:u64 = PendingEmission::<T>::get(netuid_i);
						//
						// RUN EPOCH for this network
						Self::epoch(netuid_i, net_emission, true);
					} 
				}
			return 0; 
		}
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
		/// 	* `dests` (Vec<u16>):
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
			netuid: u16,
			dests: Vec<u16>, 
			weights: Vec<u16>
		) -> DispatchResult {
			Self::do_set_weights(origin, netuid, dests, weights)
		}

		/// --- Adds stake to a hotkey. The call is made from the
		/// coldkey account linked in the hotkey.
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

		/// ---- Serves or updates axon information for the neuron associated with the caller. If the caller
		/// already registered the metadata is updated. If the caller is not registered this call throws NotRegsitered.
		///
		/// # Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, a hotkey associated of the registered neuron.
		///
		/// 	* 'netuid' (u16):
		/// 		- The u16 network identifier.
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
			origin:OriginFor<T>, 
			netuid: u16,
			version: u32, 
			ip: u128, 
			port: u16, 
			ip_type: u8
		) -> DispatchResult {
			Self::do_serve_axon( origin, netuid, version, ip, port, ip_type ) 
		}
		/// ---- Registers a new neuron to the subnetwork. 
		///
		/// # Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, registration key as found in RegistrationKey::get(0);
		///
		/// 	* 'netuid' (u16):
		/// 		- The u16 network identifier.
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

		/// ---- SUDO ONLY FUNCTIONS ------------------------------------------------------------

		/// ---- Sudo set this networks emission values.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `emission_values` (u16, u64):
		/// 		- A vector of (netuid, emission values) tuples.
		/// 
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn sudo_set_emission_values (
			origin: OriginFor<T>,
			emission_values: Vec<(u16, u64)>
		) -> DispatchResult{
			Self::do_set_emission_values( origin,  emission_values)
		}

		/// ---- Sudo set this network's bonds moving average.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network we are setting the moving average on.
		///
		/// 	* `bonds_moving_average` (u16):
		/// 		- The bonds moving average.
		///
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_bonds_moving_average ( 
			origin:OriginFor<T>, 
			netuid: u16,
			bonds_moving_average: u64 
		) -> DispatchResult {  
			ensure_root( origin )?;
			BondsMovingAverage::<T>::insert( netuid, bonds_moving_average );
			Self::deposit_event( Event::BondsMovingAverageSet( netuid, bonds_moving_average ) );
			Ok(())
		}

		/// ---- Sudo set networks difficulty hyper parameters.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network we are setting the hyper parameter on.
		///
		/// 	* `difficulty` (u16):
		/// 		- The network POW difficulty.
		///
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_difficulty ( 
			origin:OriginFor<T>, 
			netuid: u16,
			difficulty: u64 
		) -> DispatchResult {
			ensure_root( origin )?;
			Difficulty::<T>::insert( netuid, difficulty );
			Self::deposit_event( Event::DifficultySet( netuid, difficulty ) );
			Ok(())
		}

		/// ---- Sudo set the POW adjustment interval for this network.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set the adjustment interval.
		///
		/// 	* `adjustment_interval` (u16):
		/// 		- The network POW adjustment interval.
		///
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_adjustment_interval ( 
			origin:OriginFor<T>, 
			netuid: u16,
			adjustment_interval: u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			AdjustmentInterval::<T>::insert(netuid, adjustment_interval);
			Self::deposit_event( Event::AdjustmentIntervalSet( netuid, adjustment_interval) );
			Ok(()) 
		}

		/// ---- Sudo set the target registrations per interval for this network.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set the adjustment interval.
		///
		/// 	* `target_registrations_per_interval` (u16):
		/// 		- The network POW target registrations per interval
		///
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_target_registrations_per_interval ( 
			origin:OriginFor<T>, 
			netuid: u16,
			target_registrations_per_interval: u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			TargetRegistrationsPerInterval::<T>::insert(netuid, target_registrations_per_interval);
			Self::deposit_event( Event::RegistrationPerIntervalSet( netuid, target_registrations_per_interval) );
			Ok(())
		}
		
		/// ---- Sudo set the activity for this network.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set the adjustment interval.
		///
		/// 	* `target_registrations_per_interval` (u16):
		/// 		- The network POW target registrations per interval
		///
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_activity_cutoff ( 
			origin:OriginFor<T>, 
			netuid: u16,
			activity_cutoff: u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			ActivityCutoff::<T>::insert(netuid, activity_cutoff);
			Self::deposit_event( Event::ActivityCutoffSet( netuid, activity_cutoff) );
			Ok(())
		}

		/// ---- Sudo set rho.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set rho.
		///
		/// 	* `rho` (u16):
		/// 		- The network rho value.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_rho ( 
			origin:OriginFor<T>, 
			netuid: u16,
			rho: u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			Rho::<T>::insert(netuid, rho);
			Self::deposit_event( Event::RhoSet( rho ) );
			Ok(())
		}

		/// ---- Sudo set kappa.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set kappa on.
		///
		/// 	* `kappa` (u16):
		/// 		- The network kappa value.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_kappa ( 
			origin:OriginFor<T>, 
			netuid: u16,
			kappa: u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			Kappa::<T>::insert(netuid, kappa);
			Self::deposit_event( Event::KappaSet( netuid, kappa) );
			Ok(())
		}

		/// ---- Sudo set max allowed uids.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set max_allowed_uids on.
		///
		/// 	* `max_allowed_uids` (u16):
		/// 		- The network max_allowed_uids hyper-parameter.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_max_allowed_uids ( 
			origin:OriginFor<T>,
			netuid: u16, 
			max_allowed_uids: u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			ensure!( max_allowed_uids < u16::MAX, Error::<T>::MaxAllowedUIdsNotAllowed );
			MaxAllowedUids::<T>::insert(netuid, max_allowed_uids);
			Self::deposit_event( Event::MaxAllowedUidsSet( netuid, max_allowed_uids) );
			Ok(())
		}

		/// ---- Sudo set min_allowed_weights.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set min_allowed_weights  on.
		///
		/// 	* `min_allowed_weights` (u16):
		/// 		- The network min_allowed_weights  hyper-parameter.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_min_allowed_weights ( 
			origin:OriginFor<T>,
			netuid: u16, 
			min_allowed_weights: u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			MinAllowedWeights::<T>::insert(netuid, min_allowed_weights);
			Self::deposit_event( Event::MinAllowedWeightSet( netuid, min_allowed_weights) );
			Ok(())
		}

		/// ---- Sudo set max_allowed_max_min_ratio.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set max_allowed_max_min_ratio  on.
		///
		/// 	* `max_allowed_max_min_ratio` (u16):
		/// 		- The network max_allowed_max_min_ratio hyper-parameter.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_max_allowed_max_min_ratio ( 
			origin:OriginFor<T>, 
			netuid: u16,
			max_allowed_max_min_ratio: u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			MaxAllowedMaxMinRatio::<T>::insert(netuid, max_allowed_max_min_ratio);
			Self::deposit_event( Event::MaxAllowedMaxMinRatioSet( netuid, max_allowed_max_min_ratio) );
			Ok(())
		}

		/// ---- Sudo set validator_batch_size.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set validator_batch_size on.
		///
		/// 	* `validator_batch_size` (u16):
		/// 		- The network validator_batch_size hyper-parameter.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_validator_batch_size ( 
			origin:OriginFor<T>, 
			netuid: u16,
			validator_batch_size: u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			ValidatorBatchSize::<T>::insert(netuid, validator_batch_size);
			Self::deposit_event(Event::ValidatorBatchSizeSet(netuid, validator_batch_size));
			Ok(())
		}

		/// ---- Sudo set validator_sequence_length.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set validator_sequence_length on.
		///
		/// 	* `validator_sequence_length` (u16):
		/// 		- The network validator_sequence_length hyper-parameter.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_validator_sequence_length ( 
			origin:OriginFor<T>, 
			netuid: u16,
			validator_sequence_length: u16 
		) -> DispatchResult {
			ensure_root( origin )?; 
			ValidatorSequenceLength::<T>::insert(netuid, validator_sequence_length);
			Self::deposit_event(Event::ValidatorSequenceLengthSet(netuid, validator_sequence_length));
			Ok(())
		}

		/// ---- Sudo set validator_epochs_per_reset.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set validator_epochs_per_reset on.
		///
		/// 	* `validator_epochs_per_reset` (u16):
		/// 		- The network validator_epochs_per_reset hyper-parameter.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_validator_epochs_per_reset ( 
			origin:OriginFor<T>, 
			netuid: u16,
			validator_epochs_per_reset : u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			ValidatorEpochsPerReset::<T>::insert(netuid, validator_epochs_per_reset);
			Self::deposit_event(Event::ValidatorEpochPerResetSet(netuid, validator_epochs_per_reset));
			Ok(())
		}

		/// ---- Sudo set incentive_pruning_denominator.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set incentive_pruning_denominator on.
		///
		/// 	* `incentive_pruning_denominator` (u16):
		/// 		- The network incentive_pruning_denominator hyper-parameter.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_incentive_pruning_denominator( 
			origin:OriginFor<T>, 
			netuid: u16,
			incentive_pruning_denominator: u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			IncentivePruningDenominator::<T>::insert(netuid, incentive_pruning_denominator);
			Self::deposit_event(Event::IncentivePruningDenominatorSet(netuid, incentive_pruning_denominator));
			Ok(())
		}

		/// ---- Sudo set stake_pruning_denominator.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set stake_pruning_denominator on.
		///
		/// 	* `stake_pruning_denominator ` (u16):
		/// 		- The network stake_pruning_denominator hyper-parameter.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_stake_pruning_denominator( 
			origin:OriginFor<T>, 
			netuid: u16,
			stake_pruning_denominator: u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			StakePruningDenominator::<T>::insert(netuid, stake_pruning_denominator);
			Self::deposit_event(Event::StakePruningDenominatorSet(netuid, stake_pruning_denominator));
			Ok(())
		}

		/// ---- Sudo set immunity_period.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set immunity_period on.
		///
		/// 	* `immunity_period ` (u16):
		/// 		- The network immunity_period hyper-parameter.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_immunity_period ( 
			origin:OriginFor<T>, 
			netuid: u16,
			immunity_period: u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			ImmunityPeriod::<T>::insert(netuid, immunity_period);
			Self::deposit_event(Event::ImmunityPeriodSet(netuid, immunity_period));
			Ok(())
		}

		/// ---- Sudo set max_weight_limit.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set max_weight_limit on.
		///
		/// 	* `max_weight_limit ` (u16):
		/// 		- The network max_weight_limit hyper-parameter.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_max_weight_limit ( 
			origin:OriginFor<T>,
			netuid: u16, 
			max_weight_limit: u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			MaxWeightsLimit::<T>::insert( netuid, max_weight_limit );
			Self::deposit_event( Event::MaxWeightLimitSet( netuid, max_weight_limit ) );
			Ok(())
		}
		/*TO DO: impl reset_bonds in epoch,  
		sudo_set_validator_exclude_quantile function  */ 
		
		// --- SUDO functions to manage NETWORKS ---

		/// ---- Sudo set validator_exclude_quantile.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network id to set validator_exclude_quantile on.
		///
		/// 	* `validator_exclude_quantile ` (u16):
		/// 		- The network validator_exclude_quantile hyper-parameter.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_validator_exclude_quantile( 
			origin:OriginFor<T>, 
			netuid: u16,
			validator_exclude_quantile: u16 
		) -> DispatchResult {
			ensure_root( origin )?;
			ensure!( validator_exclude_quantile <= 100, Error::<T>::StorageValueOutOfRange ); // The quantile must be between 0 and 100 => 0% and 100%
		    ValidatorExcludeQuantile::<T>::insert(netuid, validator_exclude_quantile );
			Self::deposit_event( Event::ValidatorExcludeQuantileSet( netuid, validator_exclude_quantile ));
			Ok(())
		}

		/*TO DO: impl reset_bonds in epoch,  
		sudo_set_validator_exclude_quantile function  */ 


		/// ---- Sudo reset bonds on a network.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network to reset bonds on.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_reset_bonds (
			origin: OriginFor<T>,
			netuid: u16
		)-> DispatchResult {
			ensure_root( origin )?;
			// TODO (const) This function should be implemented
			// Self::reset_bonds(netuid);
			Self::deposit_event( Event::ResetBonds(netuid) );
			Ok(())
		}

		/// ---- Sudo add a network to the network set.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network uid to create.
		///
		/// 	* `modality` (u8):
		/// 		- The network modality identifier.
		///	
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_add_network (
			origin: OriginFor<T>,
			netuid: u16,
			tempo: u16,
			modality: u16
		)-> DispatchResult {
			Self::do_add_network(origin, netuid, tempo, modality)
		}

		/// ---- Sudo remove a network from the network set.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network uid to remove.
		///
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_remove_network (
			origin: OriginFor<T>,
			netuid: u16
		) -> DispatchResult {
			Self::do_remove_network(origin, netuid)
		} 
	}	
}
