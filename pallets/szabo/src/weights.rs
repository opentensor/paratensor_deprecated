use super::*;
use frame_support::inherent::Vec;
use frame_support::sp_std::vec;

impl<T: Config> Pallet<T> {
    pub fn do_set_weights(origin: T::Origin, netuid: u16, uids: Vec<u16>, values: Vec<u16>) -> dispatch::DispatchResult
    {
        // ---- We check the caller signature
        let hotkey_id = ensure_signed( origin )?;

        // ---- We check to see that the calling hotkey account is in the active set.
        ensure!( Self::is_hotkey_subnetwork_active( netuid, &hotkey_id ), Error::<T>::NotRegistered );
        let uid = Self::get_subnetwork_uid( netuid, &hotkey_id );

        // --- We check that the length of these two lists are equal.
        ensure!( uids_match_values( &uids, &values ), Error::<T>::WeightVecNotEqualSize );

        // --- We check if the uids vector does not contain duplicate ids
        ensure!( !has_duplicate_uids( &uids ), Error::<T>::DuplicateUids );

        // --- We check if the weight uids are valid
        ensure!( !Self::contains_invalid_uids( netuid, &uids ), Error::<T>::InvalidUid );

        // --- We check if the weights have the desired length.
        ensure!( Self::check_length( netuid, uid, &uids, &values ), Error::<T>::NotSettingEnoughWeights );

        // Normalize weights.
        let normalized_values = normalize(values);

        // --- We check if the weights have an allowed max min multiple.
        ensure!( Self::min_is_allowed_multiple_of_max( netuid, &normalized_values ), Error::<T>::MaxAllowedMaxMinRatioExceeded );

        // Zip weights.
        let mut zipped_weights: Vec<(u16,u16)> = vec![];
        for (uid, val) in uids.iter().zip(normalized_values.iter()) {
            zipped_weights.push((*uid, *val))
        }

        // Sink update.
        Weights::<T>::insert( netuid, uid, zipped_weights );

        // ---- Emit the staking event.
        Self::deposit_event(Event::WeightsSet(netuid, uid));

        // --- Emit the event and return ok.
        Ok(())
    }

    /********************************
    --==[[  Helper functions   ]]==--
   *********************************/
    
    pub fn contains_invalid_uids(netuid:u16, uids: &Vec<u16>) -> bool {
        for uid in uids {
            if !Self::is_subnetwork_uid_active(netuid, *uid) {
                return true;
            }
        }
        return false;
    }

    pub fn check_length( netuid: u16, uid: u16, uids: &Vec<u16>, weights: &Vec<u16>) -> bool {
        let min_allowed_length: usize = Self::get_min_allowed_weights( netuid ) as usize;

        // Check the self weight.
        if weights.len() == 1 {
            if uid == uids[0] {
                // Allows the self weight.
                return true;
            } else {
                // Always fails when setting just a single weight.
                return false;
            }

        // Otherwise we check to ensure we passed the weigh limit.
        } else if weights.len() >= min_allowed_length {
            return true
        } else {
            return false
        }
    }

    pub fn min_is_allowed_multiple_of_max( netuid:u16, weights: &Vec<u16>) -> bool {
        // We allow the 0 value multiple to be cardinal -> We always return true.
        let max_allowed_max_min_ratio: u16 = Self::get_max_allowed_max_min_ratio( netuid ) as u16;
        if max_allowed_max_min_ratio == 0 {
            return true;
        }
    
        let min: u16 = *weights.iter().min().unwrap();
        let max: u16 = *weights.iter().max().unwrap();
        if min == 0 { 
            return false
        } else {
            // Check that the min is a allowed multiple of the max.
            if max / min > max_allowed_max_min_ratio {
                return false;
            } else {
                return true;
            }
        }
    }
}

fn uids_match_values(uids: &Vec<u16>, values: &Vec<u16>) -> bool {
    return uids.len() == values.len();
}

/**
* This function tests if the uids half of the weight matrix contains duplicate uid's.
* If it does, an attacker could
*/
fn has_duplicate_uids(items: &Vec<u16>) -> bool {
    let mut parsed: Vec<u16> = Vec::new();
    for item in items {
        if parsed.contains(&item) { return true; }
        parsed.push(item.clone());
    }

    return false;
}


fn normalize(mut weights: Vec<u16>) -> Vec<u16> {
    let sum: u64 = weights.iter().map(|x| *x as u64).sum();
    if sum == 0 {
        return weights;
    }
    weights.iter_mut().for_each(|x| {
        *x = (*x as u64 * u16::max_value() as u64 / sum) as u16;
    });
    return weights;
}


#[cfg(test)]
mod tests {
    use crate::weights::{normalize, has_duplicate_uids};

    #[test]
    fn normalize_sum_smaller_than_one() {
        let values: Vec<u16> = vec![u16::max_value() / 10, u16::max_value() / 10];
        assert_eq!(normalize(values), vec![u16::max_value() / 2, u16::max_value() / 2]);
    }

    #[test]
    fn normalize_sum_greater_than_one() {
        let values: Vec<u16> = vec![u16::max_value() / 7, u16::max_value() / 7];
        assert_eq!(normalize(values), vec![u16::max_value() / 2, u16::max_value() / 2]);
    }

    #[test]
    fn normalize_sum_zero() {
        let weights: Vec<u16> = vec![0, 0];
        assert_eq!(normalize(weights), vec![0, 0]);
    }

    #[test]
    fn normalize_values_maxed() {
        let weights: Vec<u16> = vec![u16::max_value(), u16::max_value()];
        assert_eq!(normalize(weights), vec![u16::max_value() / 2, u16::max_value() / 2]);
    }

    #[test]
    fn has_duplicate_elements_true() {
        let weights = vec![1, 2, 3, 4, 4, 4, 4];
        assert_eq!(has_duplicate_uids(&weights), true);
    }

    #[test]
    fn has_duplicate_elements_false() {
        let weights = vec![1, 2, 3, 4, 5];
        assert_eq!(has_duplicate_uids(&weights), false);
    }
}
