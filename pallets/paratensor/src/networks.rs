use super::*;
use frame_support::{sp_std::vec};
use sp_std::vec::Vec;
use crate::system::ensure_root;
use frame_support::storage::IterableStorageMap;
use frame_support::storage::IterableStorageDoubleMap;

impl<T: Config> Pallet<T> { 


    /// ---- The implementation for the extrinsic add_network.
    ///
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
    pub fn do_add_network( 
        origin: T::Origin, 
        netuid: u16, 
        tempo: u16, 
        modality: u16 
    ) -> dispatch::DispatchResult{

        // --- 1. Ensure this is a sudo caller.
        ensure_root( origin )?;

        // --- 2. Ensure this subnetwork does not already exist.
        ensure!( !Self::if_subnet_exist(netuid), Error::<T>::NetworkExist );

        // --- 3. Ensure the modality is valid.
        ensure!( Self::if_modality_is_valid( modality ), Error::<T>::InvalidModality );

        // --- 4. Ensure the tempo is valid.
        ensure!( Self::if_tempo_is_valid( tempo ), Error::<T>::InvalidTempo );

        // --- 5. Initialize the network and all its parameters.
        Self::init_new_network( netuid, tempo, modality );
        
        // --- 6. Emit the new network event.
        Self::deposit_event( Event::NetworkAdded( netuid, modality ) );

        // --- 7. Ok and return.
        Ok(())
    }

    /// ---- The implementation for the extrinsic remove_network.
    ///
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
    pub fn do_remove_network( origin: T::Origin, netuid: u16 ) -> dispatch::DispatchResult {

        // --- 1. Ensure the function caller it Sudo.
        ensure_root( origin )?;

        // --- 2. Ensure the network to be removed exists.
        ensure!( Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist );

        // --- 3. Explicitly erase the network and all its parameters.
        Self::remove_network( netuid );
    
        // --- 4. Emit the event.
        Self::deposit_event( Event::NetworkRemoved( netuid ) );
        
        // --- 5. Ok and return.
        Ok(())
    }

    /// Initializes a new subnetwork under netuid with parameters.
    pub fn init_new_network( netuid:u16, tempo:u16, modality:u16 ){

        // --- 1. Set network to 0 size.
        SubnetworkN::<T>::insert( netuid, 0 );

        // --- 2. Set this network uid to alive.
        NetworksAdded::<T>::insert( netuid, true );
        
        // --- 3. Fill tempo memory item.
        Tempo::<T>::insert( netuid, tempo );

        // --- 4 Fill modality item.
        NetworkModality::<T>::insert( netuid, modality );

        // --- 5. Increase total network count.
        TotalNetworks::<T>::mutate( |n| *n += 1 );

        // --- 6. Set all default values **explicitly**.
        Self::set_default_values_for_all_parameters( netuid );
    }

      /// Removes the network (netuid) and all of its parameters.
      pub fn remove_network( netuid:u16 ) {

        // --- 1. Remove network count.
        SubnetworkN::<T>::remove( netuid );

        // --- 2. Remove network modality storage.
        NetworkModality::<T>::remove( netuid );

        // --- 3. Set False the network.
        NetworksAdded::<T>::insert( netuid, false );

        // --- 4. Erase all memory associated with the network.
        Self::erase_all_network_data( netuid );

        // --- 5. Remove subnetworks for all hotkeys.
        Self::remove_subnet_for_all_hotkeys( netuid );

        // --- 6. Decrement the network counter.
        TotalNetworks::<T>::mutate(|val| *val -= 1);
    }


    /// Explicitly sets all network parameters to their default values.
    /// Note: this is required because, although there are defaults, they do not come through on all calls.
    pub fn set_default_values_for_all_parameters(netuid: u16){
        // Make network parameters explicit.
        if !Tempo::<T>::contains_key(netuid) { Tempo::<T>::insert(netuid, Tempo::<T>::get(netuid));}
        if !Kappa::<T>::contains_key(netuid) { Kappa::<T>::insert(netuid, Kappa::<T>::get(netuid));}
        if !Difficulty::<T>::contains_key(netuid) { Difficulty::<T>::insert(netuid, Difficulty::<T>::get(netuid));}
        if !MaxAllowedUids::<T>::contains_key(netuid) { MaxAllowedUids::<T>::insert(netuid, MaxAllowedUids::<T>::get(netuid));}
        if !ImmunityPeriod::<T>::contains_key(netuid) { ImmunityPeriod::<T>::insert(netuid, ImmunityPeriod::<T>::get(netuid));}
        if !ActivityCutoff::<T>::contains_key(netuid) { ActivityCutoff::<T>::insert(netuid, ActivityCutoff::<T>::get(netuid));}
        if !EmissionValues::<T>::contains_key(netuid) { EmissionValues::<T>::insert(netuid, EmissionValues::<T>::get(netuid));}   
        if !StakePruningMin::<T>::contains_key(netuid) { StakePruningMin::<T>::insert(netuid, StakePruningMin::<T>::get(netuid));}
        if !MaxWeightsLimit::<T>::contains_key(netuid) { MaxWeightsLimit::<T>::insert(netuid, MaxWeightsLimit::<T>::get(netuid));}
        if !ValidatorEpochLen::<T>::contains_key(netuid) { ValidatorEpochLen::<T>::insert(netuid, ValidatorEpochLen::<T>::get(netuid));}
        if !MinAllowedWeights::<T>::contains_key(netuid) { MinAllowedWeights::<T>::insert(netuid, MinAllowedWeights::<T>::get(netuid)); }
        if !ValidatorBatchSize::<T>::contains_key(netuid) { ValidatorBatchSize::<T>::insert(netuid, ValidatorBatchSize::<T>::get(netuid));}
        if !MaxAllowedMaxMinRatio::<T>::contains_key(netuid) { MaxAllowedMaxMinRatio::<T>::insert(netuid, MaxAllowedMaxMinRatio::<T>::get(netuid));}
        if !ValidatorEpochsPerReset::<T>::contains_key(netuid) { ValidatorEpochsPerReset::<T>::insert(netuid, ValidatorEpochsPerReset::<T>::get(netuid));}
        if !ValidatorSequenceLength::<T>::contains_key(netuid) { ValidatorSequenceLength::<T>::insert(netuid, ValidatorSequenceLength::<T>::get(netuid));}
        if !RegistrationsThisInterval::<T>::contains_key(netuid) { RegistrationsThisInterval::<T>::insert(netuid, RegistrationsThisInterval::<T>::get(netuid));}
        if !IncentivePruningDenominator::<T>::contains_key(netuid) { IncentivePruningDenominator::<T>::insert(netuid, IncentivePruningDenominator::<T>::get(netuid));}
    }

    /// Explicitly erases all data associated with this network.
    pub fn erase_all_network_data(netuid: u16){

        // --- 1. Remove incentive mechanism memory.
        S::<T>::remove_prefix( netuid, None );
        Uids::<T>::remove_prefix( netuid, None );
        Keys::<T>::remove_prefix( netuid, None );
        Rank::<T>::remove_prefix( netuid, None );
        Trust::<T>::remove_prefix( netuid, None );
        Bonds::<T>::remove_prefix( netuid, None );
        Active::<T>::remove_prefix( netuid, None );
        Weights::<T>::remove_prefix( netuid, None );
        Emission::<T>::remove_prefix( netuid, None );
        Incentive::<T>::remove_prefix( netuid, None );
        Consensus::<T>::remove_prefix( netuid, None );
        Dividends::<T>::remove_prefix( netuid, None );
        PruningScores::<T>::remove_prefix( netuid, None );

        // --- 2. Erase network parameters.
        Tempo::<T>::remove( netuid );
        Kappa::<T>::remove( netuid );
        Difficulty::<T>::remove( netuid );
        MaxAllowedUids::<T>::remove( netuid );
        ImmunityPeriod::<T>::remove( netuid );
        ActivityCutoff::<T>::remove( netuid );
        EmissionValues::<T>::remove( netuid );
        StakePruningMin::<T>::remove( netuid );
        MaxWeightsLimit::<T>::remove( netuid );
        ValidatorEpochLen::<T>::remove( netuid );
        MinAllowedWeights::<T>::remove( netuid );
        ValidatorBatchSize::<T>::remove( netuid );
        MaxAllowedMaxMinRatio::<T>::remove( netuid );
        ValidatorEpochsPerReset::<T>::remove( netuid );
        ValidatorSequenceLength::<T>::remove( netuid );
        RegistrationsThisInterval::<T>::remove( netuid );
        IncentivePruningDenominator::<T>::remove( netuid );
    }

    /// ---- The implementation for the extrinsic set_emission_values.
    ///
    /// # Args:
    /// 	* 'origin': (<T as frame_system::Config>Origin):
    /// 		- Must be sudo.
    ///
   	/// 	* `netuids` (Vec<u16>):
	/// 		- A vector of network uids values. This must include all netuids.
	///
	/// 	* `emission` (Vec<u64>):
	/// 		- The emission values associated with passed netuids in order.
    ///
    /// # Event:
    /// 	* NetworkRemoved;
    /// 		- On the successfull removing of this network.
    ///
    /// # Raises:
    /// 	* 'EmissionValuesDoesNotMatchNetworks':
    /// 		- Attempting to remove a non existent network.
    ///
    pub fn do_set_emission_values( 
        origin: T::Origin, 
        netuids: Vec<u16>,
        emission: Vec<u64>
    ) -> dispatch::DispatchResult {

        // --- 1. Ensure caller is sudo.
        ensure_root( origin )?;

        // --- 2. Ensure emission values match up to network uids.
        ensure!( netuids.len() == emission.len(), Error::<T>::WeightVecNotEqualSize );

        // --- 3. Ensure we are setting emission for all networks. 
        ensure!( netuids.len() as u16 == TotalNetworks::<T>::get(), Error::<T>::NotSettingEnoughWeights );

        // --- 4. Ensure the passed uids contain no duplicates.
        ensure!( !Self::has_duplicate_netuids( &netuids ), Error::<T>::DuplicateUids );

        // --- 5. Ensure that the passed uids are valid for the network.
        ensure!( !Self::contains_invalid_netuids( &netuids ), Error::<T>::InvalidUid );

        // --- 6. check if sum of emission rates is equal to 1.
        ensure!( emission.iter().sum::<u64>() as u64 == BlockEmission::<T>::get(), Error::<T>::InvalidEmissionValues);

        // --- 7. Add emission values for each network
        Self::set_emission_values( &netuids, &emission );

        // --- 8. Add emission values for each network
        Self::deposit_event( Event::EmissionValuesSet() );

        // --- 9. Ok and return.
        Ok(())
    }

    /// Returns true if the items contain duplicates.
    fn has_duplicate_netuids(netuids: &Vec<u16>) -> bool {
        let mut parsed: Vec<u16> = Vec::new();
        for item in netuids {
            if parsed.contains(&item) { return true; }
            parsed.push(item.clone());
        }
        return false;
    }

    /// Checks for any invalid netuids on this network.
    pub fn contains_invalid_netuids( netuids: &Vec<u16> ) -> bool {
        for netuid in netuids {
            if !Self::if_subnet_exist( *netuid ) {
                return true;
            }
        }
        return false;
    }

    // Set emission values on networks.
    pub fn set_emission_values( netuids: &Vec<u16>, emission: &Vec<u64> ){
        for (i, netuid_i) in netuids.iter().enumerate() {
            EmissionValues::<T>::insert( netuid_i, emission[i] );
        }
    }

    // Checks to see if a subnet exists.
    pub fn if_subnet_exist(netuid: u16) -> bool{
        return NetworksAdded::<T>::get( netuid );
    }

    pub fn if_modality_is_valid(modality: u16) -> bool{
        let allowed_values: Vec<u16> = vec![0, 1, 2];
        return allowed_values.contains(&modality);
    } 

    pub fn remove_subnet_for_all_hotkeys(netuid: u16){

        let mut vec_new_hotkey_subnets : Vec<u16>;

        for (hotkey_i, vec)  in <Subnets<T> as IterableStorageMap<T::AccountId, Vec<u16>>>::iter() {
            vec_new_hotkey_subnets = vec.clone();
            //hotkey_to_be_updated.push(hotkey_i.clone());
            for (i, val) in vec.iter().enumerate(){
                if *val == netuid{
                    vec_new_hotkey_subnets.remove(i);
                }
            }
            Subnets::<T>::insert(hotkey_i, vec_new_hotkey_subnets)
        }
        /* check if the hotkey is deregistred from all networks, 
        if so, then we need to transfer stake from hotkey to cold key */
        for (hotkey_i, _)  in <Subnets<T> as IterableStorageMap<T::AccountId, Vec<u16>>>::iter() {
            let vec_subnets_for_pruning_hotkey: Vec<u16> = Subnets::<T>::get(&hotkey_i); // a list of subnets that hotkey is registered on.
            if vec_subnets_for_pruning_hotkey.len() == 0 { 
                // we need to remove all stakes since this hotkey is not staked in any other networks
                    // These funds are deposited back into the coldkey account so that no funds are destroyed. 
                    //
                    let coldkey_to_add_stake = GlobalAccounts::<T>::get(&hotkey_i);
                    let stake_to_remove = Stake::<T>::get(&hotkey_i);
                    Self::add_balance_to_coldkey_account( &coldkey_to_add_stake, Self::u64_to_balance(stake_to_remove).unwrap());
                    Self::decrease_total_stake( stake_to_remove );
                    Self::remove_global_stake(&hotkey_i);
                    //
                    Subnets::<T>::remove(hotkey_i);
            }
        }
    }


    pub fn clear_last_update_for_subnet(netuid: u16){
        let mut exist = false;
        for (_uid_i, _) in <LastUpdate<T> as IterableStorageDoubleMap<u16, u16, u64 >>::iter_prefix( netuid ) {
            exist = true;
        }
        if exist { LastUpdate::<T>::remove_prefix(netuid, None); }
       
    }

    pub fn clear_min_allowed_weight_for_subnet(netuid: u16){

        if MinAllowedWeights::<T>::contains_key(netuid)
            {MinAllowedWeights::<T>::remove(netuid);}
    }

    pub fn clear_max_weight_limit_for_subnet(netuid: u16){
        if MaxWeightsLimit::<T>::contains_key(netuid)
            {MaxWeightsLimit::<T>::remove(netuid);}
    }

    pub fn clear_max_allowed_max_min_ratio_for_subnet(netuid: u16){
        if MaxAllowedMaxMinRatio::<T>::contains_key(netuid)
            {MaxAllowedMaxMinRatio::<T>::remove(netuid);}
    }

    pub fn clear_tempo_for_subnet(netuid: u16){
        if Tempo::<T>::contains_key(netuid)
            {Tempo::<T>::remove(netuid);}
    }

    pub fn clear_difficulty_for_subnet(netuid: u16){
        if Difficulty::<T>::contains_key(netuid)
            {Difficulty::<T>::remove(netuid);}
    }

    pub fn clear_kappa_for_subnet(netuid: u16){
        if Kappa::<T>::contains_key(netuid)
            {Kappa::<T>::remove(netuid);}
    }

    pub fn clear_max_allowed_uids_for_subnet(netuid: u16){
        if MaxAllowedUids::<T>::contains_key(netuid)
            {MaxAllowedUids::<T>::remove(netuid);}
    }

    pub fn clear_validator_batch_size_for_subnet(netuid: u16){
       if ValidatorBatchSize::<T>::contains_key(netuid)
            { ValidatorBatchSize::<T>::remove(netuid);}
    }

    pub fn clear_validator_seq_length_for_subnet(netuid: u16){
        if ValidatorSequenceLength::<T>::contains_key(netuid)
            {ValidatorSequenceLength::<T>::remove(netuid);}
    }

    pub fn clear_validator_epoch_length_for_subnet(netuid: u16){
        if ValidatorEpochLen::<T>::contains_key(netuid)
            {ValidatorEpochLen::<T>::remove(netuid);}
    }

    pub fn clear_validator_epoch_per_reset_for_subnet(netuid: u16){
        if ValidatorEpochsPerReset::<T>::contains_key(netuid)
            {ValidatorEpochsPerReset::<T>::remove(netuid);}
    }

    pub fn clear_incentive_pruning_denom_for_subnet(netuid: u16){
        if IncentivePruningDenominator::<T>::contains_key(netuid)
            {IncentivePruningDenominator::<T>::remove(netuid);}
    }

    pub fn clear_stake_pruning_denom_for_subnet(netuid: u16){
        if StakePruningDenominator::<T>::contains_key(netuid)
            {StakePruningDenominator::<T>::remove(netuid);}
    }

    pub fn clear_stake_pruning_min_for_subnet(netuid: u16){
        if StakePruningMin::<T>::contains_key(netuid)
            {StakePruningMin::<T>::remove(netuid);}
    }

    pub fn clear_immunity_period_for_subnet(netuid: u16){
        if ImmunityPeriod::<T>::contains_key(netuid)
            {ImmunityPeriod::<T>::remove(netuid);}
    }

    pub fn clear_activity_cutoff_for_subnet(netuid: u16){
        if ActivityCutoff::<T>::contains_key(netuid)
            {ActivityCutoff::<T>::remove(netuid);}
    }

    pub fn clear_reg_this_interval_for_subnet(netuid: u16){
        if RegistrationsThisInterval::<T>::contains_key(netuid)
            {RegistrationsThisInterval::<T>::remove(netuid);}
    }

    pub fn remove_uids_for_subnet(netuid: u16){
        let mut exist = false;
        for (_uid_i, _) in <Uids<T> as IterableStorageDoubleMap<u16, T::AccountId, u16 >>::iter_prefix( netuid ) {
            exist = true;
        }
        if exist { Uids::<T>::remove_prefix(netuid, None); }
    }

    pub fn remove_keys_for_subnet(netuid: u16){
        let mut exist = false;
        for (_uid_i, _) in <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId >>::iter_prefix( netuid ) {
            exist = true;
        }
        if exist { Keys::<T>::remove_prefix(netuid, None); }
    }

    pub fn remove_weights_for_subnet(netuid: u16){
        let mut exist = false;
        for (_uid_i, _) in <Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix( netuid ) {
            exist = true;
        }
        if exist { Weights::<T>::remove_prefix(netuid, None);}
    }

    pub fn remove_bonds_for_subnet(netuid: u16){
        let mut exist = false;
        for (_uid_i, _) in <Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix( netuid ) {
            exist = true;
        }
        if exist { Bonds::<T>::remove_prefix(netuid, None); }
    }

    pub fn remove_active_for_subnet(netuid: u16){
        let mut exist = false;
        for (_uid_i, _) in <Active<T> as IterableStorageDoubleMap<u16, u16, bool>>::iter_prefix( netuid ) {
            exist = true;
        }
        if exist {Active::<T>::remove_prefix(netuid, None);}
    }  

    pub fn remove_rank_for_subnet(netuid: u16){
        let mut exist = false;
        for (_uid_i, _) in <Rank<T> as IterableStorageDoubleMap<u16, u16, u16>>::iter_prefix( netuid ) {
            exist = true;
        }
        if exist {Rank::<T>::remove_prefix(netuid, None);}
    }

    pub fn remove_trust_for_subnet(netuid: u16){
        let mut exist = false;
        for (_uid_i, _) in <Trust<T> as IterableStorageDoubleMap<u16, u16, u16>>::iter_prefix( netuid ) {
            exist = true;
        }
        if exist {Trust::<T>::remove_prefix(netuid, None);}
    }

    pub fn remove_incentive_for_subnet(netuid: u16){
        let mut exist = false;
        for (_uid_i, _) in <Incentive<T> as IterableStorageDoubleMap<u16, u16, u16>>::iter_prefix( netuid ) {
            exist = true;
        }
        if exist {Incentive::<T>::remove_prefix(netuid, None);}
    }

    pub fn remove_consensus_for_subnet(netuid: u16){
        let mut exist = false;
        for (_uid_i, _) in <Consensus<T> as IterableStorageDoubleMap<u16, u16, u16>>::iter_prefix( netuid ) {
            exist = true;
        }
        if exist {Consensus::<T>::remove_prefix(netuid, None);}
    }

    pub fn remove_dividends_for_subnet(netuid: u16){
        let mut exist = false;
        for (_uid_i, _) in <Dividends<T> as IterableStorageDoubleMap<u16, u16, u16>>::iter_prefix( netuid ) {
            exist = true;
        }
        if exist {Dividends::<T>::remove_prefix(netuid, None);}
    }

    pub fn remove_emission_for_subnet(netuid: u16){
        let mut exist = false;
        for (_uid_i, _) in <Emission<T> as IterableStorageDoubleMap<u16, u16, u64>>::iter_prefix( netuid ) {
            exist = true;
        }
        if exist {Emission::<T>::remove_prefix(netuid, None);}
    }

    pub fn remove_pruning_score_for_subnet(netuid: u16){
        let mut exist = false;
        for (_uid_i, _) in <PruningScores<T> as IterableStorageDoubleMap<u16, u16, u16>>::iter_prefix( netuid ) {
            exist = true;
        }
        if exist {PruningScores::<T>::remove_prefix(netuid, None);}
        
    }
    
    pub fn remove_all_stakes_for_subnet(netuid: u16){
      
        let mut exist = false;
        for (_uid_i, _) in <S<T> as IterableStorageDoubleMap<u16, u16, u64>>::iter_prefix( netuid ) {
            exist = true;
        }
        if exist {S::<T>::remove_prefix(netuid, None);}
    }

}