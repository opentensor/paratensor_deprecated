use super::*;
use frame_support::{sp_std::vec};
use sp_std::vec::Vec;
use crate::system::ensure_root;
use frame_support::storage::IterableStorageMap;
use frame_support::pallet_prelude::DispatchError;
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
        ensure!( !Self::if_subnet_exist( netuid ), Error::<T>::NetworkExist );

        // --- 3. Ensure the modality is valid.
        ensure!( Self::if_modality_is_valid( modality ), Error::<T>::InvalidModality );

        // --- 4. Ensure the tempo is valid.
        ensure!( Self::if_tempo_is_valid( tempo ), Error::<T>::InvalidTempo );

        // --- 5. Initialize the network and all its parameters.
        Self::init_new_network( netuid, tempo, modality );
        
        // --- 6. Emit the new network event.
        log::info!("NetworkAdded( netuid:{:?}, modality:{:?} )", netuid, modality);
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
        ensure!( Self::if_subnet_exist( netuid ), Error::<T>::NetworkDoesNotExist );

        // --- 3. Explicitly erase the network and all its parameters.
        Self::remove_network( netuid );
    
        // --- 4. Emit the event.
        log::info!("NetworkRemoved( netuid:{:?} )", netuid);
        Self::deposit_event( Event::NetworkRemoved( netuid ) );

        // --- 5. Ok and return.
        Ok(())
    }

    /// ---- The implementation for the extrinsic sudo_add_network_connect_requirement.
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
    /// 	* `prunning_score_requirement` (u16):
    /// 		- The topk percentile prunning score requirement (u16:MAX normalized.)
    ///
    pub fn do_sudo_add_network_connection_requirement(
        origin: T::Origin, 
        netuid_a: u16,
        netuid_b: u16,
        requirement: u16
    ) -> dispatch::DispatchResult {
        ensure_root( origin )?;
        ensure!( netuid_a != netuid_b, Error::<T>::InvalidConnectionRequirement );
        ensure!( Self::if_subnet_exist( netuid_a ), Error::<T>::NetworkDoesNotExist );
        ensure!( Self::if_subnet_exist( netuid_b ), Error::<T>::NetworkDoesNotExist );
        Self::add_connection_requirement( netuid_a, netuid_b, requirement );
        log::info!("NetworkConnectionAdded( netuid_a:{:?}, netuid_b:{:?} requirement: {:?} )", netuid_a, netuid_b, requirement);
        Self::deposit_event( Event::NetworkConnectionAdded( netuid_a, netuid_b, requirement ) );
        Ok(())
    }

    /// ---- The implementation for the extrinsic sudo_remove_network_connect_requirement.
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
    pub fn do_sudo_remove_network_connection_requirement(
        origin: T::Origin, 
        netuid_a: u16,
        netuid_b: u16,
    ) -> dispatch::DispatchResult {
        ensure_root( origin )?;
        ensure!( Self::if_subnet_exist( netuid_a ), Error::<T>::NetworkDoesNotExist );
        ensure!( Self::if_subnet_exist( netuid_b ), Error::<T>::NetworkDoesNotExist );
        Self::remove_connection_requirment( netuid_a, netuid_b );
        log::info!("NetworkConnectionRemoved( netuid_a:{:?}, netuid_b:{:?} )", netuid_a, netuid_b );
        Self::deposit_event( Event::NetworkConnectionRemoved( netuid_a, netuid_b ) );
        Ok(())
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
        ensure!( emission.iter().sum::<u64>() as u64 == Self::get_block_emission(), Error::<T>::InvalidEmissionValues);

        // --- 7. Add emission values for each network
        Self::set_emission_values( &netuids, &emission );

        // --- 8. Add emission values for each network
        log::info!("EmissionValuesSet()");
        Self::deposit_event( Event::EmissionValuesSet() );

        // --- 9. Ok and return.
        Ok(())
    }

    /// Initializes a new subnetwork under netuid with parameters.
    ///
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
    ///
    pub fn remove_network( netuid:u16 ) {

        // --- 1. Remove network count.
        SubnetworkN::<T>::remove( netuid );

        // --- 2. Remove network modality storage.
        NetworkModality::<T>::remove( netuid );

        // --- 3. Remove netuid from added networks.
        NetworksAdded::<T>::remove( netuid );

        // --- 4. Erase all memory associated with the network.
        Self::erase_all_network_data( netuid );

        // --- 5. Decrement the network counter.
        TotalNetworks::<T>::mutate(|val| *val -= 1);
    }


    /// Explicitly sets all network parameters to their default values.
    /// Note: this is required because, although there are defaults, they are not explicitly set until this call.
    ///
    pub fn set_default_values_for_all_parameters(netuid: u16){
        // Make network parameters explicit.
        if !Tempo::<T>::contains_key( netuid ) { Tempo::<T>::insert( netuid, Tempo::<T>::get( netuid ));}
        if !Kappa::<T>::contains_key( netuid ) { Kappa::<T>::insert( netuid, Kappa::<T>::get( netuid ));}
        if !Difficulty::<T>::contains_key( netuid ) { Difficulty::<T>::insert( netuid, Difficulty::<T>::get( netuid ));}
        if !MaxAllowedUids::<T>::contains_key( netuid ) { MaxAllowedUids::<T>::insert( netuid, MaxAllowedUids::<T>::get( netuid ));}
        if !ImmunityPeriod::<T>::contains_key( netuid ) { ImmunityPeriod::<T>::insert( netuid, ImmunityPeriod::<T>::get( netuid ));}
        if !ActivityCutoff::<T>::contains_key( netuid ) { ActivityCutoff::<T>::insert( netuid, ActivityCutoff::<T>::get( netuid ));}
        if !EmissionValues::<T>::contains_key( netuid ) { EmissionValues::<T>::insert( netuid, EmissionValues::<T>::get( netuid ));}   
        if !MaxWeightsLimit::<T>::contains_key( netuid ) { MaxWeightsLimit::<T>::insert( netuid, MaxWeightsLimit::<T>::get( netuid ));}
        if !ValidatorEpochLen::<T>::contains_key( netuid ) { ValidatorEpochLen::<T>::insert( netuid, ValidatorEpochLen::<T>::get( netuid ));}
        if !MinAllowedWeights::<T>::contains_key( netuid ) { MinAllowedWeights::<T>::insert( netuid, MinAllowedWeights::<T>::get( netuid )); }
        if !ValidatorBatchSize::<T>::contains_key( netuid ) { ValidatorBatchSize::<T>::insert( netuid, ValidatorBatchSize::<T>::get( netuid ));}
        if !ValidatorEpochsPerReset::<T>::contains_key( netuid ) { ValidatorEpochsPerReset::<T>::insert( netuid, ValidatorEpochsPerReset::<T>::get( netuid ));}
        if !ValidatorSequenceLength::<T>::contains_key( netuid ) { ValidatorSequenceLength::<T>::insert( netuid, ValidatorSequenceLength::<T>::get( netuid ));}
        if !RegistrationsThisInterval::<T>::contains_key( netuid ) { RegistrationsThisInterval::<T>::insert( netuid, RegistrationsThisInterval::<T>::get( netuid ));}
    }

    /// Explicitly erases all data associated with this network.
    ///
    pub fn erase_all_network_data(netuid: u16){

        // --- 1. Remove incentive mechanism memory.
        Uids::<T>::remove_prefix( netuid, None );
        Keys::<T>::remove_prefix( netuid, None );
        Rank::<T>::remove_prefix( netuid, None );
        Trust::<T>::remove_prefix( netuid, None );
        ValidatorTrust::<T>::remove_prefix( netuid, None );
        Bonds::<T>::remove_prefix( netuid, None );
        Active::<T>::remove_prefix( netuid, None );
        Weights::<T>::remove_prefix( netuid, None );
        Emission::<T>::remove_prefix( netuid, None );
        Incentive::<T>::remove_prefix( netuid, None );
        Consensus::<T>::remove_prefix( netuid, None );
        WeightConsensus::<T>::remove_prefix( netuid, None );
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
        MaxWeightsLimit::<T>::remove( netuid );
        ValidatorEpochLen::<T>::remove( netuid );
        MinAllowedWeights::<T>::remove( netuid );
        ValidatorBatchSize::<T>::remove( netuid );
        ValidatorEpochsPerReset::<T>::remove( netuid );
        ValidatorSequenceLength::<T>::remove( netuid );
        RegistrationsThisInterval::<T>::remove( netuid );
    }


    /// Returns the number of filled slots on a network.
    ////
    pub fn get_subnetwork_n( netuid:u16 ) -> u16 { 
        return SubnetworkN::<T>::get( netuid ) 
    }

    /// Increments the number of slots used on a network.
    ///
    pub fn increment_subnetwork_n( netuid:u16 ) {
        SubnetworkN::<T>::insert( netuid, SubnetworkN::<T>::take( netuid ) + 1 );
    }

    /// Decrements the number of used slots on a network.
    ///
    pub fn decrement_subnetwork_n( netuid:u16 ) { 
        let n = SubnetworkN::<T>::get( netuid ); 
        if n > 0 {
             SubnetworkN::<T>::insert(netuid, n - 1); 
        } 
    }

    /// Returns true if the uid is set on the network.
    ///
    pub fn is_uid_exist_on_network(netuid: u16, uid: u16) -> bool {
        return  Keys::<T>::contains_key(netuid, uid);
    }

    /// Returns true if the hotkey holds a slot on the network.
    ///
    pub fn is_hotkey_registered_on_network( netuid:u16, hotkey: &T::AccountId ) -> bool { 
        return Uids::<T>::contains_key( netuid, hotkey ) 
    }

    /// Returs the hotkey under the network uid as a Result. Ok if the uid is taken.
    ///
    pub fn get_hotkey_for_net_and_uid( netuid: u16, neuron_uid: u16) ->  Result<T::AccountId, DispatchError> {
        Keys::<T>::try_get(netuid, neuron_uid).map_err(|_err| Error::<T>::NotRegistered.into()) 
    }

    /// Returns the uid of the hotkey in the network as a Result. Ok if the hotkey has a slot.
    ///
    pub fn get_uid_for_net_and_hotkey( netuid: u16, hotkey: &T::AccountId) -> Result<u16, DispatchError> { 
        return Uids::<T>::try_get(netuid, &hotkey).map_err(|_err| Error::<T>::NotRegistered.into()) 
    }

    /// Returns the stake of the uid on network or 0 if it doesnt exist.
    ///
    pub fn get_stake_for_uid_and_subnetwork( netuid: u16, neuron_uid: u16) -> u64 { 
        if Self::is_uid_exist_on_network( netuid, neuron_uid) {
            return Self::get_total_stake_for_hotkey( &Self::get_hotkey_for_net_and_uid( netuid, neuron_uid ).unwrap() ) 
        } else {
            return 0;
        }
    }

    /// Fills a uid on the network.
    ///
    pub fn add_subnetwork_account( netuid:u16, uid: u16, hotkey: &T::AccountId ) { 
        Keys::<T>::insert( netuid, uid, hotkey.clone() ); 
        Uids::<T>::insert( netuid, hotkey.clone(), uid );
        Self::increment_subnetwork_n( netuid );
    }

    /// Removes a uid from the subnetwork.
    ///
    pub fn remove_subnetwork_account( netuid:u16, uid: u16 ) { 
        let hotkey = Keys::<T>::get( netuid, uid );
        Uids::<T>::remove( netuid, hotkey.clone() );
        Keys::<T>::remove( netuid, uid ); 
        Self::decrement_subnetwork_n( netuid );
    }

    /// Return the total number of subnetworks available on the chain.
    ///
    pub fn get_number_of_subnets()-> u16 {
        let mut number_of_subnets : u16 = 0;
        for (_, _)  in <SubnetworkN<T> as IterableStorageMap<u16, u16>>::iter(){
            number_of_subnets = number_of_subnets + 1;
        }
        return number_of_subnets;
    }

    /// Return a list of all networks a hotkey is registered on.
    ///
    pub fn get_registered_networks_for_hotkey( hotkey: &T::AccountId )-> Vec<u16> {
        let mut all_networks: Vec<u16> = vec![];
        for ( network, is_registered)  in <IsNetworkMember<T> as IterableStorageDoubleMap< T::AccountId, u16, bool >>::iter_prefix( hotkey ){
            if is_registered { all_networks.push( network ) }
        }
        all_networks
    }

    /// Return true if a hotkey is registered on any network.
    ///
    pub fn is_hotkey_registered_on_any_network( hotkey: &T::AccountId )-> bool {
        for ( _, is_registered)  in <IsNetworkMember<T> as IterableStorageDoubleMap< T::AccountId, u16, bool >>::iter_prefix( hotkey ){
            if is_registered { return true }
        }
        false
    }


    /// --- Returns true if a network connection exists.
    ///
    pub fn network_connection_requirement_exists( netuid_a: u16, netuid_b: u16 ) -> bool {
        NetworkConnect::<T>::contains_key( netuid_a, netuid_b )
    }

    /// --- Returns the network connection requirment between net A and net B.
    ///
    pub fn get_network_connection_requirement( netuid_a: u16, netuid_b: u16 ) -> u16 {
        if Self::network_connection_requirement_exists( netuid_a, netuid_b ){
            return NetworkConnect::<T>::get( netuid_a, netuid_b ).unwrap();
        } else {
            // Should never occur.
            return u16::MAX;
        }
    }

    /// --- Adds a network b connection requirement to network a. 
    ///
    pub fn add_connection_requirement( netuid_a: u16, netuid_b: u16, requirement: u16 ) {
        NetworkConnect::<T>::insert( netuid_a, netuid_b, requirement );
    }

    /// --- Removes the network b connection requirement from network a. 
    ///
    pub fn remove_connection_requirment( netuid_a: u16, netuid_b: u16) {
        if Self::network_connection_requirement_exists(netuid_a, netuid_b) { NetworkConnect::<T>::remove( netuid_a, netuid_b ); }
    }

    /// Returns true if the items contain duplicates.
    ///
    fn has_duplicate_netuids( netuids: &Vec<u16> ) -> bool {
        let mut parsed: Vec<u16> = Vec::new();
        for item in netuids {
            if parsed.contains(&item) { return true; }
            parsed.push(item.clone());
        }
        return false;
    }

    /// Checks for any invalid netuids on this network.
    ///
    pub fn contains_invalid_netuids( netuids: &Vec<u16> ) -> bool {
        for netuid in netuids {
            if !Self::if_subnet_exist( *netuid ) {
                return true;
            }
        }
        return false;
    }

    /// Set emission values for the passed networks. 
    ///
    pub fn set_emission_values( netuids: &Vec<u16>, emission: &Vec<u64> ){
        for (i, netuid_i) in netuids.iter().enumerate() {
            Self::set_emission_for_network( *netuid_i, emission[i] ); 
        }
    }

    /// Set the emission on a single network.
    ///
    pub fn set_emission_for_network( netuid: u16, emission: u64 ){
        EmissionValues::<T>::insert( netuid, emission );
    }

    /// Returns true if the subnetwork exists.
    ///
    pub fn if_subnet_exist( netuid: u16 ) -> bool{
        return NetworksAdded::<T>::get( netuid );
    }

    /// Returns true if the passed modality is allowed.
    ///
    pub fn if_modality_is_valid( modality: u16 ) -> bool{
        let allowed_values: Vec<u16> = vec![0, 1, 2];
        return allowed_values.contains( &modality );
    } 

    /// Returns true if the passed tempo is allowed.
    ///
    pub fn if_tempo_is_valid(tempo: u16) -> bool {
        tempo < u16::MAX
    }
}