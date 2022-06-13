use super::*;

impl<T: Config> Pallet<T> {
    /***********************************************************
    * add_account()
    ***********************************************************/
    pub fn add_account( hotkey: &T::AccountId, coldkey: &T::AccountId )  {
        if !Hotkeys::<T>::contains_key( &hotkey ) { 
            Hotkeys::<T>::insert( hotkey.clone(), coldkey.clone() );
            Coldkeys::<T>::insert( coldkey.clone(), hotkey.clone() );
            Stake::<T>::insert( hotkey.clone(), 0 );
            Self::increment_n();
        }
    }

    pub fn get_n() -> u64 { 
        return N::<T>::get(); 
    }
    pub fn increment_n() {
        let n = N::<T>::get();
        if n < u64::MAX {
            N::<T>::put(n + 1);
        }
    }
    pub fn decrement_n() {
        let n = N::<T>::get();
        if n > 0 {
            N::<T>::put(n - 1);
        }
    }

}

