
use super::*;
use sp_core::U256;
use frame_support::inherent::Vec;
use frame_support::sp_std::vec;
use frame_support::storage::IterableStorageDoubleMap;
use sp_runtime::sp_std::if_std;

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

    pub fn get_difficulty(netuid: u16 ) -> U256 {
        return U256::from( Self::get_difficulty_as_u64(netuid) );
    }

    pub fn get_difficulty_as_u64(netuid: u16 ) -> u64 {
        Difficulty::<T>::get(netuid)
    }

    pub fn get_max_allowed_uids(netuid: u16 ) -> u16 {
        return MaxAllowedUids::<T>::get(netuid);
    }

    // --- Returns the next available network uid.
    // uids increment up to u64:MAX, this allows the chain to
    // have 18,446,744,073,709,551,615 peers before an overflow.
    pub fn get_neuron_count(netuid: u16) -> u16 {
        let uid_count = SubnetworkN::<T>::get(netuid);
        uid_count
    }
    // --- Returns the next available network uid and increments uid.
		pub fn get_next_uid() -> u16 {
			let uid = GlobalN::<T>::get();
			assert!(uid < u16::MAX);  // The system should fail if this is ever reached.
			GlobalN::<T>::put(uid + 1); if_std! { println!( "uid in next_uid func: {}", uid);};
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
        pub fn get_neuron_to_prune(netuid: u16) -> u16 {
            let mut min_score : u16 = u16::MAX;
            let mut uid_with_min_score = 0;
            for (uid_i, _prune_score) in <PrunningScores<T> as IterableStorageDoubleMap<u16, u16, u16 >>::iter_prefix( netuid ) {
                let value = PrunningScores::<T>::get(netuid, uid_i);
                if min_score > value { 
                    min_score = value; 
                    uid_with_min_score = uid_i;
                }
            }
            uid_with_min_score
        } 
        pub fn get_neuron_stake_for_subnetwork(netuid: u16, neuron_uid: u16) -> u64 {
            S::<T>::get(netuid, neuron_uid)
        }

    /// =========================
	/// ==== Global Accounts ====
	/// =========================
    pub fn get_global_n() -> u16 { return GlobalN::<T>::get() }
    pub fn is_hotkey_globally_active( hotkey: &T::AccountId ) -> bool { return Coldkeys::<T>::contains_key( hotkey ) }
    pub fn increment_global_n() { let n = GlobalN::<T>::get(); if n < u16::MAX { GlobalN::<T>::put(n + 1); } }
    pub fn decrement_global_n() { let n = GlobalN::<T>::get(); if n > 0 { GlobalN::<T>::put(n - 1); } }
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
            Self::decrement_global_n();
        }
    }


    /// ==============================
	/// ==== Subnetworks Accounts ====
	/// ==============================
    pub fn is_hotkey_subnetwork_active( netuid:u16, hotkey: &T::AccountId ) -> bool { return Uids::<T>::contains_key( netuid, hotkey ) }
    pub fn is_subnetwork_uid_active( netuid:u16, uid: u16 ) -> bool { return uid < SubnetworkN::<T>::get( netuid ) }
    pub fn get_subnetwork_uid( netuid:u16, hotkey: &T::AccountId ) -> u16 { return Uids::<T>::get( netuid, hotkey ) }
    pub fn get_subnetwork_n( netuid:u16 ) -> u16 { return SubnetworkN::<T>::get( netuid ) }
    pub fn increment_subnetwork_n( netuid:u16 ) { let n = SubnetworkN::<T>::get( netuid ); if n < u16::MAX { SubnetworkN::<T>::insert(netuid, n + 1); } }
    pub fn decrement_subnetwork_n( netuid:u16 ) { let n = SubnetworkN::<T>::get( netuid ); if n > 0 { SubnetworkN::<T>::insert(netuid, n - 1); } }
    pub fn add_subnetwork_account( netuid:u16, uid: u16, hotkey: &T::AccountId ) { 
        Keys::<T>::insert( netuid, uid, hotkey.clone() ); 
        Uids::<T>::insert( netuid, hotkey.clone(), uid );
        Self::increment_subnetwork_n( netuid );
    }
    pub fn remove_subnetwork_account( netuid:u16, uid: u16 ) { 
        let hotkey = Keys::<T>::get( netuid, uid );
        Uids::<T>::remove( netuid, hotkey.clone() );
        Keys::<T>::remove( netuid, uid ); 
        Self::decrement_subnetwork_n( netuid );
    }
    pub fn get_coldkey_for_hotkey(hotkey:  T::AccountId) ->  T::AccountId {
        return Hotkeys::<T>::get(hotkey);
    }

    pub fn get_hotkey_for_coldkey(coldkey: T::AccountId) -> T::AccountId {
        return Coldkeys::<T>::get(coldkey);
    }

    pub fn get_subnets_for_hotkey(hotkey: T::AccountId) -> Vec<u16> {
        let subnets: Vec<u16> = Subnets::<T>::get(hotkey);
        subnets
    }

    pub fn get_hotkey_for_net_and_neuron(netuid: u16, neuron_uid: u16) -> T::AccountId {
        return Keys::<T>::get(netuid, neuron_uid);
    }

    pub fn get_neuron_for_net_and_hotkey(netuid: u16, hotkey: T::AccountId) -> u16 {
        return Uids::<T>::get(netuid, hotkey);
    }
    pub fn increment_subnets_for_hotkey(netuid: u16, hotkey: &T::AccountId){
        let mut vec_new_hotkey_subnets = vec![];
        if Subnets::<T>::contains_key(&hotkey){ //update the list of subnets that hotkey is registered on
            vec_new_hotkey_subnets = Subnets::<T>::take(&hotkey);
            //Subnets::<T>::remove(&hotkey);
            vec_new_hotkey_subnets.push(netuid);
            Subnets::<T>::insert(&hotkey, vec_new_hotkey_subnets); 
        } else {
            vec_new_hotkey_subnets.push(netuid); 
            Subnets::<T>::insert(&hotkey, vec_new_hotkey_subnets); 
        }
    }
}


