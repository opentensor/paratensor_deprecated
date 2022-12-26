use super::*;

impl<T: Config> Pallet<T> {

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

        // --- 4. Potentially create the global account. Note that this creates a global account and thus this transaction should be paid.
        Self::create_account_if_non_existent( &hotkey, &coldkey );         

        // --- 5. Ensure that the hot - cold pairing is correct, the hotkey is associated with this coldkey.
        ensure!( Self::account_belongs_to_coldkey( &hotkey, &coldkey ), Error::<T>::NonAssociatedColdKey );

        // --- 6. Ensure the remove operation from the coldkey is a success.
        ensure!( Self::remove_balance_from_coldkey_account( &coldkey, stake_as_balance.unwrap() ) == true, Error::<T>::BalanceWithdrawalError );

        // --- 7. If we reach here, add the balance to the hotkey.
        Self::add_stake_to_neuron_hotkey_account( &hotkey, stake_to_be_added );
 
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
    pub fn do_remove_stake(origin: T::Origin, hotkey: T::AccountId, stake_to_be_removed: u64) -> dispatch::DispatchResult {

        // --- 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed( origin )?;

        // --- 2. Ensure that the hotkey exists as an active account. Otherwise there is nothing to withdraw
        ensure!( Self::account_exists( &hotkey ), Error::<T>::NotRegistered );

        // --- 3. Ensure that the hot - cold pairing is correct, the hotkey is associated with this coldkey.
        ensure!( Self::account_belongs_to_coldkey( &hotkey, &coldkey ), Error::<T>::NonAssociatedColdKey );

        // --- 4. Ensure that the hotkey has enough stake to withdraw.
        ensure!( Self::has_enough_stake( &hotkey, stake_to_be_removed ), Error::<T>::NotEnoughStaketoWithdraw );

        // --- 5. Ensure that we can conver this u64 to a balance.
        let stake_to_be_added_as_currency = Self::u64_to_balance( stake_to_be_removed );
        ensure!( stake_to_be_added_as_currency.is_some(), Error::<T>::CouldNotConvertToBalance );

        // --- 6. We remove the balance from the hotkey.
        Self::remove_stake_from_hotkey_account( &hotkey, stake_to_be_removed );

        // --- 7. We add the balancer to the coldkey.  If the above fails we will not credit this coldkey.
        Self::add_balance_to_coldkey_account( &coldkey, stake_to_be_added_as_currency.unwrap() );

        // --- 8. Emit the unstaking event.
        Self::deposit_event( Event::StakeRemoved( hotkey, stake_to_be_removed ) );

        // --- 9. Done and ok.
        Ok(())
    }

    /// This adds stake (balance) to a cold key account. It takes the account id of the coldkey account and a Balance as parameters.
    /// The Balance parameter is a from u64 converted number. This is needed for T::Currency to work.
    /// Make sure stake is removed from another account before calling this method, otherwise you'll end up with double the value
    ///
    pub fn add_balance_to_coldkey_account(coldkey: &T::AccountId, amount: <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance) {
        T::Currency::deposit_creating(&coldkey, amount); // Infallibe
    }

    /// Reduces the amount of stake of the entire stake pool by the supplied amount
    ///
    pub fn decrease_total_stake(decrement: u64) {
        // --- We update the total staking pool with the removed funds.
        let total_stake: u64 = TotalStake::<T>::get();

        // Sanity check so that total stake does not underflow past 0
        debug_assert!(decrement <= total_stake);

        TotalStake::<T>::put(total_stake.saturating_sub(decrement));
    }

    /// Checks if the coldkey account has enough balance to be able to withdraw the specified amount.
    ///
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

    /// Returns the current balance in the cold key account
    ///
    pub fn get_coldkey_balance(coldkey: &T::AccountId) -> <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance {
        return T::Currency::free_balance(&coldkey);
    }

    /// This removes stake from the hotkey. This should be used together with the function to store the stake
    /// in the hot key account.
    /// The internal mechanics can fail. When this happens, this function returns false, otherwise true
    /// The output of this function MUST be checked before writing the amount to the hotkey account
    ///
    ///
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

    /// Increases the amount of stake in the hotkey account by the amount provided
    ///
    /// Calling function should make sure the uid exists within the system
    /// This function should always increase the total stake, so the operation
    /// of inserting new stake for a neuron and the increment of the total stake is
    /// atomic. This is important because at some point the fraction of stake/total stake
    /// is calculated and this should always <= 1. Having this function be atomic, fills this
    /// requirement.
    ///
    pub fn add_stake_to_neuron_hotkey_account(hotkey: &T::AccountId, amount: u64) {
        
        let prev_stake: u64 = Self::get_stake_for_hotkey(hotkey);

        // This should never happen. If a user has this ridiculous amount of stake,
        // we need to come up with a better solution
        debug_assert!(u64::MAX.saturating_sub(amount) > prev_stake);

        let new_stake = prev_stake.saturating_add(amount);
        Self::add_stake_for_hotkey(hotkey, new_stake);

        Self::add_stake_for_subnet(hotkey, amount);

        Self::increase_total_stake(amount);

    }

    /// Checks if the hotkey account of the specified account has enough stake to be able to withdraw
    /// the requested amount.
    ///
    pub fn has_enough_stake(hotkey: &T::AccountId, amount: u64) -> bool {
        let stake = Stake::<T>::get(hotkey);
        return stake >= amount;
    }

    /// Decreases the amount of stake in a hotkey account by the amount provided
    /// When using this function, it is important to also increase another account by the same value,
    /// as otherwise value gets lost.
    ///
    /// A check if there is enough stake in the hotkey account should have been performed
    /// before this function is called. If not, the node will crap out.
    ///
    /// Furthermore, a check to see if the uid is active before this method is called is also required
    ///
    pub fn remove_stake_from_hotkey_account(hotkey: &T::AccountId, amount: u64) {

        let hotkey_stake = Stake::<T>::get(hotkey);
        // By this point, there should be enough stake in the hotkey account for this to work.
        debug_assert!(hotkey_stake >= amount);
        let decreased_stake = hotkey_stake.saturating_sub(amount);

        Stake::<T>::insert(&hotkey, decreased_stake);
        Self::decrease_total_stake(amount);
        //
        Self::remove_stake_for_subnet(hotkey);
    }
    
    /// Increases the amount of stake of the entire stake pool by the supplied amount
    ///
    pub fn increase_total_stake( increment: u64) {

        let total_stake: u64 = TotalStake::<T>::get();
        // Sanity check
        debug_assert!(increment <= u64::MAX.saturating_sub(total_stake));

        TotalStake::<T>::put(total_stake.saturating_add(increment));
    }

    pub fn add_stake_for_subnet( hotkey: &T::AccountId, amount: u64){
        if Subnets::<T>::contains_key(&hotkey){
            let vec_new_hotkey_subnets = Subnets::<T>::get(&hotkey);
                for i in vec_new_hotkey_subnets{

                    let netuid = i;
                    let neuron_uid ;
                    match Self::get_neuron_for_net_and_hotkey(netuid, hotkey) {
                        Ok(k) => neuron_uid = k,
                        Err(e) => panic!("Error: {:?}", e),
                    } 

                    if S::<T>::contains_key(netuid, neuron_uid){

                        let prev_stake = S::<T>::get(netuid, neuron_uid);
                        S::<T>::insert(netuid, neuron_uid, prev_stake+amount);
                    }
                    else { S::<T>::insert(netuid, neuron_uid, amount);}
            }
        }
    }

    pub fn get_stake_of_neuron_hotkey_account(hotkey: &T::AccountId) -> u64 {
        Stake::<T>::get(hotkey)
    }
}