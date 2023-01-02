
use super::*;
use sp_core::U256;

impl<T: Config> Pallet<T> {

    /// ========================
	/// ==== Global Getters ====
	/// ========================
    pub fn set_difficulty( netuid: u16, difficulty: u64 ) { Difficulty::<T>::insert(netuid, difficulty); }
    pub fn set_max_allowed_uids(netuid: u16, max_allowed: u16) { MaxAllowedUids::<T>::insert(netuid, max_allowed); }
    pub fn set_min_difficulty( netuid: u16, min_difficulty: u64 ) { MinDifficulty::<T>::insert( netuid, min_difficulty) }
    pub fn set_max_difficulty( netuid: u16, max_difficulty: u64 ) { MaxDifficulty::<T>::insert( netuid, max_difficulty) }
    pub fn set_max_weight_limit( netuid: u16, max_weight_limit: u16 ) { MaxWeightsLimit::<T>::insert(netuid, max_weight_limit ); }
    pub fn set_last_update_for_neuron(netuid: u16, neuron_uid: u16, update: u64){ LastUpdate::<T>::insert(netuid, neuron_uid, update); }
    pub fn set_min_allowed_weights( netuid: u16, min_allowed_weights: u16 ) { MinAllowedWeights::<T>::insert(netuid, min_allowed_weights ); }
    pub fn set_max_registrations_per_block( netuid: u16, max_registrations: u16 ){ MaxRegistrationsPerBlock::<T>::insert( netuid, max_registrations ); }
    pub fn set_adjustment_interval( netuid: u16, adjustment_interval: u16 ) { AdjustmentInterval::<T>::insert( netuid, adjustment_interval ); }
    pub fn set_last_adjustment_block( netuid: u16, last_adjustment_block: u64 ) { LastAdjustmentBlock::<T>::insert( netuid, last_adjustment_block ); }
    pub fn set_blocks_since_last_step( netuid: u16, blocks_since_last_step: u64 ) { BlocksSinceLastStep::<T>::insert( netuid, blocks_since_last_step ); }
    pub fn set_registrations_this_block( netuid: u16, registrations_this_block: u16 ) { RegistrationsThisBlock::<T>::insert(netuid, registrations_this_block); }
    pub fn set_validator_exclude_quantile( netuid: u16, validator_exclude_quantile: u16 ) { ValidatorExcludeQuantile::<T>::insert(netuid, validator_exclude_quantile); }
    pub fn set_registrations_this_interval( netuid: u16, registrations_this_interval: u16 ) { RegistrationsThisInterval::<T>::insert(netuid, registrations_this_interval); }
    pub fn set_target_registrations_per_interval( netuid: u16, target_registrations_per_interval: u16 ) { TargetRegistrationsPerInterval::<T>::insert(netuid, target_registrations_per_interval); }

    /// ========================
	/// ==== Global Getters ====
	/// ========================
    pub fn get_total_issuance() -> u64 { TotalIssuance::<T>::get() }
    pub fn get_block_emission() -> u64 { BlockEmission::<T>::get() }
    pub fn get_current_block_as_u64( ) -> u64 { TryInto::try_into( system::Pallet::<T>::block_number() ).ok().expect("blockchain will not exceed 2^64 blocks; QED.") }

    /// ============================
	/// ==== Subnetwork Getters ====
	/// ============================
    pub fn get_rho( netuid: u16 ) -> u16{ Rho::<T>::get( netuid ) }
    pub fn get_tempo( netuid:u16 ) -> u16{ Tempo::<T>::get( netuid ) }
    pub fn get_kappa( netuid: u16 ) -> u16 {Kappa::<T>::get( netuid ) }
    pub fn get_min_difficulty( netuid: u16 ) -> u64 { MinDifficulty::<T>::get( netuid ) }
    pub fn get_max_difficulty( netuid: u16 ) -> u64 { MaxDifficulty::<T>::get( netuid ) }
    pub fn get_difficulty_as_u64( netuid: u16 ) -> u64 { Difficulty::<T>::get( netuid ) }
    pub fn get_immunity_period(netuid: u16 ) -> u16 { ImmunityPeriod::<T>::get( netuid ) }
    pub fn get_emission_value( netuid: u16 ) -> u64 { EmissionValues::<T>::get( netuid ) }
    pub fn get_last_mechanism_step_block() -> u64 { LastMechansimStepBlock::<T>::get() }
    pub fn get_activity_cutoff( netuid: u16 ) -> u16 { ActivityCutoff::<T>::get( netuid ) }
    pub fn get_pending_emission( netuid:u16 ) -> u64{ PendingEmission::<T>::get( netuid ) }
    pub fn get_max_weight_limit( netuid: u16) -> u16 { MaxWeightsLimit::<T>::get( netuid ) }    
    pub fn get_max_allowed_uids( netuid: u16 ) -> u16  { MaxAllowedUids::<T>::get( netuid ) }
    pub fn get_min_allowed_weights( netuid:u16 ) -> u16 { MinAllowedWeights::<T>::get( netuid ) }
    pub fn get_adjustment_interval( netuid: u16) -> u16 { AdjustmentInterval::<T>::get( netuid ) }
    pub fn get_n_allowed_validators( netuid: u16 ) -> u16 { NAllowedValidators::<T>::get( netuid ) }
    pub fn get_bonds_moving_average( netuid: u16 ) -> u64 { BondsMovingAverage::<T>::get( netuid ) }
    pub fn get_validator_batch_size( netuid: u16 ) -> u16 { ValidatorBatchSize::<T>::get( netuid ) }
    pub fn get_last_adjustment_block( netuid: u16) -> u64 { LastAdjustmentBlock::<T>::get( netuid ) }
    pub fn get_blocks_since_last_step(netuid:u16 ) -> u64 { BlocksSinceLastStep::<T>::get( netuid ) }
    pub fn get_difficulty( netuid: u16 ) -> U256 { U256::from( Self::get_difficulty_as_u64( netuid ) ) }    
    pub fn get_registrations_this_block( netuid:u16 ) -> u16 { RegistrationsThisBlock::<T>::get( netuid ) }
    pub fn get_validator_epochs_per_reset( netuid: u16 )-> u16 {ValidatorEpochsPerReset::<T>::get( netuid ) }
    pub fn get_validator_sequence_length( netuid: u16 ) -> u16 { ValidatorSequenceLength::<T>::get( netuid ) }
    pub fn get_validator_exclude_quantile( netuid: u16 ) -> u16 { ValidatorExcludeQuantile::<T>::get( netuid ) }
    pub fn get_registrations_this_interval( netuid: u16 ) -> u16 { RegistrationsThisInterval::<T>::get( netuid ) } 
    pub fn get_max_registratations_per_block( netuid: u16 ) -> u16 { MaxRegistrationsPerBlock::<T>::get( netuid ) } 
    pub fn get_target_registrations_per_interval( netuid: u16 ) -> u16 { TargetRegistrationsPerInterval::<T>::get( netuid ) }
    pub fn get_neuron_block_at_registration( netuid: u16, neuron_uid: u16 ) -> u64 { BlockAtRegistration::<T>::get( netuid, neuron_uid )}

}


