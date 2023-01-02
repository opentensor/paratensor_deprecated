use super::*;
use sp_runtime::sp_std::if_std;
use substrate_fixed::types::I64F64;
use frame_support::storage::IterableStorageDoubleMap;

impl<T: Config> Pallet<T> { 

    /// ---- The implementation for the extrinsic become_delegate: signals that this hotkey allows delegated stake.
    ///
    /// # Args:
    /// 	* 'origin': (<T as frame_system::Config>Origin):
    /// 		- The signature of the caller's coldkey.
    ///
    /// 	* 'hotkey' (T::AccountId):
    /// 		- The hotkey we are delegating (must be owned by the coldkey.)
    ///
    /// 	* 'take' (u64):
    /// 		- The stake proportion that this hotkey takes from delegations.
    ///
    /// # Event:
    /// 	* DelegateAdded;
    /// 		- On successfully setting a hotkey as a delegate.
    ///
    /// # Raises:
    /// 	* 'NotRegistered':
    /// 		- The hotkey we are delegating is not registered on the network.
    ///
    /// 	* 'NonAssociatedColdKey':
    /// 		- The hotkey we are delegating is not owned by the calling coldket.
    ///
    ///
	pub fn do_become_delegate(
        origin: T::Origin, 
        hotkey: T::AccountId, 
        take: u16
    ) -> dispatch::DispatchResult {
        // --- 1. We check the coldkey signuture.
        let coldkey = ensure_signed( origin )?;
 
        // --- 2. Ensure we are delegating an known key.
        ensure!( Self::hotkey_account_exists( &hotkey ), Error::<T>::NotRegistered );    
  
        // --- 3. Ensure that the coldkey is the owner.
        ensure!( Self::coldkey_owns_hotkey( &coldkey, &hotkey ), Error::<T>::NonAssociatedColdKey );

        // --- 4. Ensure we are not already a delegate (dont allow changing delegate take.)
        ensure!( !Self::hotkey_is_delegate( &hotkey ), Error::<T>::AlreadyDelegate );

        // --- 4. Delegate the key.
        Self::delegate_hotkey( &hotkey, take );
      
        // --- 5. Emit the staking event.
        Self::deposit_event( Event::DelegateAdded( coldkey, hotkey, take ) );
   
        // --- 9. Ok and return.
        Ok(())
    }

    /// ---- The implementation for the extrinsic add_stake: Adds stake to a hotkey account.
    ///
    /// # Args:
    /// 	* 'origin': (<T as frame_system::Config>Origin):
    /// 		- The signature of the caller's coldkey.
    ///
    /// 	* 'hotkey' (T::AccountId):
    /// 		- The associated hotkey account.
    ///
    /// 	* 'stake_to_be_added' (u64):
    /// 		- The amount of stake to be added to the hotkey staking account.
    ///
    /// # Event:
    /// 	* StakeAdded;
    /// 		- On the successfully adding stake to a global account.
    ///
    /// # Raises:
    /// 	* 'CouldNotConvertToBalance':
    /// 		- Unable to convert the passed stake value to a balance.
    ///
    /// 	* 'NotEnoughBalanceToStake':
    /// 		- Not enough balance on the coldkey to add onto the global account.
    ///
    /// 	* 'NonAssociatedColdKey':
    /// 		- The calling coldkey is not associated with this hotkey.
    ///
    /// 	* 'BalanceWithdrawalError':
    /// 		- Errors stemming from transaction pallet.
    ///
    ///
	pub fn do_add_stake(
        origin: T::Origin, 
        hotkey: T::AccountId, 
        stake_to_be_added: u64
    ) -> dispatch::DispatchResult {
        // --- 1. We check that the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed( origin )?;
 
        // --- 2. We convert the stake u64 into a balancer.
        let stake_as_balance = Self::u64_to_balance( stake_to_be_added );
        ensure!( stake_as_balance.is_some(), Error::<T>::CouldNotConvertToBalance );
 
        // --- 3. Ensure the callers coldkey has enough stake to perform the transaction.
        ensure!( Self::can_remove_balance_from_coldkey_account( &coldkey, stake_as_balance.unwrap() ), Error::<T>::NotEnoughBalanceToStake );

        // --- 4. Ensure that the hotkey account exists this is only possible through registration.
        ensure!( Self::hotkey_account_exists( &hotkey ), Error::<T>::NotRegistered );    

        // --- 5. Ensure that the hotkey allows delegation or that the hotkey is owned by the calling coldkey.
        ensure!( Self::hotkey_is_delegate( &hotkey ) || Self::coldkey_owns_hotkey( &coldkey, &hotkey ), Error::<T>::NonAssociatedColdKey );
    
        // --- 6. Ensure the remove operation from the coldkey is a success.
        ensure!( Self::remove_balance_from_coldkey_account( &coldkey, stake_as_balance.unwrap() ) == true, Error::<T>::BalanceWithdrawalError );

        // --- 7. If we reach here, add the balance to the hotkey.
        Self::increase_stake_on_coldkey_hotkey_account( &coldkey, &hotkey, stake_to_be_added );
 
        // --- 8. Emit the staking event.
        Self::deposit_event( Event::StakeAdded( hotkey, stake_to_be_added ) );
 
        // --- 9. Ok and return.
        Ok(())
    }

    /// ---- The implementation for the extrinsic remove_stake: Removes stake from a hotkey account and adds it onto a coldkey.
    ///
    /// # Args:
    /// 	* 'origin': (<T as frame_system::Config>Origin):
    /// 		- The signature of the caller's coldkey.
    ///
    /// 	* 'hotkey' (T::AccountId):
    /// 		- The associated hotkey account.
    ///
    /// 	* 'stake_to_be_added' (u64):
    /// 		- The amount of stake to be added to the hotkey staking account.
    ///
    /// # Event:
    /// 	* StakeRemoved;
    /// 		- On the successfully removing stake from the hotkey account.
    ///
    /// # Raises:
    /// 	* 'NotRegistered':
    /// 		- Thrown if the account we are attempting to unstake from is non existent.
    ///
    /// 	* 'NonAssociatedColdKey':
    /// 		- Thrown if the coldkey does not own the hotkey we are unstaking from.
    ///
    /// 	* 'NotEnoughStaketoWithdraw':
    /// 		- Thrown if there is not enough stake on the hotkey to withdwraw this amount. 
    ///
    /// 	* 'CouldNotConvertToBalance':
    /// 		- Thrown if we could not convert this amount to a balance.
    ///
    ///
    pub fn do_remove_stake(
        origin: T::Origin, 
        hotkey: T::AccountId, 
        stake_to_be_removed: u64
    ) -> dispatch::DispatchResult {

        // --- 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed( origin )?;

        // --- 2. Ensure that the hotkey account exists this is only possible through registration.
        ensure!( Self::hotkey_account_exists( &hotkey ), Error::<T>::NotRegistered );    

        // --- 3. Ensure that the hotkey allows delegation or that the hotkey is owned by the calling coldkey.
        ensure!( Self::hotkey_is_delegate( &hotkey ) || Self::coldkey_owns_hotkey( &coldkey, &hotkey ), Error::<T>::NonAssociatedColdKey );

        // --- 4. Ensure that the hotkey has enough stake to withdraw.
        ensure!( Self::has_enough_stake( &coldkey, &hotkey, stake_to_be_removed ), Error::<T>::NotEnoughStaketoWithdraw );

        // --- 5. Ensure that we can conver this u64 to a balance.
        let stake_to_be_added_as_currency = Self::u64_to_balance( stake_to_be_removed );
        ensure!( stake_to_be_added_as_currency.is_some(), Error::<T>::CouldNotConvertToBalance );

        // --- 6. We remove the balance from the hotkey.
        Self::decrease_stake_on_coldkey_hotkey_account( &coldkey, &hotkey, stake_to_be_removed );

        // --- 7. We add the balancer to the coldkey.  If the above fails we will not credit this coldkey.
        Self::add_balance_to_coldkey_account( &coldkey, stake_to_be_added_as_currency.unwrap() );

        // --- 8. Emit the unstaking event.
        Self::deposit_event( Event::StakeRemoved( hotkey, stake_to_be_removed ) );

        // --- 9. Done and ok.
        Ok(())
    }

    /// Distributes token inflation through the hotkey based on emission. The call ensures that the inflation
    /// is distributed onto the accounts in proportion of the stake delegated minus the take. This function
    /// is called after an epoch to distribute the newly minted stake according to delegation.
    ///
    pub fn emit_inflation_through_hotkey_account( hotkey: &T::AccountId, emission: u64 ) {
        
        // --- 1. Check if the hotkey is a delegate. If not we simply pass the stake through to the 
        // coldkye - hotkey account as normal.
        if !Self::hotkey_is_delegate( hotkey ) { 
            Self::increase_stake_on_hotkey_account( &hotkey, emission ); 
            return; 
        }

        // --- 2. The hotkey is a delegate. We first distribute a proportion of the emission to the hotkey
        // directly as a function of its 'take'
        let total_hotkey_stake: u64 = Self::get_total_stake_for_hotkey( hotkey );
        let delegate_take: u64 = Self::calculate_delegate_proportional_take( hotkey, emission );
        let remaining_emission: u64 = emission - delegate_take;
        if_std! { println!( "emission: {:?} delegate_take: {:?} remaining_emission: {:?}\n", emission, delegate_take, remaining_emission);}

        // 3. -- The remaining emission does to the owners in proportion to the stake delegated.
        for ( owning_coldkey_i, stake_i ) in < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64 >>::iter_prefix( hotkey ) {
            
            // --- 4. The emission proportion is remaining_emission * ( stake / total_stake ).
            let stake_proportion: u64 = Self::calculate_stake_proportional_emission( stake_i, total_hotkey_stake, remaining_emission );
            if_std! { println!( "emission: {:?} stake_i: {:?}  total_hotkey_stake: {:?} stake_proportion: {:?}\n", emission, stake_i, total_hotkey_stake, stake_proportion);}
            Self::increase_stake_on_coldkey_hotkey_account( &owning_coldkey_i, &hotkey, stake_proportion );
        }
        Self::increase_stake_on_hotkey_account( &hotkey, delegate_take );

    }

    /// Returns emission awarded to a hotkey as a function of its proportion of the total stake.
    ///
    pub fn calculate_stake_proportional_emission( stake: u64, total_stake:u64, emission: u64 ) -> u64 {
        let stake_proportion: I64F64 = I64F64::from_num( stake ) / I64F64::from_num( total_stake );
        let proportional_emission: I64F64 = I64F64::from_num( emission ) * stake_proportion;
        return proportional_emission.to_num::<u64>();
    }

    /// Returns the delegated stake 'take' assigend to this key. (If exists, otherwise 0)
    ///
    pub fn calculate_delegate_proportional_take( hotkey: &T::AccountId, emission: u64 ) -> u64 {
        if Self::hotkey_is_delegate( hotkey ) {
            let take_proportion: I64F64 = I64F64::from_num( Delegates::<T>::get( hotkey ) ) / I64F64::from_num( u16::MAX );
            let take_emission: I64F64 = take_proportion * I64F64::from_num( emission );
            return take_emission.to_num::<u64>();
        } else {
            return 0;
        }
    }

    /// Returns true if the passed hotkey allow delegative staking. 
    ///
    pub fn hotkey_is_delegate( hotkey: &T::AccountId ) -> bool {
		return Delegates::<T>::contains_key( hotkey );
    }

    /// Sets the hotkey as a delegate with take.
    ///
    pub fn delegate_hotkey( hotkey: &T::AccountId, take: u16 ) {
        Delegates::<T>::insert( hotkey, take );
    }

    /// Returns the total amount of stake in the staking table.
    ///
    pub fn get_total_stake() -> u64 { 
        return TotalStake::<T>::get();
    }

    /// Increases the total amount of stake by the passed amount.
    ///
    pub fn increase_total_stake( increment: u64 ) { 
        TotalStake::<T>::put( Self::get_total_stake().saturating_add( increment ) );
    }

    /// Decreases the total amount of stake by the passed amount.
    ///
    pub fn decrease_total_stake( decrement: u64 ) { 
        TotalStake::<T>::put( Self::get_total_stake().saturating_sub( decrement ) );
    }

    /// Returns the total amount of stake under a hotkey (delegative or otherwise)
    ///
    pub fn get_total_stake_for_hotkey( hotkey: &T::AccountId ) -> u64 { 
        return TotalHotkeyStake::<T>::get( hotkey ); 
    }

    /// Returns the total amount of stake held by the coldkey (delegative or otherwise)
    ///
    pub fn get_total_stake_for_coldkey( coldkey: &T::AccountId ) -> u64 { 
        return TotalColdkeyStake::<T>::get( coldkey ); 
    }

    /// Returns the stake under the cold - hot pairing in the staking table.
    ///
    pub fn get_stake_for_coldkey_and_hotkey( coldkey: &T::AccountId, hotkey: &T::AccountId ) -> u64 { 
        return Stake::<T>::get( hotkey, coldkey );
    }

    /// Creates a cold - hot pairing account if the hotkey is not already an active account.
    ///
    pub fn create_account_if_non_existent( coldkey: &T::AccountId, hotkey: &T::AccountId ) {
        if !Self::hotkey_account_exists( hotkey ) {
            Stake::<T>::insert( hotkey, coldkey, 0 ); 
            Owner::<T>::insert( hotkey, coldkey );
        }
    }

    /// Returns the coldkey owning this hotkey. This function should only be called for active accounts.
    ///
    pub fn get_owning_coldkey_for_hotkey( hotkey: &T::AccountId ) ->  T::AccountId { 
        return Owner::<T>::get( hotkey );
    }

    /// Returns true if the hotkey account has been created.
    ///
    pub fn hotkey_account_exists( hotkey: &T::AccountId ) -> bool {
		return Owner::<T>::contains_key( hotkey );
    }

    /// Return true if the passed coldkey owns the hotkey. 
    ///
    pub fn coldkey_owns_hotkey( coldkey: &T::AccountId, hotkey: &T::AccountId ) -> bool {
        if Self::hotkey_account_exists( hotkey ){
		    return Owner::<T>::get( hotkey ) == *coldkey;
        } else {
            return false;
        }
    }

    /// Returns true if the cold-hot staking account has enough balance to fufil the decrement.
    ///
    pub fn has_enough_stake( coldkey: &T::AccountId, hotkey: &T::AccountId, decrement: u64 ) -> bool {
        return Self::get_stake_for_coldkey_and_hotkey( coldkey, hotkey ) >= decrement;
    }

    /// Increases the stake on the hotkey account under its owning coldkey.
    ///
    pub fn increase_stake_on_hotkey_account( hotkey: &T::AccountId, increment: u64 ){
        Self::increase_stake_on_coldkey_hotkey_account( &Self::get_owning_coldkey_for_hotkey( hotkey ), hotkey, increment );
    }

    /// Decreases the stake on the hotkey account under its owning coldkey.
    ///
    pub fn decrease_stake_on_hotkey_account( hotkey: &T::AccountId, decrement: u64 ){
        Self::decrease_stake_on_coldkey_hotkey_account( &Self::get_owning_coldkey_for_hotkey( hotkey ), hotkey, decrement );
    }

    /// Increases the stake on the cold - hot pairing by increment while also incrementing other counters.
    /// This function should be called rather than set_stake under account.
    /// 
    pub fn increase_stake_on_coldkey_hotkey_account( coldkey: &T::AccountId, hotkey: &T::AccountId, increment: u64 ){
        TotalColdkeyStake::<T>::insert( coldkey, Self::get_total_stake_for_coldkey( coldkey ).saturating_add( increment ) );
        TotalHotkeyStake::<T>::insert( hotkey,  Self::get_total_stake_for_hotkey( hotkey ).saturating_add( increment ) );
        let previous_stake: u64 = Self::get_stake_for_coldkey_and_hotkey( coldkey, hotkey );
        Self::__set_stake_under_coldkey_hotkey( coldkey, hotkey, previous_stake.saturating_add( increment ) ); 
        TotalStake::<T>::put( Self::get_total_stake().saturating_add( increment ) );
    }

    /// Decreases the stake on the cold - hot pairing by the decrement while decreasing other counters.
    ///
    pub fn decrease_stake_on_coldkey_hotkey_account( coldkey: &T::AccountId, hotkey: &T::AccountId, decrement: u64 ){
        TotalColdkeyStake::<T>::insert( coldkey, Self::get_total_stake_for_coldkey( coldkey ).saturating_sub( decrement ) );
        TotalHotkeyStake::<T>::insert( hotkey, Self::get_total_stake_for_hotkey( hotkey ).saturating_sub( decrement ) );
        let previous_stake: u64 = Self::get_stake_for_coldkey_and_hotkey( coldkey, hotkey );
        Self::__set_stake_under_coldkey_hotkey( coldkey, hotkey, previous_stake.saturating_sub( decrement ) ); 
        TotalStake::<T>::put( Self::get_total_stake().saturating_sub( decrement ) );
    }

    /// Private: Sets the amount of stake under the cold and hot pairinig in the staking table.
    ///
    pub fn __set_stake_under_coldkey_hotkey( coldkey: &T::AccountId, hotkey: &T::AccountId, stake:u64 ) { 
        Stake::<T>::insert( hotkey, coldkey, stake );
    }




	pub fn u64_to_balance( input: u64 ) -> Option<<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance> { input.try_into().ok() }

    pub fn add_balance_to_coldkey_account(coldkey: &T::AccountId, amount: <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance) {
        T::Currency::deposit_creating(&coldkey, amount); // Infallibe
    }

    pub fn can_remove_balance_from_coldkey_account(coldkey: &T::AccountId, amount: <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance) -> bool {
        let current_balance = Self::get_coldkey_balance(coldkey);
        if amount > current_balance {
            return false;
        }

        // This bit is currently untested. @todo
        let new_potential_balance = current_balance - amount;
        let can_withdraw = T::Currency::ensure_can_withdraw(&coldkey, amount, WithdrawReasons::except(WithdrawReasons::TIP), new_potential_balance).is_ok();
        can_withdraw
    }

    pub fn get_coldkey_balance(coldkey: &T::AccountId) -> <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance {
        return T::Currency::free_balance(&coldkey);
    }

    pub fn remove_balance_from_coldkey_account(coldkey: &T::AccountId, amount: <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance) -> bool {
        return match T::Currency::withdraw(&coldkey, amount, WithdrawReasons::except(WithdrawReasons::TIP), ExistenceRequirement::KeepAlive) {
            Ok(_result) => {
                true
            }
            Err(_error) => {
                false
            }
        };
    }

}