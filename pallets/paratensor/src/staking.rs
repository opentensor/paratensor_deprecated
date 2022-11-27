use super::*;

impl<T: Config> Pallet<T> {

      /***********************************************************
     * do_add_stake() - main function called from parent module
     ***********************************************************/
     /* TO DO:
     1. heck the transaction is signed by the caller and retrieve the T::AccountId coldkey. 
     2. Check if the hotkey is active
     3. check that the hotkey is linked to the calling cold key, otherwise throw a NonAssociatedColdKey error.
     4. check that the calling coldkey contains enough funds to create the staking transaction.
     5. transfer stake from coldkey to hotkey
     6. emit the staking event.*/


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
    pub fn do_add_stake(origin: T::Origin, hotkey: T::AccountId, stake_to_be_added: u64) -> dispatch::DispatchResult
    {
         // --- 1. We check that the transaction is signed by the caller and retrieve the T::AccountId pubkey information.
         let coldkey = ensure_signed(origin)?;
 
         // --- 2. We check if the hotkey is active on any subnetworks.
         // TODO(Saeideh): I think we should remove the functionality here where a peer cannot stake/unstake unless they are registered
         // Staking can be disjoint from registration to networks for instance if they are staking to the DAO contract.
         ensure!(Self::is_hotkey_registered_any(&hotkey), Error::<T>::NotRegistered);
 
         //  --- 3. We check that the hotkey is linked to the calling cold key, 
         // otherwise throw a NonAssociatedColdKey error.
         ensure!(Self::hotkey_belongs_to_coldkey(&hotkey, &coldkey), Error::<T>::NonAssociatedColdKey);
 
         //  --- 4. We check that the calling coldkey contains enough funds to
         // create the staking transaction.
         let stake_as_balance = Self::u64_to_balance(stake_to_be_added);
         ensure!(stake_as_balance.is_some(), Error::<T>::CouldNotConvertToBalance);
 
         // --- 5. We check if the staking coldkey has enough stake to add to the hotkey.
         // otherwise we throw a NotEnoughBalanceToStake error.
         ensure!(Self::can_remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()), Error::<T>::NotEnoughBalanceToStake);

         // --- 6. Transfer stake from coldkey to hotkey. Removing first from coldkey and then adding to the hotkey.
         // This can throw a BalanceWidthdrawError so we remove from the coldkey first before adding to the hotkey.
         ensure!(Self::remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()) == true, Error::<T>::BalanceWithdrawalError);
         Self::add_stake_to_neuron_hotkey_account(&hotkey, stake_to_be_added);
 
         // --- 7. Emit the staking event.
         Self::deposit_event(Event::StakeAdded(hotkey, stake_to_be_added));
 
         // --- ok and return.
         Ok(())
     }
    
    
    /// This function removes stake from a hotkey account and puts into a coldkey account.
    /// This function should be called through an extrinsic signed with the coldkeypair's private
    /// key. It takes a hotkey account id and an ammount as parameters.
    ///
    /// Generally, this function works as follows
    /// 1) A Check is performed to see if the hotkey is active (ie, the node using the key is subscribed)
    /// 2) If these checks pass, inflation is emitted to the nodes' peers
    /// 3) If the account has enough stake, the requested amount it transferred to the coldkey account
    /// 4) The total amount of stake is reduced after transfer is complete
    ///
    /// It throws the following errors if there is something wrong
    /// - NotRegistered : The suplied hotkey is not in use. This ususally means a node that uses this key has not subscribed yet, or has unsubscribed
    /// - NonAssociatedColdKey : The supplied hotkey account id is not subscribed using the supplied cold key
    /// - NotEnoughStaketoWithdraw : The ammount of stake available in the hotkey account is lower than the requested amount
    /// - CouldNotConvertToBalance : A conversion error occured while converting stake from u64 to Balance
    ///
    pub fn do_remove_stake(origin: T::Origin, hotkey: T::AccountId, stake_to_be_removed: u64) -> dispatch::DispatchResult {

        // ---- 1. We check the transaction is signed by the caller
        // and retrieve the T::AccountId pubkey information.
        let coldkey = ensure_signed(origin)?;

        // ---- 2. We check if hotkey is active on any subnetworks. Optionally throw not registered error.
        // TODO(Saeideh): Same todo as above.
        ensure!(Self::is_hotkey_registered_any(&hotkey), Error::<T>::NotRegistered);

        // ---- 3. We check that the hotkey is linked to the calling cold key, otherwise throw a NonAssociatedColdKey error.
        ensure!(Self::hotkey_belongs_to_coldkey(&hotkey, &coldkey), Error::<T>::NonAssociatedColdKey);

        // ---- 4. We check that the hotkey has enough stake to withdraw
        // and then withdraw from the account and convert to a balance currency object.
        ensure!(Self::has_enough_stake(&hotkey, stake_to_be_removed), Error::<T>::NotEnoughStaketoWithdraw);
        let stake_to_be_added_as_currency = Self::u64_to_balance(stake_to_be_removed);
        ensure!(stake_to_be_added_as_currency.is_some(), Error::<T>::CouldNotConvertToBalance);

        // --- 5. We perform the withdrawl by converting the stake to a u64 balance
        // and deposit the balance into the coldkey account. If the coldkey account
        // does not exist it is created.
        Self::remove_stake_from_hotkey_account(&hotkey, stake_to_be_removed);
        Self::add_balance_to_coldkey_account(&coldkey, stake_to_be_added_as_currency.unwrap());

        // ---- 6. Emit the unstaking event.
        Self::deposit_event(Event::StakeRemoved(hotkey, stake_to_be_removed));

        // --- Done and ok.
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

    pub fn hotkey_belongs_to_coldkey(hotkey: &T::AccountId, coldkey: &T::AccountId) -> bool {
        return Coldkeys::<T>::get(coldkey) == *hotkey;
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
                    let neuron_uid = Self::get_neuron_for_net_and_hotkey(netuid, hotkey);

                    if S::<T>::contains_key(netuid, neuron_uid){

                        let prev_stake = S::<T>::get(netuid, neuron_uid);
                        S::<T>::insert(netuid, neuron_uid, prev_stake+amount);
                    }
                    else { S::<T>::insert(netuid, neuron_uid, amount);}
            }
        }
    }
}