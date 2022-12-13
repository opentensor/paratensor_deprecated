use super::*;
use frame_support::sp_std::vec;
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    pub fn do_set_weights(origin: T::Origin, netuid: u16, uids: Vec<u16>, values: Vec<u16>) -> dispatch::DispatchResult{
        /* TO DO:
        1. Check the caller signature
        2. Check if network exists
        3. Check to see that the calling neuron is in the active set.
        4. Check that the length of uid list and value list are equal for this network.
        5. Check if the uids vector does not contain duplicate ids.
        6. Check if the weight uids are valid.
        7. Check if the weights have the desired length.
        8. Normalize weights.
        9. Check if the weights do not exceed the max weight limit.
        10. Zip weights.
        11. Update weights
        12. Emit the staking event. */
        // TODO( Saeideh ): Dont need this.

        // 1. Check the caller signature
        let hotkey_id = ensure_signed(origin)?;

        // 2. check if network exist 
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);

        // 3. Check to see that the calling neuron is in the active set.
        ensure!(Self::is_hotkey_registered(netuid, &hotkey_id), Error::<T>::NotRegistered);
        //
        let neuron_uid ;
        match Self::get_neuron_for_net_and_hotkey(netuid, &hotkey_id) {
            Ok(k) => neuron_uid = k,
            Err(e) => panic!("Error: {:?}", e),
        } 

        // 4. Check that the length of uid list and value list are equal for this network.
        ensure!(Self::uids_match_values(&uids, &values), Error::<T>::WeightVecNotEqualSize);

        // 5. Check if the uids vector does not contain duplicate ids.
        ensure!(!Self::has_duplicate_uids(&uids), Error::<T>::DuplicateUids);

        // 6. Check if the weight uids are valid.
        ensure!(!Self::contains_invalid_uids(netuid, &uids), Error::<T>::InvalidUid);

        // 7. Check if the weights have the desired length.
        ensure!( Self::check_length(netuid, neuron_uid, &uids, &values), Error::<T>::NotSettingEnoughWeights);

        // 8. Normalize weights.
        let normalized_values = Self::normalize_weights(values);

        // 9. Check if the weights do not exceed the max weight limit.
        ensure!( Self::max_weight_limited(netuid, neuron_uid, &uids, &normalized_values), Error::<T>::MaxWeightExceeded );

        // 10. Zip weights.
        let mut zipped_weights: Vec<(u16,u16)> = vec![];
        for (uid, val) in uids.iter().zip(normalized_values.iter()) {
            zipped_weights.push((*uid, *val))
        }
        Weights::<T>::insert(netuid, neuron_uid, zipped_weights);
        Keys::<T>::insert(netuid, neuron_uid, hotkey_id); //set neuron active
        LastUpdate::<T>::insert(netuid, neuron_uid, Self::get_current_block_as_u64());

         // ---- Emit the staking event.
         Self::deposit_event(Event::WeightsSet(netuid, neuron_uid));

         // --- Emit the event and return ok.
         Ok(())

    }

    /********************************
    --==[[  Helper functions   ]]==--
   *********************************/

   //check if uid exist in this network
   pub fn contains_invalid_uids(netuid: u16, uids: &Vec<u16>) -> bool {
    for uid in uids {
        if !Self::is_uid_exist(netuid, *uid) {
            return true;
        }
    }
    return false;
}
    fn uids_match_values(uids: &Vec<u16>, values: &Vec<u16>) -> bool {
        return uids.len() == values.len();
    }

    fn has_duplicate_uids(items: &Vec<u16>) -> bool {
        let mut parsed: Vec<u16> = Vec::new();
        for item in items {
            if parsed.contains(&item) { return true; }
            parsed.push(item.clone());
        }
    
        return false;
    }

    // Check if weights have fewer values than are allowed.
    pub fn check_length( netuid: u16, uid: u16, uids: &Vec<u16>, weights: &Vec<u16> ) -> bool {
        let min_allowed_length: usize = Self::get_min_allowed_weights(netuid) as usize;

        // Check self weight. Allowed to set single value for self weight.
        if Self::is_self_weight(uid, uids, weights) {
            return true;
        }
        // Check if number of weights exceeds min.
        if weights.len() >= min_allowed_length {
            return true;
        }
        // To few weights.
        return false;
    }

       // Returns true if the peer is setting a single self weight.
       pub fn is_self_weight( uid: u16, uids: &Vec<u16>, weights: &Vec<u16> ) -> bool {
        if weights.len() != 1 {
            return false;
        }
        if uid != uids[0] {
            return false;
        } 
        return true;
    }

    fn normalize_weights(mut weights: Vec<u16>) -> Vec<u16> {
        let sum: u64 = weights.iter().map(|x| *x as u64).sum();
        if sum == 0 {
            return weights;
        }
        weights.iter_mut().for_each(|x| {
            *x = (*x as u64 * u16::max_value() as u64 / sum) as u16;
        });
        return weights;
    }

     // Checks if none of the normalized weight magnitudes exceed the max weight limit.
     pub fn max_weight_limited( netuid: u16, uid: u16, uids: &Vec<u16>, weights: &Vec<u16> ) -> bool {

        // Allow self weights to exceed max weight limit.
        if Self::is_self_weight(uid, uids, weights) {
            return true;
        }

        let max_weight_limit: u16 = Self::get_max_weight_limit(netuid);
        if max_weight_limit == u16::MAX {
            return true;
        }
    
        let max: u16 = *weights.iter().max().unwrap();
        if max <= max_weight_limit { 
            return true;
        }
        return false;
    }
    
}