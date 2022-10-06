use super::*;

impl<T: Config> Pallet<T> {

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
}