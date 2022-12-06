
use super::*;

use sp_core::U256;
use frame_support::inherent::Vec;
use frame_support::sp_std::vec;
use frame_support::storage::IterableStorageMap;

impl<T: Config> Pallet<T> {
    
    /// ==============
	/// ==== Misc ====
	/// ==============
    pub fn get_total_issuance() -> u64 { return TotalIssuance::<T>::get() }
    pub fn get_current_block_as_u64( ) -> u64 {
        let block_as_u64: u64 = TryInto::try_into( system::Pallet::<T>::block_number() ).ok().expect("blockchain will not exceed 2^64 blocks; QED.");
        block_as_u64
    }
    pub fn get_registrations_this_block( ) -> u16 {
        RegistrationsThisBlock::<T>::get()
    }
    pub fn get_max_registratations_per_block( ) -> u16 {
        MaxRegistrationsPerBlock::<T>::get()
    }
    pub fn set_max_registratations_per_block( max_registrations: u16 ){
        MaxRegistrationsPerBlock::<T>::put( max_registrations );
    }

    pub fn get_difficulty(netuid: u16 ) -> U256 {
        return U256::from( Self::get_difficulty_as_u64(netuid) );
    }

    pub fn get_difficulty_as_u64(netuid: u16 ) -> u64 {
         return Difficulty::<T>::get(netuid); 
    }

    /*pub fn get_max_allowed_uids(netuid: u16 ) -> Result<u16, DispatchError>  {
        return MaxAllowedUids::<T>::try_get(netuid).map_err(|_err| Error::<T>::NetworkDoesNotExist.into());
    } */
    pub fn get_max_allowed_uids(netuid: u16) -> u16 {
        return MaxAllowedUids::<T>::get(netuid);
    }
    pub fn set_max_allowed_uids(netuid: u16, max_allowed: u16) {
        return MaxAllowedUids::<T>::insert(netuid, max_allowed);
    }


    // --- Returns the next available uid for the network and increments uid.
	pub fn get_next_uid(netuid: u16) -> u16 {
		let uid = SubnetworkN::<T>::get(netuid);
		assert!(uid < MaxAllowedUids::<T>::get(netuid));  // The system should fail if this is ever reached.
        assert!(uid < u16::MAX);  // The system should fail if this is ever reached.
		SubnetworkN::<T>::insert(netuid, uid + 1); 
		uid
	}

	pub fn get_immunity_period(netuid: u16 ) -> u16 {
		return ImmunityPeriod::<T>::get(netuid);
	}

	pub fn get_total_stake( ) -> u64 {
		return TotalStake::<T>::get();
	}

	pub fn get_stake_pruning_denominator( netuid: u16) -> u16 {
		return StakePruningDenominator::<T>::get(netuid);
	}

	pub fn get_incentive_pruning_denominator(netuid: u16) -> u16 {
		return IncentivePruningDenominator::<T>::get(netuid);
	}

	// --- Returns Option if the u64 converts to a balance
	// use .unwarp if the result returns .some().
	pub fn u64_to_balance(input: u64) -> Option<<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance>
	{
		input.try_into().ok()
	}
    pub fn get_stake_pruning_min(netuid: u16) -> u16 {
		return StakePruningMin::<T>::get(netuid);
	}
	pub fn get_registrations_this_interval( netuid: u16) -> u16 {
		return RegistrationsThisInterval::<T>::get(netuid);
	}
    pub fn remove_global_stake(hotkey: &T::AccountId){
        if Stake::<T>::contains_key(&hotkey){
            Stake::<T>::remove(&hotkey);
        }
    }
    pub fn remove_stake_for_subnet(hotkey: &T::AccountId){
        if Subnets::<T>::contains_key(&hotkey){ //the list of subnets that hotkey is registered on
            let vec_hotkey_subnets = Subnets::<T>::get(&hotkey);
            //
            for i in vec_hotkey_subnets{
                let neuron_uid = Self::get_neuron_for_net_and_hotkey(i, &hotkey);
                S::<T>::remove(i, neuron_uid);
            }
        }
    }
  
    pub fn get_neuron_stake_for_subnetwork(netuid: u16, neuron_uid: u16) -> u64 {
        S::<T>::get(netuid, neuron_uid)
    }
    pub fn get_target_registrations_per_interval(netuid: u16) -> u16 {
		TargetRegistrationsPerInterval::<T>::get(netuid)
	}
    pub fn get_adjustment_interval(netuid: u16) -> u16 {
		AdjustmentInterval::<T>::get(netuid)
	}
    pub fn get_blocks_since_last_step(netuid:u16 ) -> u64 {
		BlocksSinceLastStep::<T>::get(netuid)
	}
    pub fn set_blocks_since_last_step( netuid: u16, blocks_since_last_step: u64 ) {
		BlocksSinceLastStep::<T>::insert( netuid, blocks_since_last_step );
	}
    pub fn get_blocks_per_step(netuid: u16 ) -> u16 {
		BlocksPerStep::<T>::get(netuid)
	}
    // -- Get Block emission.
	pub fn get_block_emission( ) -> u64 {
		return 1000000000;
	}
    pub fn get_last_mechanism_step_block( ) -> u64 {
		return LastMechansimStepBlock::<T>::get();
	}
    pub fn set_pruning_score(netuid:u16, neuron_uid: u16, pruning_score: u16){
        PruningScores::<T>::insert(netuid, neuron_uid, pruning_score);
    }
    pub fn get_min_allowed_weights(netuid:u16 ) -> u16 {
		return MinAllowedWeights::<T>::get(netuid);
	}
    pub fn get_max_weight_limit(netuid: u16) -> u16 {
		return MaxWeightsLimit::<T>::get(netuid);
	}
    pub fn set_max_weight_limit( netuid: u16, max_weight_limit: u16 ) {
        MaxWeightsLimit::<T>::insert(netuid, max_weight_limit )
    }
    pub fn set_min_allowed_weights( netuid: u16, min_allowed_weights: u16 ) {
		MinAllowedWeights::<T>::insert(netuid, min_allowed_weights )
	}
    pub fn  get_bonds_moving_average(netuid: u16) -> u64 {
        BondsMovingAverage::<T>::get(netuid)
    }
    pub fn get_activity_cutoff(netuid: u16) -> u16{
        ActivityCutoff::<T>::get(netuid)
    }
    pub fn get_kappa(netuid: u16) -> u16{
        Kappa::<T>::get(netuid)
    }
    pub fn get_rho(netuid: u16) -> u16{
        Rho::<T>::get(netuid)
    }
    pub fn get_max_allowed_max_min_ratio(netuid: u16) -> u16{
        MaxAllowedMaxMinRatio::<T>::get(netuid)
    }
    pub fn get_validator_batch_size(netuid: u16) -> u16{
        ValidatorBatchSize::<T>::get(netuid)
    }
    pub fn get_validator_sequence_length(netuid: u16) -> u16{
        ValidatorSequenceLength::<T>::get(netuid)
    }
    pub fn get_validator_epochs_per_reset(netuid: u16)-> u16{
        ValidatorEpochsPerReset::<T>::get(netuid)
    }
    pub fn set_validator_exclude_quantile(netuid: u16, validator_exclude_quantile: u16 ) {
		ValidatorExcludeQuantile::<T>::insert(netuid, validator_exclude_quantile)
	}
    pub fn get_validator_exclude_quantile(netuid: u16 ) -> u16 {
		return ValidatorExcludeQuantile::<T>::get(netuid);
	}
    pub fn is_uid_active(netuid: u16, neuron_uid: u16) ->bool{
        Active::<T>::get(netuid, neuron_uid)
    }
    pub fn deactive_neuron(netuid: u16, neuron_uid: u16){
        Active::<T>::remove(netuid, neuron_uid)
    }
    pub fn set_last_update_for_neuron(netuid: u16, neuron_uid: u16, update: u64){
        LastUpdate::<T>::insert(netuid, neuron_uid, update);
    }	
    pub fn get_tempo(netuid:u16) -> u16{
        Tempo::<T>::get(netuid)
    }
    
    /// =========================
	/// ==== Global Accounts ====
	/// =========================
    pub fn is_hotkey_globally_active( hotkey: &T::AccountId ) -> bool { return Coldkeys::<T>::contains_key( hotkey ) }
    pub fn add_global_account( hotkey: &T::AccountId, coldkey: &T::AccountId )  {
        if !Hotkeys::<T>::contains_key( &hotkey ) { 
            Hotkeys::<T>::insert( hotkey.clone(), coldkey.clone() );
            Coldkeys::<T>::insert( coldkey.clone(), hotkey.clone() );
            //Self::increment_global_n();
        }
    }
    pub fn remove_global_account( hotkey: &T::AccountId )  {
        if Hotkeys::<T>::contains_key( &hotkey ) { 
            let coldkey = Coldkeys::<T>::get( &hotkey.clone() );
            Hotkeys::<T>::remove( coldkey.clone() );
            Coldkeys::<T>::remove( hotkey.clone() );
        }
    }
    pub fn get_stake_for_hotkey(hotkey: &T::AccountId) -> u64 {
        Stake::<T>::get(hotkey)
    }
    pub fn add_stake_for_hotkey(hotkey: &T::AccountId, amount: u64){
        Stake::<T>::insert(hotkey, amount);
    }
   
    /// ==============================
	/// ==== Subnetworks Accounts ====
	/// ==============================
    pub fn is_hotkey_registered( netuid:u16, hotkey: &T::AccountId ) -> bool { return Uids::<T>::contains_key( netuid, hotkey ) }
    pub fn get_number_of_subnets()-> u16 {
        let mut number_of_subnets : u16 = 0;
        for (_, _)  in <SubnetworkN<T> as IterableStorageMap<u16, u16>>::iter(){
            number_of_subnets = number_of_subnets + 1;
        }
        return number_of_subnets;
    }
    //pub fn is_subnetwork_uid_active( netuid:u16, uid: u16 ) -> bool { return uid < SubnetworkN::<T>::get( netuid ) }
    //pub fn get_subnetwork_uid( netuid:u16, hotkey: &T::AccountId ) -> u16 { return Uids::<T>::get( netuid, hotkey ) }
    pub fn get_subnetwork_n( netuid:u16 ) -> u16 { return SubnetworkN::<T>::get( netuid ) }
    pub fn increment_subnetwork_n( netuid:u16 ) {let n = SubnetworkN::<T>::get( netuid ); if n < Self::get_max_allowed_uids(netuid) { SubnetworkN::<T>::insert(netuid, n + 1); } }
    pub fn decrement_subnetwork_n( netuid:u16 ) { let n = SubnetworkN::<T>::get( netuid ); if n > 0 { SubnetworkN::<T>::insert(netuid, n - 1); } }
    pub fn add_subnetwork_account( netuid:u16, uid: u16, hotkey: &T::AccountId ) { 
        Keys::<T>::insert( netuid, uid, hotkey.clone() ); 
        Uids::<T>::insert( netuid, hotkey.clone(), uid );
        //Self::increment_subnetwork_n( netuid );
    }
    pub fn remove_subnetwork_account( netuid:u16, uid: u16 ) { 
        let hotkey = Keys::<T>::get( netuid, uid );
        Uids::<T>::remove( netuid, hotkey.clone() );
        Keys::<T>::remove( netuid, uid ); 
        Self::decrement_subnetwork_n( netuid );
    }
    pub fn get_coldkey_for_hotkey(hotkey:  &T::AccountId) ->  T::AccountId {
        return Hotkeys::<T>::get(hotkey);
    }

    pub fn get_hotkey_for_coldkey(coldkey: &T::AccountId) -> T::AccountId {
        return Coldkeys::<T>::get(coldkey);
    }

    pub fn get_subnets_for_hotkey(hotkey: T::AccountId) -> Vec<u16> {
        let subnets: Vec<u16> = Subnets::<T>::get(hotkey);
        subnets
    }

    pub fn get_hotkey_for_net_and_neuron(netuid: u16, neuron_uid: u16) -> T::AccountId {
        return Keys::<T>::get(netuid, neuron_uid);
    }

    pub fn get_neuron_for_net_and_hotkey(netuid: u16, hotkey: &T::AccountId) -> u16 {
        return Uids::<T>::get(netuid, &hotkey);
    }

    pub fn increment_subnets_for_hotkey(netuid: u16, hotkey: &T::AccountId){

        let mut vec_new_hotkey_subnets = vec![];

        if Subnets::<T>::contains_key(&hotkey){ //update the list of subnets that hotkey is registered on
            vec_new_hotkey_subnets = Subnets::<T>::take(&hotkey);
            
            vec_new_hotkey_subnets.push(netuid);
            Subnets::<T>::insert(&hotkey, vec_new_hotkey_subnets); 
        } else {
            vec_new_hotkey_subnets.push(netuid); 
            Subnets::<T>::insert(&hotkey, vec_new_hotkey_subnets); 
        }
    }
    //check if horkey is registered on any network
    pub fn is_hotkey_registered_any(hotkey:  &T::AccountId)-> bool {
        return Subnets::<T>::contains_key( hotkey)
    }

    pub fn get_hotkey_stake_for_subnet(netuid: u16, hotkey:  &T::AccountId) -> u64{

        let neuron_uid = Self::get_neuron_for_net_and_hotkey(netuid, hotkey);
        S::<T>::get(netuid, neuron_uid)
    }

    pub fn is_uid_exist(netuid: u16, uid: u16) -> bool {
        return  Keys::<T>::contains_key(netuid, uid);
    }

    pub fn get_weights_for_neuron(netuid: u16, neuron_uid: u16) -> Vec<u16>  {
            
        let mut w : Vec<u16> = vec![ 0; Self::get_subnetwork_n(netuid) as usize ];
        let weights = Weights::<T>::get(netuid, neuron_uid);
		for (uid_j, weights_ij) in weights.iter(){
			w[ *uid_j as usize ] = *weights_ij;
		}
		return w;
    } 

    pub fn get_emission_ratio(netuid: u16) -> u64 {
        EmissionValues::<T>::get(netuid)
    }

    pub fn get_neuron_metadata(neuron_id: u16) -> NeuronMetadataOf {
        return NeuronsMetaData::<T>::get(neuron_id).unwrap();
    }

    pub fn if_tempo_is_valid(tempo: u16) -> bool {
        tempo < u16::MAX
    }

    /// ==============================
	/// ==== Subnetworks Consensus ===
	/// ==============================
    pub fn remove_emission_from_subnet(netuid:u16, neuron_uid: u16){
        Emission::<T>::remove(netuid, neuron_uid);
    }
    pub fn if_emission_is_set_for_neuron(netuid: u16, neuron_uid: u16) -> bool{
        Emission::<T>::contains_key(netuid, neuron_uid)
    }
    //
    pub fn remove_dividend_from_subnet(netuid:u16, neuron_uid: u16){
        Dividends::<T>::remove(netuid, neuron_uid);
    }
    pub fn if_dividend_is_set_for_neuron(netuid: u16, neuron_uid: u16) -> bool{
        Dividends::<T>::contains_key(netuid, neuron_uid)
    }
    //
    pub fn remove_consensus_from_subnet(netuid:u16, neuron_uid: u16){
        Consensus::<T>::remove(netuid, neuron_uid);
    }
    pub fn if_consensus_is_set_for_neuron(netuid: u16, neuron_uid: u16) -> bool{
        Consensus::<T>::contains_key(netuid, neuron_uid)
    }
    //
    pub fn remove_incentive_from_subnet(netuid:u16, neuron_uid: u16){
        Incentive::<T>::remove(netuid, neuron_uid);
    }
    pub fn if_incentive_is_set_for_neuron(netuid: u16, neuron_uid: u16) -> bool{
        Incentive::<T>::contains_key(netuid, neuron_uid)
    }
    //
    pub fn remove_trust_from_subnet(netuid:u16, neuron_uid: u16){
        Trust::<T>::remove(netuid, neuron_uid);
    }
    pub fn if_trust_is_set_for_neuron(netuid: u16, neuron_uid: u16) -> bool{
        Trust::<T>::contains_key(netuid, neuron_uid)
    }
    //
    pub fn remove_rank_from_subnet(netuid:u16, neuron_uid: u16){
        Rank::<T>::remove(netuid, neuron_uid);
    }
    pub fn if_rank_is_set_for_neuron(netuid: u16, neuron_uid: u16) -> bool{
        Rank::<T>::contains_key(netuid, neuron_uid)
    }
    //
    pub fn remove_pruning_score_from_subnet(netuid:u16, neuron_uid: u16){
        PruningScores::<T>::remove(netuid, neuron_uid);
    }
    //
    pub fn remove_bonds_from_subnet(netuid:u16, neuron_uid: u16){
        Bonds::<T>::remove(netuid, neuron_uid);
    }
    pub fn if_bonds_is_set_for_neuron(netuid: u16, neuron_uid: u16) -> bool{
        Bonds::<T>::contains_key(netuid, neuron_uid)
    }
    //
    pub fn remove_weights_from_subnet(netuid:u16, neuron_uid: u16){
        Weights::<T>::remove(netuid, neuron_uid);
    }
    pub fn if_weights_is_set_for_neuron(netuid: u16, neuron_uid: u16) -> bool{
        Weights::<T>::contains_key(netuid, neuron_uid)
    }
}


