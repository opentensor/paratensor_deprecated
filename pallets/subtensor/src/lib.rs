#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use frame_system::{
	self as system
};
use frame_support::{traits::{Currency}};
use sp_std::vec::Vec;
use sp_std::vec;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


/// ************************************************************
///	-Subtensor-Imports
/// ************************************************************
mod step;

#[frame_support::pallet]
pub mod pallet {
	use sp_core::{U256};
	use frame_support::{pallet_prelude::*, traits::{Currency}};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// --- Currency type that will be used to place deposits on neurons
		type Currency: Currency<Self::AccountId> + Send + Sync;

		/// Debug is on
		#[pallet::constant]
		type SDebug: Get<u64>;

		/// Rho constant
		#[pallet::constant]
		type InitialRho: Get<u64>;

		/// Kappa constant
		#[pallet::constant]
		type InitialKappa: Get<u64>;

		/// Minimum registration difficulty
		#[pallet::constant]
		type MinimumDifficulty: Get<u64>;
		
		/// Maximum registration difficulty
		#[pallet::constant]
		type MaximumDifficulty: Get<u64>;

		/// Initial adjustment interval.
		#[pallet::constant]
		type InitialAdjustmentInterval: Get<u64>;

		/// Initial registration difficulty.
		#[pallet::constant]
		type InitialDifficulty: Get<u64>;

		/// Initial target registrations per interval.
		#[pallet::constant]
		type InitialTargetRegistrationsPerInterval: Get<u64>;

		/// Blocks per era.
		#[pallet::constant]
		type InitialBondsMovingAverage: Get<u64>;
				
		/// SelfOwnership constant
		#[pallet::constant]
		type SelfOwnership: Get<u64>;

		/// Activity constant
		#[pallet::constant]
		type InitialActivityCutoff: Get<u64>;

		/// Initial registration difficulty.
		#[pallet::constant]
		type InitialIssuance: Get<u64>;


	}

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type NeuronMetadataOf<T> = NeuronMetadata<AccountIdOf<T>>;
	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Encode, Decode, TypeInfo)]
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
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage

	// --- Number of peers.
	#[pallet::storage]
	pub type N<T> = StorageValue<
		_, 
		u32, 
		ValueQuery
	>;

	#[pallet::storage]
	pub type TotalStake<T> = StorageValue<
		_, 
		u64, 
		ValueQuery
	>;

	#[pallet::storage]
	pub type RegistrationsThisBlock<T> = StorageValue<
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
	pub fn DefaultDifficulty<T: Config>() -> u64 { T::InitialDifficulty::get() }
	#[pallet::storage]
	pub type Difficulty<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultDifficulty<T>
	>;

	#[pallet::storage]
	pub type LastMechansimStepBlock<T> = StorageValue<
		_, 
		u64, 
		ValueQuery
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
	pub fn DefaultBondsMovingAverage<T: Config>() -> u64 { T::InitialBondsMovingAverage::get() }
	#[pallet::storage]
	pub type BondsMovingAverage<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultBondsMovingAverage<T>
	>;

	#[pallet::type_value] 
	pub fn DefaultRho<T: Config>() -> u64 { T::InitialRho::get() }
	#[pallet::storage]
	pub type Rho<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultRho<T>
	>;

	#[pallet::type_value] 
	pub fn DefaultKappa<T: Config>() -> u64 { T::InitialKappa::get() }
	#[pallet::storage]
	pub type Kappa<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultKappa<T>
	>;

	#[pallet::type_value] 
	pub fn DefaultActivityCutoff<T: Config>() -> u64 { T::InitialActivityCutoff::get() }
	#[pallet::storage]
	pub type ActivityCutoff<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultActivityCutoff<T>
	>;

	#[pallet::type_value] 
	pub fn DefaultTotalIssuance<T: Config>() -> u64 { T::InitialIssuance::get() }
	#[pallet::storage]
	pub type TotalIssuance<T> = StorageValue<
		_, 
		u64, 
		ValueQuery,
		DefaultTotalIssuance<T>
	>;


	#[pallet::storage]
	pub type LastDifficultyAdjustmentBlock<T> = StorageValue<
		_, 
		u64, 
		ValueQuery
	>;

	#[pallet::storage]
	pub type RegistrationsThisInterval<T> = StorageValue<
		_, 
		u64, 
		ValueQuery
	>;

	#[pallet::storage]
	pub type TotalEmission<T> = StorageValue<
		_, 
		u64, 
		ValueQuery
	>;

	#[pallet::storage]
	pub type TotalBondsPurchased<T> = StorageValue<
		_, 
		u64, 
		ValueQuery
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

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
	}

	// ---- Subtensor helper functions.
	impl<T: Config> Pallet<T> {

		// TURN ON DEBUG
		pub fn debug() -> bool {
			return T::SDebug::get() == 1
		}

		// -- Minimum difficulty
		pub fn get_minimum_difficulty( ) -> u64 {
			return T::MinimumDifficulty::get();
		}
		
		// -- Maximum difficulty
		pub fn get_maximum_difficulty( ) -> u64 {
			return T::MaximumDifficulty::get();
		}

		// -- Adjustment Interval.
		pub fn get_adjustment_interval() -> u64 {
			AdjustmentInterval::<T>::get()
		}

		// -- Difficulty.
		pub fn get_difficulty( ) -> U256 {
			return U256::from( Self::get_difficulty_as_u64() );
		}

		pub fn get_difficulty_as_u64( ) -> u64 {
			Difficulty::<T>::get()
		}

		pub fn set_difficulty_from_u64( difficulty: u64 ) {
			Difficulty::<T>::set( difficulty );
		}

		// -- Target registrations per interval.
		pub fn get_target_registrations_per_interval() -> u64 {
			TargetRegistrationsPerInterval::<T>::get()
		}
		pub fn set_target_registrations_per_interval( target: u64 ) {
			TargetRegistrationsPerInterval::<T>::put( target );
		}
		// Variable Parameters
		pub fn get_registrations_this_interval( ) -> u64 {
			RegistrationsThisInterval::<T>::get()
		}

		pub fn get_bonds_moving_average( ) -> u64 {
			BondsMovingAverage::<T>::get()
		}
		pub fn set_bonds_moving_average( bonds_moving_average: u64 ) {
			BondsMovingAverage::<T>::set( bonds_moving_average );
		}

		// -- Get step consensus temperature (rho)
		pub fn get_rho( ) -> u64 {
			return Rho::<T>::get();
		}
		pub fn set_rho( rho: u64 ) {
			Rho::<T>::put( rho );
		}

		// -- Get step consensus shift (1/kappa)
		pub fn get_kappa( ) -> u64 {
			return Kappa::<T>::get();
		}
		pub fn set_kappa( kappa: u64 ) {
			Kappa::<T>::put( kappa );
		}

		// -- Get self ownership proportion denominator
		pub fn get_self_ownership( ) -> u64 {
			return T::SelfOwnership::get();
		}

		// -- Activity cuttoff
		pub fn get_activity_cutoff( ) -> u64 {
			return ActivityCutoff::<T>::get();
		}
		pub fn set_activity_cutoff( cuttoff: u64 ) {
			ActivityCutoff::<T>::set( cuttoff );
		}

		// --- Returns the next available network uid.
		// uids increment up to u64:MAX, this allows the chain to
		// have 18,446,744,073,709,551,615 peers before an overflow.
		pub fn get_neuron_count() -> u32 {
			let uid = N::<T>::get();
			uid
		}

		
	}
	
}
