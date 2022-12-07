use super::*;
use frame_support::{sp_std::vec};
use sp_std::vec::Vec;
use crate::system::ensure_root;
use frame_support::storage::IterableStorageMap;
use frame_support::storage::IterableStorageDoubleMap;

impl<T: Config> Pallet<T> { 
    pub fn do_add_network(origin: T::Origin, netuid: u16, tempo: u16, modality: u16) -> dispatch::DispatchResult{
        /*TO DO:
        1. check if caller is sudo account
        2. check if network does not exist
        3. check if modality is valid
        4. check if tempo is valid
        5. add network and modality 
        6. add defualt value for all other parameters to the storage*/

        // 1. if caller is sudo account
        ensure_root( origin )?;

        // 2. check if network exist
        ensure!(!Self::if_subnet_exist(netuid), Error::<T>::NetworkExist);

        // 3. if modality is valid
        ensure!(Self::if_modality_is_valid(modality), Error::<T>::InvalidModality);

        // 4. if tempo is valid
        ensure!(Self::if_tempo_is_valid(tempo), Error::<T>::InvalidTempo);

        //5. Add network
        SubnetworkN::<T>::insert(netuid, 0); //initial size for each network is 0
        TotalNetworks::<T>::mutate(|val| *val += 1);
        NetworksAdded::<T>::insert(netuid, true);
        //
        Tempo::<T>::insert(netuid, tempo);
        NetworkModality::<T>::insert(netuid, modality);

        // 5. Add default value for all other parameters
        Self::set_default_values_for_all_parameters(netuid);
        
        // ---- Emit the event.
        Self::deposit_event(Event::NetworkAdded(netuid, modality));

        // --- Emit the event and return ok.
        Ok(())
    }
    
    pub fn do_remove_network(origin: T::Origin, netuid: u16) -> dispatch::DispatchResult{
        /* TO DO:
        1. check if caller is sudo account
        2. check if network exist
        3. remove network and modality
        4. update all other storage*/

        // 1. if caller is sudo account
        ensure_root( origin )?;

        // 2. check if network exist
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);

        // 3. remove network and mdality
        SubnetworkN::<T>::remove(netuid);
        NetworkModality::<T>::remove(netuid);
        TotalNetworks::<T>::mutate(|val| *val -= 1);
        NetworksAdded::<T>::insert(netuid, false);

        // 4. update all other storage
        Self::remove_subnet_for_all_hotkeys(netuid);
        Self::clear_last_update_for_subnet(netuid);
        Self::clear_min_allowed_weight_for_subnet(netuid);
        Self::clear_max_weight_limit_for_subnet(netuid);
        Self::clear_max_allowed_max_min_ratio_for_subnet(netuid);
        Self::clear_tempo_for_subnet(netuid);
        Self::clear_difficulty_for_subnet(netuid);
        Self::clear_kappa_for_subnet(netuid);
        Self::clear_max_allowed_uids_for_subnet(netuid);
        Self::clear_validator_batch_size_for_subnet(netuid);
        Self::clear_validator_seq_length_for_subnet(netuid);
        Self::clear_validator_epoch_length_for_subnet(netuid);
        Self::clear_validator_epoch_per_reset_for_subnet(netuid);
        Self::clear_incentive_pruning_denom_for_subnet(netuid);
        Self::clear_stake_pruning_denom_for_subnet(netuid);
        Self::clear_stake_pruning_min_for_subnet(netuid);
        Self::clear_immunity_period_for_subnet(netuid);
        Self::clear_activity_cutoff_for_subnet(netuid);
        Self::clear_reg_this_interval_for_subnet(netuid);
        //
        Self::remove_uids_for_subnet(netuid);
        Self::remove_keys_for_subnet(netuid);
        Self::remove_weights_for_subnet(netuid);
        Self::remove_bonds_for_subnet(netuid);
        Self::remove_active_for_subnet(netuid);
        Self::remove_rank_for_subnet(netuid);
        Self::remove_trust_for_subnet(netuid);
        Self::remove_incentive_for_subnet(netuid);
        Self::remove_consensus_for_subnet(netuid);
        Self::remove_dividends_for_subnet(netuid);
        Self::remove_emission_for_subnet(netuid);
        Self::remove_pruning_score_for_subnet(netuid); 
        Self::remove_all_stakes_for_subnet(netuid);

        // --- Emit the event and return ok.
        Self::deposit_event(Event::NetworkRemoved(netuid ));
        //
        Ok(())
    }

    pub fn do_set_emission_values( origin: T::Origin, emission_values: Vec<(u16,u64)>) -> dispatch::DispatchResult{
        /*TO DO:
        1. check if caller is sudo account
        2. check if we receive emission rate for all existing networks (not more, not less)
        3. check if sum of emission rates is equal to 1.
        5. add emission values  */
        //
        // 1. if caller is sudo account
        ensure_root( origin )?;

        // 2. check if we have received emission values for all existing networks and not more
        ensure!(Self::if_emission_values_match(&emission_values), Error::<T>::EmissionValuesDoesNotMatchNetworks);

        // 3. check if sum of emission rates is equal to 1.
        ensure!(Self::if_sum_emission_values(&emission_values), Error::<T>::InvalidEmissionValues);

        // 4. Add emission values for each network
        Self::add_emission_values(&emission_values);

        Self::deposit_event(Event::EmissionValuesSet());
        Ok(())
    }

    // helper functions
    pub fn add_emission_values(emission_values: &Vec<(u16, u64)>){
        
        for (netuid_i, emission_i) in emission_values.iter(){
            EmissionValues::<T>::insert(netuid_i, emission_i);
        }
    }

    pub fn if_sum_emission_values(emission_values: &Vec<(u16, u64)>) -> bool{
        let mut emission_values_sum: u64 = 0;
        
        for (_, emission_i) in emission_values.iter(){ 
            emission_values_sum = emission_values_sum + emission_i;
        } 
        if emission_values_sum == BlockEmission::<T>::get() {return true;}
        else {return false;}
    } 

    pub fn if_emission_values_match(emission_values: &Vec<(u16, u64)>) -> bool{ 
        
        let tota_nets = TotalNetworks::<T>::get();
        let mut nets: Vec<u16> = vec![];

        for (netuid_j, _) in emission_values.iter() {nets.push(*netuid_j);} 

        if nets.len() as u16 != tota_nets {return false;}
        
        for (uid_i, _) in <SubnetworkN<T> as IterableStorageMap<u16, u16 >>::iter() {
            if !nets.contains(&uid_i) {return false;}  

        }
        for (_, val) in nets.iter().enumerate(){
            if NetworksAdded::<T>::get(val) == false {return false;}
        }
        return true;
    }

    pub fn if_subnet_exist(netuid: u16) -> bool{
        return  SubnetworkN::<T>::contains_key(netuid);
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
                    let coldkey_to_add_stake = Coldkeys::<T>::get(&hotkey_i);
                    let stake_to_remove = Stake::<T>::get(&hotkey_i);
                    Self::add_balance_to_coldkey_account( &coldkey_to_add_stake, Self::u64_to_balance(stake_to_remove).unwrap());
                    Self::decrease_total_stake( stake_to_remove );
                    Self::remove_global_stake(&hotkey_i);
                    //
                    Subnets::<T>::remove(hotkey_i);
            }
        }
    }

    pub fn set_default_values_for_all_parameters(netuid: u16){
        if !MinAllowedWeights::<T>::contains_key(netuid)
            { MinAllowedWeights::<T>::insert(netuid, MinAllowedWeights::<T>::get(netuid));}
        
        if !EmissionValues::<T>::contains_key(netuid)
            { EmissionValues::<T>::insert(netuid, EmissionValues::<T>::get(netuid));}   

        if !MaxWeightsLimit::<T>::contains_key(netuid)
            { MaxWeightsLimit::<T>::insert(netuid, MaxWeightsLimit::<T>::get(netuid));}

        if !MaxAllowedMaxMinRatio::<T>::contains_key(netuid)
            { MaxAllowedMaxMinRatio::<T>::insert(netuid, MaxAllowedMaxMinRatio::<T>::get(netuid));}

        if !Tempo::<T>::contains_key(netuid)
            { Tempo::<T>::insert(netuid, Tempo::<T>::get(netuid));}

        if !Difficulty::<T>::contains_key(netuid)
            { Difficulty::<T>::insert(netuid, Difficulty::<T>::get(netuid));}

        if !Kappa::<T>::contains_key(netuid)
            { Kappa::<T>::insert(netuid, Kappa::<T>::get(netuid));}

        if !MaxAllowedUids::<T>::contains_key(netuid)
            { MaxAllowedUids::<T>::insert(netuid, MaxAllowedUids::<T>::get(netuid));}

        if !ValidatorBatchSize::<T>::contains_key(netuid)
            { ValidatorBatchSize::<T>::insert(netuid, ValidatorBatchSize::<T>::get(netuid));}

        if !ValidatorSequenceLength::<T>::contains_key(netuid)
            { ValidatorSequenceLength::<T>::insert(netuid, ValidatorSequenceLength::<T>::get(netuid));}

        if !ValidatorEpochLen::<T>::contains_key(netuid)
            { ValidatorEpochLen::<T>::insert(netuid, ValidatorEpochLen::<T>::get(netuid));}

        if !ValidatorEpochsPerReset::<T>::contains_key(netuid)
            { ValidatorEpochsPerReset::<T>::insert(netuid, ValidatorEpochsPerReset::<T>::get(netuid));}

        if !IncentivePruningDenominator::<T>::contains_key(netuid)
            { IncentivePruningDenominator::<T>::insert(netuid, IncentivePruningDenominator::<T>::get(netuid));}

        if !StakePruningMin::<T>::contains_key(netuid)
            { StakePruningMin::<T>::insert(netuid, StakePruningMin::<T>::get(netuid));}

        if !ImmunityPeriod::<T>::contains_key(netuid)
            { ImmunityPeriod::<T>::insert(netuid, ImmunityPeriod::<T>::get(netuid));}

        if !ActivityCutoff::<T>::contains_key(netuid)
            { ActivityCutoff::<T>::insert(netuid, ActivityCutoff::<T>::get(netuid));}

        if !RegistrationsThisInterval::<T>::contains_key(netuid)
            { RegistrationsThisInterval::<T>::insert(netuid, RegistrationsThisInterval::<T>::get(netuid));}
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