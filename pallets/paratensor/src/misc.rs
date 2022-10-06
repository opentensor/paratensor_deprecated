
use super::*;
impl<T: Config> Pallet<T> {
    
    /// ==============
	/// ==== Misc ====
	/// ==============
    pub fn get_total_issuance() -> u64 { return TotalIssuance::<T>::get() }
    //pub fn get_total_stake() -> u64 { return TotalStake::<T>::get() }
    pub fn get_current_block_as_u64( ) -> u64 {
        let block_as_u64: u64 = TryInto::try_into( system::Pallet::<T>::block_number() ).ok().expect("blockchain will not exceed 2^64 blocks; QED.");
        block_as_u64
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
            Self::increment_global_n();
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
}


