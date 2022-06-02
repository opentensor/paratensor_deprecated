#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

use frame_support::{dispatch, ensure, traits::{
		Currency, 
		ExistenceRequirement,
		IsSubType, 
		tokens::{
			WithdrawReasons
		}
	}, weights::{
		DispatchInfo, 
		PostDispatchInfo
	}
};
use frame_system::{
	self as system, 
	ensure_signed
};


use sp_std::vec::Vec;
use sp_std::vec;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// ************************************************************
///	-Subtensor-Imports
/// ************************************************************

mod registration;
mod steps;
mod staking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_core::{U256};
	use frame_support::IterableStorageMap;
	use frame_support::{pallet_prelude::*, Printable, traits::{Currency}};
	use sp_std::vec::Vec;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// --- The transaction fee in RAO per byte
		type TransactionByteFee: Get<BalanceOf<Self>>;
		
		/// Initial registration difficulty.
		#[pallet::constant]
		type InitialDifficulty: Get<u64>;

		/// --- Currency type that will be used to place deposits on neurons
		type Currency: Currency<Self::AccountId> + Send + Sync;

		/// Initial max registrations per block.
		#[pallet::constant]
		type InitialMaxRegistrationsPerBlock: Get<u64>;

		/// Immunity Period Constant.
		#[pallet::constant]
		type InitialImmunityPeriod: Get<u64>;

		/// Initial stake pruning denominator
		#[pallet::constant]
		type InitialStakePruningDenominator: Get<u64>;
		
		/// Initial incentive pruning denominator
		#[pallet::constant]
		type InitialIncentivePruningDenominator: Get<u64>;

		/// Max UID constant.
		#[pallet::constant]
		type InitialMaxAllowedUids: Get<u64>;

		/// Debug is on
		#[pallet::constant]
		type SDebug: Get<u64>;

		/// Rho constant
		#[pallet::constant]
		type InitialRho: Get<u64>;
	
		/// Kappa constant
		#[pallet::constant]
		type InitialKappa: Get<u64>;

		/// SelfOwnership constant
		#[pallet::constant]
		type SelfOwnership: Get<u64>;

		/// Default Batch size.
		#[pallet::constant]
		type InitialValidatorBatchSize: Get<u64>;

		/// Default Batch size.
		#[pallet::constant]
		type InitialValidatorSequenceLen: Get<u64>;

		/// Default Epoch length.
		#[pallet::constant]
		type InitialValidatorEpochLen: Get<u64>;

		/// Default Reset length.
		#[pallet::constant]
		type InitialValidatorEpochsPerReset: Get<u64>;

		/// Initial min allowed weights.
		#[pallet::constant]
		type InitialMinAllowedWeights: Get<u64>;

		/// Blocks per era.
		#[pallet::constant]
		type InitialBondsMovingAverage: Get<u64>;

		/// Initial allowed max min weight ratio
		#[pallet::constant]
		type InitialMaxAllowedMaxMinRatio: Get<u64>;

		/// Initial foundation distribution
		#[pallet::constant]
		type InitialFoundationDistribution: Get<u64>;

		/// Initial registration difficulty.
		#[pallet::constant]
		type InitialIssuance: Get<u64>;

		/// Minimum registration difficulty
		#[pallet::constant]
		type MinimumDifficulty: Get<u64>;

		/// Maximum registration difficulty
		#[pallet::constant]
		type MaximumDifficulty: Get<u64>;

		/// Blocks per step.
		#[pallet::constant]
		type InitialBlocksPerStep: Get<u64>;

		/// Activity constant
		#[pallet::constant]
		type InitialActivityCutoff: Get<u64>;

		/// Initial adjustment interval.
		#[pallet::constant]
		type InitialAdjustmentInterval: Get<u64>;

		/// Initial target registrations per interval.
		#[pallet::constant]
		type InitialTargetRegistrationsPerInterval: Get<u64>;

		

	}

	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type NeuronMetadataOf<T> = NeuronMetadata<AccountIdOf<T>>;

	#[derive(Encode, Decode, Default, TypeInfo)]
	pub struct NeuronMetadata<AccountId> {

		/// ---- The endpoint's code version.
        pub version: u32,

        /// ---- The endpoint's u128 encoded ip address of type v6 or v4.
        pub ip: u128,

        /// ---- The endpoint's u16 encoded port.
        pub port: u16,

        /// ---- The endpoint's ip type, 4 for ipv4 and 6 for ipv6.
        pub ip_type: u8,

        /// ---- The endpoint's unique identifier.
        pub uid: u32,

        /// ---- The neuron modality. Modalities specify which datatype
        /// the neuron endpoint can process. This information is non
        /// verifiable. However, neurons should set this correctly
        /// in order to be detected by others with this datatype.
        /// The initial modality codes are:
        /// TEXT: 0
        /// IMAGE: 1
        /// TENSOR: 2
        pub modality: u8,

        /// ---- The associated hotkey account.
        /// Registration and changing weights can be made by this
        /// account.
        pub hotkey: AccountId,

        /// ---- The associated coldkey account.
        /// Staking and unstaking transactions must be made by this account.
        /// The hotkey account (in the Neurons map) has permission to call
        /// subscribe and unsubscribe.
        pub coldkey: AccountId,

		/// ---- Is this neuron active in the incentive mechanism.
		pub active: u32,

		/// ---- Block number of last chain update.
		pub last_update: u64,

		/// ---- Transaction priority.
		pub priority: u64,

		/// ---- The associated stake in this account.
		pub stake: u64,

		/// ---- The associated rank in this account.
		pub rank: u64,

		/// ---- The associated trust in this account.
		pub trust: u64,

		/// ---- The associated consensus in this account.
		pub consensus: u64,

		/// ---- The associated incentive in this account.
		pub incentive: u64,

		/// ---- The associated dividends in this account.
		pub dividends: u64,

		/// ---- The associated emission last block for this account.
		pub emission: u64,

		/// ---- The associated bond ownership.
		pub bonds: Vec<(u32,u64)>,

		/// ---- The associated weights ownership.
		pub weights: Vec<(u32,u32)>,
    }

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// ************************************************************
	///	*---- Storage Objects
	/// ************************************************************
	
	// --- Number of peers.
	#[pallet::storage]
	pub type N<T> = StorageValue<
		_, 
		u32, 
		ValueQuery
	>;

	/// ---- Maps from hotkey to uid.
	#[pallet::storage]
	#[pallet::getter(fn hotkey)]
    pub(super) type Hotkeys<T:Config> = StorageMap<
		_, 
		Blake2_128Concat, 
		T::AccountId, 
		u32, 
		ValueQuery
	>;


	#[pallet::storage]
	pub type RegistrationsThisBlock<T> = StorageValue<
		_, 
		u64, 
		ValueQuery
	>;


	#[pallet::type_value] 
	pub fn DefaultMaxRegistrationsPerBlock<T: Config>() -> u64 { T::InitialMaxRegistrationsPerBlock::get() }
	#[pallet::storage]
	pub type MaxRegistrationsPerBlock<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultMaxRegistrationsPerBlock<T>
	>;


	#[pallet::storage]
	#[pallet::getter(fn usedwork)]
    pub(super) type UsedWork<T:Config> = StorageMap<
		_, 
		Identity, 
		Vec<u8>, 
		u64,
		ValueQuery
	>;


	#[pallet::type_value] 
	pub fn DefaultDifficulty<T: Config>() -> u64 { T::InitialDifficulty::get() }
	#[pallet::storage]
	pub type Difficulty<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultDifficulty<T>
	>;


	#[pallet::type_value] 
	pub fn DefaultMaxAllowedUids<T: Config>() -> u64 { T::InitialMaxAllowedUids::get() }
	#[pallet::storage]
	pub type MaxAllowedUids<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultMaxAllowedUids<T>
	>;

	/// ---- Maps from uid to neuron.
	#[pallet::storage]
	#[pallet::getter(fn uid)]
	pub(super) type Neurons<T:Config> = StorageMap<
		_, 
		Identity, 
		u32, 
		NeuronMetadataOf<T>, 
		OptionQuery
	>;

	#[pallet::storage]
	pub type TotalStake<T> = StorageValue<
		_, 
		u64, 
		ValueQuery
	>;

	#[pallet::type_value] 
	pub fn DefaultImmunityPeriod<T: Config>() -> u64 { T::InitialImmunityPeriod::get() }
	#[pallet::storage]
	pub type ImmunityPeriod<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultImmunityPeriod<T>
	>;

	#[pallet::type_value] 
	pub fn DefaultIncentivePruningDenominator<T: Config>() -> u64 { T::InitialIncentivePruningDenominator::get() }
	#[pallet::storage]
	pub type IncentivePruningDenominator<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultIncentivePruningDenominator<T>
	>;

	#[pallet::type_value] 
	pub fn DefaultStakePruningDenominator<T: Config>() -> u64 { T::InitialStakePruningDenominator::get() }
	#[pallet::storage]
	pub type StakePruningDenominator<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultStakePruningDenominator<T>
	>;

	#[pallet::type_value] 
	pub fn DefaultBlockAtRegistration<T: Config>() -> u64 { 0 }
	#[pallet::storage]
	#[pallet::getter(fn block_at_registration)]
    pub(super) type BlockAtRegistration<T:Config> = StorageMap<
		_, 
		Identity, 
		u32, 
		u64, 
		ValueQuery,
		DefaultBlockAtRegistration<T>
	>;

	/// ---- Maps from uid to uid as a set which we use to record uids to prune at next epoch.
	#[pallet::storage]
	#[pallet::getter(fn uid_to_prune)]
	pub(super) type NeuronsToPruneAtNextEpoch<T:Config> = StorageMap<
		_, 
		Identity, 
		u32, 
		u32, 
		ValueQuery,
	>;

	#[pallet::storage]
	pub type RegistrationsThisInterval<T> = StorageValue<
		_, 
		u64, 
		ValueQuery
	>;

	#[pallet::type_value] 
	pub fn DefaultAdjustmentInterval<T: Config>() -> u64 { T::InitialAdjustmentInterval::get() }
	#[pallet::storage]
	pub type AdjustmentInterval<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultAdjustmentInterval<T>
	>;

	#[pallet::type_value] 
	pub fn DefaultTargetRegistrationsPerInterval<T: Config>() -> u64 { T::InitialTargetRegistrationsPerInterval::get() }
	#[pallet::storage]
	pub type TargetRegistrationsPerInterval<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultTargetRegistrationsPerInterval<T>
	>;
	#[pallet::type_value] 
	pub fn DefaultBlocksSinceLastStep<T: Config>() -> u64 { 0 }
	#[pallet::storage]
	pub type BlocksSinceLastStep<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultBlocksSinceLastStep<T>
	>;
	#[pallet::type_value] 
	pub fn DefaultBlocksPerStep<T: Config>() -> u64 { T::InitialBlocksPerStep::get() }
	#[pallet::storage]
	pub type BlocksPerStep<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultBlocksPerStep<T>
	>;
	#[pallet::storage]
	pub type LastMechansimStepBlock<T> = StorageValue<
		_, 
		u64, 
		ValueQuery
	>;
/// ************************************************************
	///	-Genesis-Configuration
	/// ************************************************************
	/// ---- Genesis Configuration (Mostly used for testing.)
    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub stake: Vec<(u64, u64)>,
    }

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {
				stake: Default::default(),
			}
		}
	}
    
    #[pallet::genesis_build]
    impl<T:Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {		
		}
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		/// --- Event created when a new neuron account has been registered to 
		/// the chain.
		NeuronRegistered(u32),
		/// --- Event created when the difficulty adjustment interval has been set.
		AdjustmentIntervalSet(u64),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
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

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
	/// ---- Registers a new neuron to the graph. 
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
		///
		/// # Event:
		/// 	* 'NeuronRegistered':
		/// 		- On subscription of a new neuron to the active set.
		///
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn register( 
				origin:OriginFor<T>, 
				block_number: u64, 
				nonce: u64, 
				work: Vec<u8>,
				hotkey: T::AccountId, 
				coldkey: T::AccountId 
		) -> DispatchResult {
			Self::do_registration(origin, block_number, nonce, work, hotkey, coldkey)
		}
	}
		// ---- Subtensor helper functions.
		impl<T: Config> Pallet<T> {
			pub fn get_registrations_this_block( ) -> u64 {
				RegistrationsThisBlock::<T>::get()
			}

			pub fn get_max_registratations_per_block( ) -> u64 {
				MaxRegistrationsPerBlock::<T>::get()
			}

			// -- Difficulty.
			pub fn get_difficulty( ) -> U256 {
			return U256::from( Self::get_difficulty_as_u64() );
			}

			pub fn get_difficulty_as_u64( ) -> u64 {
				Difficulty::<T>::get()
			}

			pub fn get_max_allowed_uids( ) -> u64 {
				return MaxAllowedUids::<T>::get();
			}

			pub fn get_immunity_period( ) -> u64 {
				return ImmunityPeriod::<T>::get();
			}

			pub fn get_total_stake( ) -> u64 {
				return TotalStake::<T>::get();
			}

			pub fn get_stake_pruning_denominator( ) -> u64 {
				return StakePruningDenominator::<T>::get();
			}

			pub fn get_incentive_pruning_denominator( ) -> u64 {
				return IncentivePruningDenominator::<T>::get();
			}

			// --- Returns the next available network uid.
			// uids increment up to u64:MAX, this allows the chain to
			// have 18,446,744,073,709,551,615 peers before an overflow.
			pub fn get_neuron_count() -> u32 {
				let uid = N::<T>::get();
				uid
			}

			// --- Returns the next available network uid and increments uid.
			pub fn get_next_uid() -> u32 {
				let uid = N::<T>::get();
				assert!(uid < u32::MAX);  // The system should fail if this is ever reached.
				N::<T>::put(uid + 1);
				uid
			}

			// --- Returns Option if the u64 converts to a balance
			// use .unwarp if the result returns .some().
			pub fn u64_to_balance(input: u64) -> Option<<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance>
			{
				input.try_into().ok()
			}

			// --- Returns hotkey associated with the hotkey account.
			// This should be called in conjunction with is_hotkey_active
			// to ensure this function does not throw an error.
			pub fn get_uid_for_hotkey(hotkey_id: &T::AccountId) -> u32{
				return Hotkeys::<T>::get(&hotkey_id);
			}
			pub fn get_neuron_for_uid ( uid: u32 ) -> NeuronMetadataOf<T> {
				return Neurons::<T>::get( uid ).unwrap();
			}

			// --- Returns the neuron associated with the passed hotkey.
			// The function makes a double mapping from hotkey -> uid -> neuron.
			pub fn get_neuron_for_hotkey(hotkey_id: &T::AccountId) -> NeuronMetadataOf<T> {
				let uid = Self::get_uid_for_hotkey(hotkey_id);
				return Self::get_neuron_for_uid(uid);
			}

			pub fn set_difficulty_from_u64( difficulty: u64 ) {
				Difficulty::<T>::set( difficulty );
			}
			// --- Returns true if the uid is to be prunned at the next epoch.
			pub fn will_be_prunned ( uid:u32 ) -> bool {
				return NeuronsToPruneAtNextEpoch::<T>::contains_key( uid );
			}
					// Setters
			pub fn set_stake_from_vector( stake: Vec<u64> ) {
				let mut total_stake: u64 = 0;
				for uid_i in 0..Self::get_neuron_count() {
					let mut neuron = Neurons::<T>::get(uid_i).unwrap();
					neuron.stake = stake[ uid_i as usize ];
					Neurons::<T>::insert( uid_i, neuron );
					total_stake += stake[ uid_i as usize ];
				}
				TotalStake::<T>::set( total_stake );
			}
			pub fn set_max_allowed_uids( max_allowed_uids: u64 ) {
				MaxAllowedUids::<T>::put( max_allowed_uids );
			}
			pub fn set_immunity_period( immunity_period: u64 ) {
				ImmunityPeriod::<T>::put( immunity_period );
			}
			pub fn get_registrations_this_interval( ) -> u64 {
				RegistrationsThisInterval::<T>::get()
			}
				// -- Adjustment Interval.
			pub fn get_adjustment_interval() -> u64 {
				AdjustmentInterval::<T>::get()
			}
			// -- Target registrations per interval.
			pub fn get_target_registrations_per_interval() -> u64 {
				TargetRegistrationsPerInterval::<T>::get()
			}
			pub fn set_target_registrations_per_interval( target: u64 ) {
				TargetRegistrationsPerInterval::<T>::put( target );
			}
			pub fn set_adjustment_interval( interval: u64 ) {
				AdjustmentInterval::<T>::put( interval );
			}
			pub fn set_max_registratations_per_block( max_registrations: u64 ){
				MaxRegistrationsPerBlock::<T>::put( max_registrations );
			}
			// --- Returns true if the uid is active, i.e. there
			// is a staking, last_update, and neuron account associated
			// with this uid.
			pub fn is_uid_active(uid: u32) -> bool {
				return Neurons::<T>::contains_key(uid);
			}
			pub fn get_blocks_since_last_step( ) -> u64 {
				BlocksSinceLastStep::<T>::get()
			}
			pub fn get_blocks_per_step( ) -> u64 {
				BlocksPerStep::<T>::get()
			}
				// -- Get Block emission.
			pub fn get_block_emission( ) -> u64 {
				return 1000000000;
			}
		}
}
