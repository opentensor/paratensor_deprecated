use super::*;

impl<T: Config> Pallet<T> {
    /***********************************************************
     * do_add_stake() - main function called from parent module
     ***********************************************************/
    pub fn do_add_stake(origin: T::Origin, hotkey: T::AccountId, stake_to_be_added: u64) -> dispatch::DispatchResult
    {
        // ---- We check the transaction is signed by the caller
        // and retrieve the T::AccountId pubkey information.
        let coldkey = ensure_signed( origin )?;

        // --- Check to see if the hotkey is active
        ensure!( Hotkeys::<T>::contains_key( &hotkey ), Error::<T>::NotRegistered );

        // ---- Check that the coldkey owns the hotkey or throw a NonAssociatedColdKey error.
        ensure!( Coldkeys::<T>::get( &hotkey ) == coldkey, Error::<T>::NonAssociatedColdKey);

        // ---- We check that the balance can be removed from the coldkey account.
        ensure!(Self::can_remove_balance_from_coldkey_account( &coldkey, stake_to_be_added ), Error::<T>::NotEnoughBalanceToStake);

        // ---- We check that the withdrawl does not throw and error.
        ensure!(Self::remove_balance_from_coldkey_account( &coldkey, stake_to_be_added ) == true, Error::<T>::BalanceWithdrawalError);

        // ---- We add the stake to the hotkey account.
        Self::add_stake_to_hotkey_account( &hotkey, stake_to_be_added );

        // ---- Emit the staking event.
        Self::deposit_event(Event::StakeAdded( hotkey, stake_to_be_added ));

        // --- ok and return.
        Ok(())
    }

    /// This function removes stake from a hotkey account and puts into a coldkey account.
    /// This function should be called through an extrinsic signed with the coldkeypair's private
    /// key. It takes a hotkey account id and an ammount as parameters.
    ///
    /// Generally, this function works as follows
    /// 1) A Check is performed to see if the hotkey is active (ie, the node using the key is subscribed)
    /// 2) The neuron metadata associated with the hotkey is retrieved, and is checked if it is subscribed with the supplied cold key
    /// 3) If these checks pass, inflation is emitted to the nodes' peers
    /// 4) If the account has enough stake, the requested amount it transferred to the coldkey account
    /// 5) The total amount of stake is reduced after transfer is complete
    ///
    /// It throws the following errors if there is something wrong
    /// - NotRegistered : The suplied hotkey is not in use. This ususally means a node that uses this key has not subscribed yet, or has unsubscribed
    /// - NonAssociatedColdKey : The supplied hotkey account id is not subscribed using the supplied cold key
    /// - NotEnoughStaketoWithdraw : The ammount of stake available in the hotkey account is lower than the requested amount
    /// - CouldNotConvertToBalance : A conversion error occured while converting stake from u64 to Balance
    ///
    pub fn do_remove_stake(origin: T::Origin, hotkey: T::AccountId, stake_to_be_removed: u64) -> dispatch::DispatchResult {

        // ---- We check the transaction is signed by the caller
        // and retrieve the T::AccountId pubkey information.
        let coldkey = ensure_signed( origin )?;

        // --- Check to see if the hotkey is active
        ensure!( Hotkeys::<T>::contains_key( &hotkey ), Error::<T>::NotRegistered );

        // ---- Check that the coldkey owns the hotkey or throw a NonAssociatedColdKey error.
        ensure!( Coldkeys::<T>::get( hotkey.clone() ) == coldkey, Error::<T>::NonAssociatedColdKey);

        // ---- We check that the hotkey has enough stake to withdraw  and then withdraw from the account.
        ensure!(Self::has_enough_stake( &hotkey, stake_to_be_removed ), Error::<T>::NotEnoughStaketoWithdraw);

        // --- We remove the stake from the hotkey account.
        Self::remove_stake_from_hotkey_account( &hotkey, stake_to_be_removed );

        // --- We credit the stake amount to the coldkey.
        Self::add_balance_to_coldkey_account( &coldkey, stake_to_be_removed );

        // ---- Emit the unstaking event.
        Self::deposit_event(Event::StakeRemoved( hotkey, stake_to_be_removed ));

        // --- Done and ok.
        Ok(())
    }

    // --- Returns Option if the u64 converts to a balance
    // use .unwarp if the result returns .some().
    pub fn u64_to_balance(input: u64) -> Option<<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance>
    {
        input.try_into().ok()
    }

    /// Increases the amount of stake of the entire stake pool by the supplied amount
    ///
    pub fn increase_total_stake( increment: u64 ) {
        // --- We update the total staking pool with the new funds.
        let total_stake: u64 = TotalStake::<T>::get();
        debug_assert!(increment <= u64::MAX.saturating_sub(total_stake));
        TotalStake::<T>::put(total_stake.saturating_add(increment));
    }

    /// Reduces the amount of stake of the entire stake pool by the supplied amount
    ///
    pub fn decrease_total_stake( decrement: u64 ) {
        // --- We update the total staking pool with the removed funds.
        let total_stake: u64 = TotalStake::<T>::get();
        // Sanity check so that total stake does not underflow past 0
        debug_assert!(decrement <= total_stake);
        TotalStake::<T>::put(total_stake.saturating_sub(decrement));
    }

    /// Increases the amount of stake in a hotkey account by the amount provided
    /// The uid parameter identifies the hotkey holding the hotkey account
    ///
    /// Calling function should make sure the uid exists within the system
    /// This function should always increase the total stake, so the operation
    /// of inserting new stake for a hotkey and the increment of the total stake is
    /// atomic. This is important because at some point the fraction of stake/total stake
    /// is calculated and this should always <= 1. Having this function be atomic, fills this
    /// requirement.
    ///
    pub fn add_stake_to_hotkey_account( hotkey: &T::AccountId, amount: u64 ) {
        let prev_stake: u64 = Stake::<T>::get( hotkey );
        debug_assert!(u64::MAX.saturating_sub(amount) > prev_stake);
        let new_stake: u64 = prev_stake.saturating_add(amount);
        Stake::<T>::insert( hotkey, new_stake );
        Self::increase_total_stake(amount);
    }

    pub fn get_stake_on_hotkey_account( hotkey: &T::AccountId ) -> u64 {
        if Stake::<T>::contains_key( hotkey ) {
            return Stake::<T>::get( hotkey );
        } else {
            return 0;
        }  
    }


    /// Decreases the amount of stake in hotkey account by the amount provided
    /// The uid parameter identifies the hotkey holding the hotkey account.
    /// When using this function, it is important to also increase another account by the same value,
    /// as otherwise value gets lost.
    ///
    /// A check if there is enough stake in the hotkey account should have been performed
    /// before this function is called. If not, the node will crap out.
    ///
    /// Furthermore, a check to see if the uid is active before this method is called is also required
    ///
    pub fn remove_stake_from_hotkey_account( hotkey: &T::AccountId, amount: u64 ) {
        let prev_stake: u64 = Stake::<T>::get( hotkey );
        debug_assert!(prev_stake >= amount);
        let new_stake: u64 = prev_stake.saturating_sub( amount );
        Stake::<T>::insert( hotkey, new_stake) ;
        Self::decrease_total_stake( amount );
    }

    /// This adds stake (balance) to a cold key account. It takes the account id of the coldkey account and a Balance as parameters.
    /// The Balance parameter is a from u64 converted number. This is needed for T::Currency to work.
    /// Make sure stake is removed from another account before calling this method, otherwise you'll end up with double the value
    ///
    pub fn add_balance_to_coldkey_account( coldkey: &T::AccountId, amount: u64 ) {
        let amount = Self::u64_to_balance( amount ).unwrap();
        T::Currency::deposit_creating( &coldkey, amount ); // Infallibe
    }

    /// This removes stake from the hotkey. This should be used together with the function to store the stake
    /// in the hot key account.
    /// The internal mechanics can fail. When this happens, this function returns false, otherwise true
    /// The output of this function MUST be checked before writing the amount to the hotkey account
    ///
    ///
    pub fn remove_balance_from_coldkey_account( coldkey: &T::AccountId, amount: u64 ) -> bool {
        let amount = Self::u64_to_balance( amount ).unwrap();
        return match T::Currency::withdraw(&coldkey, amount, WithdrawReasons::except(WithdrawReasons::TIP), ExistenceRequirement::KeepAlive) {
            Ok(_result) => {
                true
            }
            Err(_error) => {
                false
            }
        };
    }

    /// Checks if the coldkey account has enough balance to be able to withdraw the specified amount.
    ///
    pub fn can_remove_balance_from_coldkey_account( coldkey: &T::AccountId, amount: u64 ) -> bool {
        let amount = Self::u64_to_balance( amount ).unwrap();
        let current_balance = Self::get_coldkey_balance(coldkey);
        if amount > current_balance {
            return false;
        }
        // This bit is currently untested. @todo
        let new_potential_balance = current_balance - amount;
        let can_withdraw = T::Currency::ensure_can_withdraw( &coldkey, amount, WithdrawReasons::except(WithdrawReasons::TIP), new_potential_balance ).is_ok();
        can_withdraw
    }

    /// Returns the current balance in the cold key account
    ///
    pub fn get_coldkey_balance( coldkey: &T::AccountId ) -> <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance {
        return T::Currency::free_balance(&coldkey);
    }

    /// Checks if the hotkey account of the specified account has enough stake to be able to withdraw
    /// the requested amount.
    ///
    pub fn has_enough_stake( hotkey: &T::AccountId, amount: u64 ) -> bool {
        if Stake::<T>::contains_key( hotkey ) {
            return Stake::<T>::get( hotkey ) >= amount;
        } else {
            return false;
        }
    }


}

