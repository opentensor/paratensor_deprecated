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

/// =========================
///	==== Pallet Imports =====
/// =========================
mod registration;
mod epoch;
mod utils;
mod staking;
mod weights;
mod networks;
mod serving; 
mod block_step;

#[frame_support::pallet]
pub mod pallet {

	/// ========================
	/// ==== Pallet Imports ====
	/// ========================
	use frame_support::pallet_prelude::{DispatchResult, StorageMap, StorageValue, ValueQuery};
	use frame_support::{pallet_prelude::*, Identity};
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

		/// =================================
		/// ==== Initial Value Constants ====
		/// =================================
		#[pallet::constant] /// Initial currency issuance.
		type InitialIssuance: Get<u64>;
		#[pallet::constant] /// Initial min allowed weights setting.
		type InitialMinAllowedWeights: Get<u16>;
		#[pallet::constant] /// Initial Emission Ratio
		type InitialEmissionValue: Get<u16>;
		#[pallet::constant] /// Initial max weight limit.
		type InitialMaxWeightsLimit: Get<u16>;
		#[pallet::constant] /// Tempo for each network
		type InitialTempo: Get<u16>;
		#[pallet::constant] /// Initial Difficulty.
		type InitialDifficulty: Get<u64>;
		#[pallet::constant] /// Initial adjustment interval.
		type InitialAdjustmentInterval: Get<u16>;
		#[pallet::constant] /// Initial bonds moving average.
		type InitialBondsMovingAverage: Get<u64>;
		#[pallet::constant] /// Initial target registrations per interval.
		type InitialTargetRegistrationsPerInterval: Get<u16>;
		#[pallet::constant] /// Rho constant
		type InitialRho: Get<u16>;
		#[pallet::constant] /// Kappa constant
		type InitialKappa: Get<u16>;		
		#[pallet::constant] /// Max UID constant.
		type InitialMaxAllowedUids: Get<u16>;
		#[pallet::constant] /// Default Batch size.
		type InitialValidatorBatchSize: Get<u16>;
		#[pallet::constant] /// Default Batch size.
		type InitialValidatorSequenceLen: Get<u16>;
		#[pallet::constant] /// Default Epoch length.
		type InitialValidatorEpochLen: Get<u16>;
		#[pallet::constant] /// Default Reset length.
		type InitialValidatorEpochsPerReset: Get<u16>;
		#[pallet::constant] /// Initial stake pruning min
		type InitialStakePruningMin: Get<u16>;
		#[pallet::constant] /// Immunity Period Constant.
		type InitialImmunityPeriod: Get<u16>;
		#[pallet::constant] /// Activity constant
		type InitialActivityCutoff: Get<u16>;
		#[pallet::constant] /// Initial max registrations per block.
		type InitialMaxRegistrationsPerBlock: Get<u16>;
		#[pallet::constant] /// Initial pruning score for each neuron
		type InitialPruningScore: Get<u16>;	
		#[pallet::constant] /// Initial validator exclude quantile.
		type InitialValidatorExcludeQuantile: Get<u16>;
		#[pallet::constant] /// Initial allowed validators per network.
		type InitialMaxAllowedValidators: Get<u16>;
		#[pallet::constant] /// Initial default delegation take.
		type InitialDefaultTake: Get<u16>;
	}

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	/// =====================
	/// ==== Axon Struct ====
	/// =====================
	pub type AxonMetadataOf = AxonMetadata;
	#[derive(Encode, Decode, Default, TypeInfo)]
    pub struct AxonMetadata {
		/// ---- The endpoint's code version.
        pub version: u32,
        /// ---- The endpoint's u128 encoded ip address of type v6 or v4.
        pub ip: u128,
        /// ---- The endpoint's u16 encoded port.
        pub port: u16,
        /// ---- The endpoint's ip type, 4 for ipv4 and 6 for ipv6.
        pub ip_type: u8,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);
	#[pallet::type_value] 
	pub fn DefaultDebug<T: Config>() -> bool { true }
	#[pallet::storage] /// --- ITEM ( debug )
	pub type Debug<T> = StorageValue<_, bool, ValueQuery, DefaultDebug<T>>;

	/// ============================
	/// ==== Staking + Accounts ====
	/// ============================
	#[pallet::type_value] 
	pub fn DefaultDefaultTake<T: Config>() -> u16 { T::InitialDefaultTake::get() }
	#[pallet::type_value] 
	pub fn DefaultAccountTake<T: Config>() -> u64 { 0 }
	#[pallet::type_value]
	pub fn DefaultBlockEmission<T: Config>() -> u64 {1_000_000_000}
	#[pallet::type_value] 
	pub fn DefaultAllowsDelegation<T: Config>() -> bool { false }
	#[pallet::type_value] 
	pub fn DefaultTotalIssuance<T: Config>() -> u64 { T::InitialIssuance::get() }
	#[pallet::type_value] 
	pub fn DefaultAccount<T: Config>() -> T::AccountId { T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap()}

	#[pallet::storage] /// --- ITEM ( total_stake )
	pub type TotalStake<T> = StorageValue<_, u64, ValueQuery>;
	#[pallet::storage] /// --- ITEM ( default_take )
	pub type DefaultTake<T> = StorageValue<_, u16, ValueQuery, DefaultDefaultTake<T>>;
	#[pallet::storage] /// --- ITEM ( global_block_emission )
	pub type BlockEmission<T> = StorageValue<_, u64, ValueQuery, DefaultBlockEmission<T>>;
	#[pallet::storage] /// --- ITEM ( total_issuance )
	pub type TotalIssuance<T> = StorageValue<_, u64, ValueQuery, DefaultTotalIssuance<T>>;
	#[pallet::storage] /// --- MAP ( hot ) --> stake | Returns the total amount of stake under a hotkey.
    pub type TotalHotkeyStake<T:Config> = StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultAccountTake<T>>;
	#[pallet::storage] /// --- MAP ( cold ) --> stake | Returns the total amount of stake under a coldkey.
    pub type TotalColdkeyStake<T:Config> = StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultAccountTake<T>>;
	#[pallet::storage] /// --- MAP ( hot ) --> cold | Returns the controlling coldkey for a hotkey.
    pub type Owner<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery, DefaultAccount<T>>;
	#[pallet::storage] /// --- MAP ( hot ) --> take | Returns the hotkey delegation take. And signals that this key is open for delegation.
    pub type Delegates<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u16, ValueQuery, DefaultDefaultTake<T>>;
	#[pallet::storage] /// --- DMAP ( hot, cold ) --> stake | Returns the stake under a hotkey prefixed by hotkey.
    pub type Stake<T:Config> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Identity, T::AccountId, u64, ValueQuery, DefaultAccountTake<T>>;

	/// =====================================
	/// ==== Difficulty / Registrations =====
	/// =====================================
	#[pallet::type_value] 
	pub fn DefaultLastAdjustmentBlock<T: Config>() -> u64 { 0 }
	#[pallet::type_value]
	pub fn DefaultRegistrationsThisBlock<T: Config>() ->  u16 { 0}
	#[pallet::type_value]
	pub fn DefaultDifficulty<T: Config>() -> u64 { T::InitialDifficulty::get() }
	#[pallet::type_value]
	pub fn DefaultMinDifficulty<T: Config>() -> u64 { 1 }
	#[pallet::type_value]
	pub fn DefaultMaxDifficulty<T: Config>() -> u64 { u64::MAX }
	#[pallet::type_value] 
	pub fn DefaultMaxRegistrationsPerBlock<T: Config>() -> u16 { T::InitialMaxRegistrationsPerBlock::get() }

	#[pallet::storage] /// ---- StorageItem Global Used Work.
    pub type UsedWork<T:Config> = StorageMap<_, Identity, Vec<u8>, u64, ValueQuery>;
	#[pallet::storage] /// --- MAP ( netuid ) --> Difficulty
	pub type Difficulty<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultDifficulty<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> Difficulty
	pub type MinDifficulty<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMinDifficulty<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> Difficulty
	pub type MaxDifficulty<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMaxDifficulty<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) -->  Block at last adjustment.
	pub type LastAdjustmentBlock<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultLastAdjustmentBlock<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> Registration this Block.
	pub type RegistrationsThisBlock<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultRegistrationsThisBlock<T>>;
	#[pallet::storage] /// --- ITEM( global_max_registrations_per_block ) 
	pub type MaxRegistrationsPerBlock<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxRegistrationsPerBlock<T> >;

	/// ==============================
	/// ==== Subnetworks Storage =====
	/// ==============================
	#[pallet::type_value] 
	pub fn DefaultN<T:Config>() -> u16 { 0 }
	#[pallet::type_value] 
	pub fn DefaultModality<T:Config>() -> u16 { 0 }
	#[pallet::type_value] 
	pub fn DefaultHotkeys<T:Config>() -> Vec<u16> { vec![ ] }
	#[pallet::type_value]
	pub fn DefaultNeworksAdded<T: Config>() ->  bool { false }

	#[pallet::storage] /// --- ITEM( tota_number_of_existing_networks )
	pub type TotalNetworks<T> = StorageValue<_, u16, ValueQuery>;
	#[pallet::storage] /// --- MAP ( netuid ) --> subnetwork_n (Number of UIDs in the network).
	pub type SubnetworkN<T:Config> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultN<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> modality   TEXT: 0, IMAGE: 1, TENSOR: 2
	pub type NetworkModality<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultModality<T>> ;
	#[pallet::storage] /// --- MAP ( netuid ) --> network_is_added
	pub type NetworksAdded<T:Config> = StorageMap<_, Identity, u16, bool, ValueQuery, DefaultNeworksAdded<T>>;	
	#[pallet::storage] /// --- DMAP ( netuid, netuid ) -> registration_requirement
	pub type NetworkConnect<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, u16, OptionQuery>;

	/// ==============================
	/// ==== Subnetwork Features =====
	/// ==============================
	#[pallet::type_value]
	pub fn DefaultEmissionValues<T: Config>() ->  u64 { 0 }
	#[pallet::type_value]
	pub fn DefaultPendingEmission<T: Config>() ->  u64 { 0 }
	#[pallet::type_value] 
	pub fn DefaultBlocksSinceLastStep<T: Config>() -> u64 { 0 }
	#[pallet::type_value] 
	pub fn DefaultLastMechansimStepBlock<T: Config>() -> u64 { 0 }
	#[pallet::type_value]
	pub fn DefaultTempo<T: Config>() -> u16 { T::InitialTempo::get() }

	#[pallet::storage] /// --- MAP ( netuid ) --> tempo
	pub type Tempo<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultTempo<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> emission_values
	pub type EmissionValues<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultEmissionValues<T>>;
	#[pallet::storage] /// --- MAP ( netuid ) --> pending_emission
	pub type PendingEmission<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultPendingEmission<T>>;
	#[pallet::storage] /// --- MAP ( netuid ) --> blocks_since_last_step.
	pub type BlocksSinceLastStep<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBlocksSinceLastStep<T>>;
	#[pallet::storage] /// --- MAP ( netuid ) --> last_mechanism_step_block
	pub type LastMechansimStepBlock<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultLastMechansimStepBlock<T> >;

	/// =======================================
	/// ==== Subnetwork Hyperparam storage ====
	/// =======================================	
	#[pallet::type_value] 
	pub fn DefaultWeightsSetRateLimit<T: Config>() -> u64 { 0 }
	#[pallet::type_value] 
	pub fn DefaultBlockAtRegistration<T: Config>() -> u64 { 0 }
	#[pallet::type_value]
	pub fn DefaultRho<T: Config>() -> u16 { T::InitialRho::get() }
	#[pallet::type_value]
	pub fn DefaultKappa<T: Config>() -> u16 { T::InitialKappa::get() }
	#[pallet::type_value] 
	pub fn DefaultMaxAllowedUids<T: Config>() -> u16 { T::InitialMaxAllowedUids::get() }
	#[pallet::type_value] 
	pub fn DefaultImmunityPeriod<T: Config>() -> u16 { T::InitialImmunityPeriod::get() }
	#[pallet::type_value] 
	pub fn DefaultActivityCutoff<T: Config>() -> u16 { T::InitialActivityCutoff::get() }
	#[pallet::type_value] 
	pub fn DefaultMaxWeightsLimit<T: Config>() -> u16 { T::InitialMaxWeightsLimit::get() }
	#[pallet::type_value] 
	pub fn DefaultStakePruningMin<T: Config>() -> u16 { T::InitialStakePruningMin::get() }
	#[pallet::type_value] 
	pub fn DefaultMinAllowedWeights<T: Config>() -> u16 { T::InitialMinAllowedWeights::get() }
	#[pallet::type_value] 
	pub fn DefaultValidatorEpochLen<T: Config>() -> u16 { T::InitialValidatorEpochLen::get() }
	#[pallet::type_value] 
	pub fn DefaultMaxAllowedValidators<T: Config>() -> u16 { T::InitialMaxAllowedValidators::get() }
	#[pallet::type_value]
	pub fn DefaultAdjustmentInterval<T: Config>() -> u16 { T::InitialAdjustmentInterval::get() }
	#[pallet::type_value]
	pub fn DefaultBondsMovingAverage<T: Config>() -> u64 { T::InitialBondsMovingAverage::get() }
	#[pallet::type_value] 
	pub fn DefaultValidatorBatchSize<T: Config>() -> u16 { T::InitialValidatorBatchSize::get() }
	#[pallet::type_value] 
	pub fn DefaultValidatorSequenceLen<T: Config>() -> u16 { T::InitialValidatorSequenceLen::get() }
	#[pallet::type_value] 
	pub fn DefaultValidatorEpochsPerReset<T: Config>() -> u16 { T::InitialValidatorEpochsPerReset::get() }
	#[pallet::type_value]
	pub fn DefaultValidatorExcludeQuantile<T: Config>() -> u16 {T::InitialValidatorExcludeQuantile::get()}
	#[pallet::type_value] 
	pub fn DefaultTargetRegistrationsPerInterval<T: Config>() -> u16 { T::InitialTargetRegistrationsPerInterval::get() }


	#[pallet::storage] /// --- MAP ( netuid ) --> Rho
	pub type Rho<T> =  StorageMap<_, Identity, u16, u16, ValueQuery, DefaultRho<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> Kappa
	pub type Kappa<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultKappa<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> uid, we use to record uids to prune at next epoch.
    pub type NeuronsToPruneAtNextEpoch<T:Config> = StorageMap<_, Identity, u16, u16, ValueQuery>;
	#[pallet::storage] /// --- MAP ( netuid ) --> registrations_this_interval
	pub type RegistrationsThisInterval<T:Config> = StorageMap<_, Identity, u16, u16, ValueQuery>;
	#[pallet::storage] /// --- MAP ( netuid ) --> max_allowed_uids
	pub type MaxAllowedUids<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxAllowedUids<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> immunity_period
	pub type ImmunityPeriod<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultImmunityPeriod<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> activity_cutoff
	pub type ActivityCutoff<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultActivityCutoff<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> stake_pruning_min
	pub type StakePruningMin<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultStakePruningMin<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> max_weight_limit
	pub type MaxWeightsLimit<T> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultMaxWeightsLimit<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> validator_epoch_len
	pub type ValidatorEpochLen<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorEpochLen<T> >; 
	#[pallet::storage] /// --- MAP ( netuid ) --> min_allowed_weights
	pub type MinAllowedWeights<T> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultMinAllowedWeights<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> max_allowed_validators
	pub type MaxAllowedValidators<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxAllowedValidators<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> adjustment_interval
	pub type AdjustmentInterval<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultAdjustmentInterval<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> bonds_moving_average
	pub type BondsMovingAverage<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBondsMovingAverage<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> validator_batch_size
	pub type ValidatorBatchSize<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorBatchSize<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> weights_set_rate_limit
	pub type WeightsSetRateLimit<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultWeightsSetRateLimit<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> validator_sequence_length
	pub type ValidatorSequenceLength<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorSequenceLen<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> validator_epochs_per_reset
	pub type ValidatorEpochsPerReset<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorEpochsPerReset<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> validator_exclude_quantile
	pub type ValidatorExcludeQuantile<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorExcludeQuantile<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> target_registrations_this_interval
	pub type TargetRegistrationsPerInterval<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultTargetRegistrationsPerInterval<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> block_at_registration
	pub type BlockAtRegistration<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, u64, ValueQuery, DefaultBlockAtRegistration<T> >;

	/// =======================================
	/// ==== Subnetwork Consensus Storage  ====
	/// =======================================
	#[pallet::type_value] 
	pub fn DefaultRank<T:Config>() -> u16 { 0 }
	#[pallet::type_value] 
	pub fn DefaultStake<T:Config>() -> u64 { 0 }
	#[pallet::type_value] 
	pub fn DefaultTrust<T:Config>() -> u16 { 0 }
	#[pallet::type_value] 
	pub fn DefaultEmission<T:Config>() -> u64 { 0 }
	#[pallet::type_value] 
	pub fn DefaultIncentive<T:Config>() -> u16 { 0 }
	#[pallet::type_value] 
	pub fn DefaultConsensus<T:Config>() -> u16 { 0 }
	#[pallet::type_value] 
	pub fn DefaultLastUpdate<T:Config>() -> u64 { 0 }
	#[pallet::type_value] 
	pub fn DefaultDividends<T: Config>() -> u16 { 0 }
	#[pallet::type_value] 
	pub fn DefaultActive<T:Config>() -> bool { false }
	#[pallet::type_value] 
	pub fn DefaultValidatorPermit<T:Config>() -> bool { false }
	#[pallet::type_value] 
	pub fn DefaultBonds<T:Config>() -> Vec<(u16, u16)> { vec![] }
	#[pallet::type_value] 
	pub fn DefaultWeights<T:Config>() -> Vec<(u16, u16)> { vec![] }
	#[pallet::type_value] 
	pub fn DefaultPruningScore<T: Config>() -> u16 { T::InitialPruningScore::get() }
	#[pallet::type_value] 
	pub fn DefaultKey<T:Config>() -> T::AccountId { T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap() }

	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> rank
	pub(super) type Rank<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u16, ValueQuery, DefaultRank<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, hotkey ) --> uid
	pub(super) type Uids<T:Config> = StorageDoubleMap<_, Identity, u16, Blake2_128Concat, T::AccountId, u16, OptionQuery>;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> trust
	pub(super) type Trust<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u16, ValueQuery, DefaultTrust<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> (version, ip address, port, ip type)
	pub(super) type AxonsMetaData<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, AxonMetadataOf, OptionQuery>;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> active
	pub(super) type Active<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, bool, ValueQuery, DefaultActive<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> hotkey
	pub(super) type Keys<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, T::AccountId, ValueQuery, DefaultKey<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> emission 
	pub(super) type Emission<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u64, ValueQuery, DefaultEmission<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> incentive
	pub(super) type Incentive<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u16, ValueQuery, DefaultIncentive<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> consensus
	pub(super) type Consensus<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u16, ValueQuery, DefaultConsensus<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> dividends
	pub(super) type Dividends<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u16, ValueQuery, DefaultDividends<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> last_update
	pub(super) type LastUpdate<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, u64 , ValueQuery, DefaultLastUpdate<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> bonds
    pub(super) type Bonds<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, Vec<(u16, u16)>, ValueQuery, DefaultBonds<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> validator_permit
    pub(super) type ValidatorPermit<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, bool, ValueQuery, DefaultValidatorPermit<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> weights
    pub(super) type Weights<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, Vec<(u16, u16)>, ValueQuery, DefaultWeights<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> pruning_score.
	pub(super) type PruningScores<T:Config> = StorageDoubleMap< _, Identity, u16, Identity, u16, u16, ValueQuery, DefaultPruningScore<T> >;
	
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
		/// --- Event created when multiple uids have been concurrently registered.
		BulkNeuronsRegistered(u16, u16),
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
		RhoSet(u16, u16),
		/// --- Event created when kappa is set for a subnet.
		KappaSet(u16, u16),
		/// --- Event created when minimun aloowed weight is set for a subnet.
		MinAllowedWeightSet(u16, u16),
		/// --- Event created when validator batch size is set for a subnet.
		ValidatorBatchSizeSet(u16, u16),
		/// --- Event created when validator sequence length i set for a subnet.
		ValidatorSequenceLengthSet(u16, u16),
		/// --- Event created when validator epoch per reset is set for a subnet.
		ValidatorEpochPerResetSet(u16, u16),
		/// --- Event create when weights set rate limit has been set for a subnet.
		WeightsSetRateLimitSet( u16, u64 ),
		/// --- Event created when immunity period is set for a subnet
		ImmunityPeriodSet(u16, u16),
		/// --- Event created when bonds moving average is set for a subnet
		BondsMovingAverageSet(u16, u64),
		/// --- Event created when setting the max number of allowed validators on a subnet.
		MaxAllowedValidatorsSet(u16, u16),
		/// --- Event created when the validator exclude quantile has been set for a subnet.
		ValidatorExcludeQuantileSet( u16, u16 ),
		/// --- Event created when the axon server information is added to the network.
		AxonServed(u16),
		/// --- Event created when emission ratios fr all networks is set
		EmissionValuesSet(),
		/// --- Event created when a network connection requirement is added.
		NetworkConnectionAdded( u16, u16, u16 ),
		/// --- Event created when a network connection requirement is removed.
		NetworkConnectionRemoved( u16, u16 ),
		/// --- Event created to signal a hotkey has become a delegate.
		DelegateAdded( T::AccountId, T::AccountId, u16 ),
		/// --- Event created when the default take is set.
		DefaultTakeSet( u16 ),
	}
	
	/// ================
	/// ==== Errors ====
	/// ================
	#[pallet::error]
	pub enum Error<T> {
		/// --- Thrown if we are attempting to create an invalid connection requirement.
		InvalidConnectionRequirement,
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
		// --- Thrown when a hotkey attempts to register into a network without passing the 
		// registration requirment from another network.
		DidNotPassConnectedNetworkRequirement,
		// --- Thrown if the hotkey attempt to become delegate when they are already.
		AlreadyDelegate,
		// --- Thrown if the hotkey attempts to set weights twice withing net_tempo/2 blocks.
		SettingWeightsToFast,
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
		fn on_initialize( _block_number: BlockNumberFor<T> ) -> Weight {
			Self::block_step();
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
		/// 	* 'NetworkDoesNotExist':
		/// 		- Attempting to set weights on a non-existent network.
		///
		/// 	* 'NotRegistered':
		/// 		- Attempting to set weights from a non registered account.
		///
		/// 	* 'WeightVecNotEqualSize':
		/// 		- Attempting to set weights with uids not of same length.
		///
		/// 	* 'DuplicateUids':
		/// 		- Attempting to set weights with duplicate uids.
		///
		/// 	* 'InvalidUid':
		/// 		- Attempting to set weights with invalid uids.
		///
		/// 	* 'NotSettingEnoughWeights':
		/// 		- Attempting to set weights with fewer weights than min.
		///
		/// 	* 'MaxWeightExceeded':
		/// 		- Attempting to set weights with max value exceeding limit.
        #[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn set_weights(
			origin:OriginFor<T>, 
			netuid: u16,
			dests: Vec<u16>, 
			weights: Vec<u16>
		) -> DispatchResult {
			Self::do_set_weights( origin, netuid, dests, weights )
		}

		/// --- Sets the key as a delegate.
		///
		/// # Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The signature of the caller's coldkey.
		///
		/// 	* 'hotkey' (T::AccountId):
		/// 		- The hotkey we are delegating (must be owned by the coldkey.)
		///
		/// 	* 'take' (u64):
		/// 		- The stake proportion that this hotkey takes from delegations.
		///
		/// # Event:
		/// 	* DelegateAdded;
		/// 		- On successfully setting a hotkey as a delegate.
		///
		/// # Raises:
		/// 	* 'NotRegistered':
		/// 		- The hotkey we are delegating is not registered on the network.
		///
		/// 	* 'NonAssociatedColdKey':
		/// 		- The hotkey we are delegating is not owned by the calling coldket.
		///
		///
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn become_delegate(
			origin: OriginFor<T>, 
			hotkey: T::AccountId
		) -> DispatchResult {
			Self::do_become_delegate(origin, hotkey, Self::get_default_take() )
		}

		/// --- Adds stake to a hotkey. The call is made from the
		/// coldkey account linked in the hotkey.
		/// Only the associated coldkey is allowed to make staking and
		/// unstaking requests. This protects the neuron against
		/// attacks on its hotkey running in production code.
		///
		/// # Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The signature of the caller's coldkey.
		///
		/// 	* 'hotkey' (T::AccountId):
		/// 		- The associated hotkey account.
		///
		/// 	* 'amount_staked' (u64):
		/// 		- The amount of stake to be added to the hotkey staking account.
		///
		/// # Event:
		/// 	* StakeAdded;
		/// 		- On the successfully adding stake to a global account.
		///
		/// # Raises:
		/// 	* 'CouldNotConvertToBalance':
		/// 		- Unable to convert the passed stake value to a balance.
		///
		/// 	* 'NotEnoughBalanceToStake':
		/// 		- Not enough balance on the coldkey to add onto the global account.
		///
		/// 	* 'NonAssociatedColdKey':
		/// 		- The calling coldkey is not associated with this hotkey.
		///
		/// 	* 'BalanceWithdrawalError':
		/// 		- Errors stemming from transaction pallet.
		///
		///
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn add_stake(
			origin: OriginFor<T>, 
			hotkey: T::AccountId, 
			amount_staked: u64
		) -> DispatchResult {
			Self::do_add_stake(origin, hotkey, amount_staked)
		}

		/// ---- Remove stake from the staking account. The call must be made
		/// from the coldkey account attached to the neuron metadata. Only this key
		/// has permission to make staking and unstaking requests.
		///
		/// # Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The signature of the caller's coldkey.
		///
		/// 	* 'hotkey' (T::AccountId):
		/// 		- The associated hotkey account.
		///
		/// 	* 'amount_unstaked' (u64):
		/// 		- The amount of stake to be added to the hotkey staking account.
		///
		/// # Event:
		/// 	* StakeRemoved;
		/// 		- On the successfully removing stake from the hotkey account.
		///
		/// # Raises:
		/// 	* 'NotRegistered':
		/// 		- Thrown if the account we are attempting to unstake from is non existent.
		///
		/// 	* 'NonAssociatedColdKey':
		/// 		- Thrown if the coldkey does not own the hotkey we are unstaking from.
		///
		/// 	* 'NotEnoughStaketoWithdraw':
		/// 		- Thrown if there is not enough stake on the hotkey to withdwraw this amount. 
		///
		/// 	* 'CouldNotConvertToBalance':
		/// 		- Thrown if we could not convert this amount to a balance.
		///
		///
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn remove_stake(
			origin: OriginFor<T>, 
			hotkey: T::AccountId, 
			amount_unstaked: u64
		) -> DispatchResult {
			Self::do_remove_stake(origin, hotkey, amount_unstaked)
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
		pub fn serve_axon(
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
		/// 		- The signature of the calling hotkey.
		///
		/// 	* 'netuid' (u16):
		/// 		- The u16 network identifier.
		///
		/// 	* 'block_number' ( u64 ):
		/// 		- Block hash used to prove work done.
		///
		/// 	* 'nonce' ( u64 ):
		/// 		- Positive integer nonce used in POW.
		///
		/// 	* 'work' ( Vec<u8> ):
		/// 		- Vector encoded bytes representing work done.
		///
		/// 	* 'hotkey' ( T::AccountId ):
		/// 		- Hotkey to be registered to the network.
		///
		/// 	* 'coldkey' ( T::AccountId ):
		/// 		- Associated coldkey account.
		///
		/// # Event:
		/// 	* NeuronRegistered;
		/// 		- On successfully registereing a uid to a neuron slot on a subnetwork.
		///
		/// # Raises:
		/// 	* 'NetworkDoesNotExist':
		/// 		- Attempting to registed to a non existent network.
		///
		/// 	* 'TooManyRegistrationsThisBlock':
		/// 		- This registration exceeds the total allowed on this network this block.
		///
		/// 	* 'AlreadyRegistered':
		/// 		- The hotkey is already registered on this network.
		///
		/// 	* 'InvalidWorkBlock':
		/// 		- The work has been performed on a stale, future, or non existent block.
		///
		/// 	* 'WorkRepeated':
		/// 		- This work for block has already been used.
		///
		/// 	* 'InvalidDifficulty':
		/// 		- The work does not match the difficutly.
		///
		/// 	* 'InvalidSeal':
		/// 		- The seal is incorrect.
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

		/// ---- Sudo add a network to the network set.
		/// # Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- Must be sudo.
		///
		/// 	* 'netuid' (u16):
		/// 		- The u16 network identifier.
		///
		/// 	* 'tempo' ( u16 ):
		/// 		- Number of blocks between epoch step.
		///
		/// 	* 'modality' ( u16 ):
		/// 		- Network modality specifier.
		///
		/// # Event:
		/// 	* NetworkAdded;
		/// 		- On successfully creation of a network.
		///
		/// # Raises:
		/// 	* 'NetworkExist':
		/// 		- Attempting to register an already existing.
		///
		/// 	* 'InvalidModality':
		/// 		- Attempting to register a network with an invalid modality.
		///
		/// 	* 'InvalidTempo':
		/// 		- Attempting to register a network with an invalid tempo.
		///
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_add_network(
			origin: OriginFor<T>,
			netuid: u16,
			tempo: u16,
			modality: u16
		)-> DispatchResult {
			Self::do_add_network(origin, netuid, tempo, modality)
		}

		/// ---- Sudo remove a network from the network set.
		/// # Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- Must be sudo.
		///
		/// 	* 'netuid' (u16):
		/// 		- The u16 network identifier.
		///
		/// # Event:
		/// 	* NetworkRemoved;
		/// 		- On the successfull removing of this network.
		///
		/// # Raises:
		/// 	* 'NetworkDoesNotExist':
		/// 		- Attempting to remove a non existent network.
		///
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_remove_network(
			origin: OriginFor<T>,
			netuid: u16
		) -> DispatchResult {
			Self::do_remove_network(origin, netuid)
		} 

		/// ---- Sudo set emission values for all networks.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuids` (Vec<u16>):
		/// 		- A vector of network uids values. This must include all netuids.
		///
		/// 	* `emission` (Vec<u64>):
		/// 		- The emission values associated with passed netuids in order.
		/// 
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn sudo_set_emission_values(
			origin: OriginFor<T>,
			netuids: Vec<u16>,
			emission: Vec<u64>,
		) -> DispatchResult {
			Self::do_set_emission_values( 
				origin,
				netuids,
				emission
			)
		}

		/// ---- Sudo create and load network.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` ( u16 ):
		/// 		- The network we are intending on performing the bulk creation on.
		///
		/// 	* `n` ( u16 ):
		/// 		- Network size.
		///
		/// 	* `uids` ( Vec<u16> ):
		/// 		- List of uids to set keys under.
		///
		/// 	* `hotkeys` ( Vec<T::AccountId> ):
		/// 		- List of hotkeys to register on account.
		///
		/// 	* `coldkeys` ( Vec<T::AccountId> ):
		/// 		- List of coldkeys related to hotkeys.
		/// 
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn sudo_bulk_register(
			origin: OriginFor<T>,
			netuid: u16,
			hotkeys: Vec<T::AccountId>,
			coldkeys: Vec<T::AccountId>
		) -> DispatchResult {
			Self::do_bulk_register( 
				origin,
				netuid,
				hotkeys,
				coldkeys
			)
		}

		/// ---- Sudo add a network connect requirement.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid_a` (u16):
		/// 		- The network we are adding the requirment to (parent network)
		///
		/// 	* `netuid_b` (u16):
		/// 		- The network we the requirement refers to (child network)
		///
		/// 	* `requirement` (u16):
		/// 		- The topk percentile prunning score requirement (u16:MAX normalized.)
		///
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_add_network_connection_requirement( origin:OriginFor<T>, netuid_a: u16, netuid_b: u16, requirement: u16 ) -> DispatchResult { 
			Self::do_sudo_add_network_connection_requirement( origin, netuid_a, netuid_b, requirement )
		}

		/// ---- Sudo remove a network connection requirement.
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid_a` (u16):
		/// 		- The network we are removing the requirment from.
		///
		/// 	* `netuid_b` (u16):
		/// 		- The required network connection to remove.
		///   
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_remove_network_connection_requirement( origin:OriginFor<T>, netuid_a: u16, netuid_b: u16 ) -> DispatchResult { 
			Self::do_sudo_remove_network_connection_requirement( origin, netuid_a, netuid_b )
		}

		/// ==================================
		/// ==== Parameter Sudo calls ========
		/// ==================================
		/// Each function sets the corresponding hyper paramter on the specified network
		/// Args:
		/// 	* 'origin': (<T as frame_system::Config>Origin):
		/// 		- The caller, must be sudo.
		///
		/// 	* `netuid` (u16):
		/// 		- The network identifier.
		///
		/// 	* `hyperparameter value` (u16):
		/// 		- The value of the hyper parameter.
		///   
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_default_take( origin:OriginFor<T>, default_take: u16 ) -> DispatchResult {  
			Self::do_sudo_set_default_take( origin, default_take )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_weights_set_rate_limit( origin:OriginFor<T>, netuid: u16, weights_set_rate_limit: u64 ) -> DispatchResult {  
			Self::do_sudo_set_weights_set_rate_limit( origin, netuid, weights_set_rate_limit )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_bonds_moving_average( origin:OriginFor<T>, netuid: u16, bonds_moving_average: u64 ) -> DispatchResult {  
			Self::do_sudo_set_bonds_moving_average( origin, netuid, bonds_moving_average )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_max_allowed_validators( origin:OriginFor<T>, netuid: u16, max_allowed_validators: u16 ) -> DispatchResult {  
			Self::do_sudo_set_max_allowed_validators( origin, netuid, max_allowed_validators )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_difficulty( origin:OriginFor<T>, netuid: u16, difficulty: u64 ) -> DispatchResult {
			Self::do_sudo_set_difficulty( origin, netuid, difficulty )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_adjustment_interval( origin:OriginFor<T>, netuid: u16, adjustment_interval: u16 ) -> DispatchResult { 
			Self::do_sudo_set_adjustment_interval( origin, netuid, adjustment_interval )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_target_registrations_per_interval( origin:OriginFor<T>, netuid: u16, target_registrations_per_interval: u16 ) -> DispatchResult {
			Self::do_sudo_set_target_registrations_per_interval( origin, netuid, target_registrations_per_interval )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_activity_cutoff( origin:OriginFor<T>, netuid: u16, activity_cutoff: u16 ) -> DispatchResult {
			Self::do_sudo_set_activity_cutoff( origin, netuid, activity_cutoff )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_rho( origin:OriginFor<T>, netuid: u16, rho: u16 ) -> DispatchResult {
			Self::do_sudo_set_rho( origin, netuid, rho )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_kappa( origin:OriginFor<T>, netuid: u16, kappa: u16 ) -> DispatchResult {
			Self::do_sudo_set_kappa( origin, netuid, kappa )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_max_allowed_uids( origin:OriginFor<T>, netuid: u16, max_allowed_uids: u16 ) -> DispatchResult {
			Self::do_sudo_set_max_allowed_uids(origin, netuid, max_allowed_uids )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_min_allowed_weights( origin:OriginFor<T>, netuid: u16, min_allowed_weights: u16 ) -> DispatchResult {
			Self::do_sudo_set_min_allowed_weights( origin, netuid, min_allowed_weights )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_validator_batch_size( origin:OriginFor<T>, netuid: u16, validator_batch_size: u16 ) -> DispatchResult {
			Self::do_sudo_set_validator_batch_size( origin, netuid, validator_batch_size )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_validator_sequence_length( origin:OriginFor<T>, netuid: u16, validator_sequence_length: u16 ) -> DispatchResult {
			Self::do_sudo_set_validator_sequence_length(origin, netuid, validator_sequence_length )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_validator_epochs_per_reset( origin:OriginFor<T>, netuid: u16, validator_epochs_per_reset: u16 ) -> DispatchResult {
			Self::do_sudo_set_validator_epochs_per_reset( origin, netuid, validator_epochs_per_reset )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_immunity_period( origin:OriginFor<T>, netuid: u16, immunity_period: u16 ) -> DispatchResult {
			Self::do_sudo_set_immunity_period( origin, netuid, immunity_period )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_max_weight_limit( origin:OriginFor<T>, netuid: u16, max_weight_limit: u16 ) -> DispatchResult {
			Self::do_sudo_set_max_weight_limit( origin, netuid, max_weight_limit )
		}
		#[pallet::weight((0, DispatchClass::Operational, Pays::No))]
		pub fn sudo_set_validator_exclude_quantile( origin:OriginFor<T>, netuid: u16, validator_exclude_quantile: u16 ) -> DispatchResult {
			Self::do_sudo_set_validator_exclude_quantile( origin, netuid, validator_exclude_quantile )
		}

	}	
}
