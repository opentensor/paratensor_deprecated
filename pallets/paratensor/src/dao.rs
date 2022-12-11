use super::*;
use substrate_fixed::types::I64F64;
use frame_support::storage::IterableStorageDoubleMap;

impl<T: Config> Pallet<T> {

    /// ---- The implementation for the extrinsic add_dao_stake: Adds stake to the dao contract account from the coldkey.
    ///
    /// # Args:
    /// 	* 'origin': (<T as frame_system::Config>Origin):
    /// 		- The signature of the caller's coldkey.
    ///
    /// 	* 'stake_to_be_added' (u64):
    /// 		- The amount of stake to be added to the dao contract.
    ///
    pub fn do_add_dao_stake( origin: T::Origin, daokey: T::AccountId, stake_to_be_added: u64 ) -> dispatch::DispatchResult
    {
        // --- 1. We check that the transaction is signed by the caller and retrieve the T::AccountId pubkey information.
        let coldkey = ensure_signed( origin )?;
   
        // --- 2. Convert u64 to balance.
        let stake_as_balance = Self::u64_to_balance(stake_to_be_added);
        ensure!( stake_as_balance.is_some(), Error::<T>::CouldNotConvertToBalance );
 
        // --- 3. Check if we have enough funds to withdrawl from the coldkey and add to the hotkey.
        ensure!( Self::can_remove_balance_from_coldkey_account( &coldkey, stake_as_balance.unwrap() ), Error::<T>::NotEnoughBalanceToStake );

        // --- 4. Remove tao from coldkey.
        // This can throw a BalanceWidthdrawError so we remove from the coldkey first before adding to the hotkey.
        ensure!( Self::remove_balance_from_coldkey_account( &coldkey, stake_as_balance.unwrap() ) == true, Error::<T>::BalanceWithdrawalError );

        // --- 5. Add the funds to the dao_stake account.
        Self::add_stake_to_dao_account( &daokey, &coldkey, stake_to_be_added );
 
        // --- 7. Emit the staking event.
        Self::deposit_event( Event::DaoStakeAdded( daokey, coldkey, stake_to_be_added ) );
 
        // --- 8. ok and return.
        Ok( () )
     }
    

    /// ---- The implementation for the extrinsic remove_dao_stake: Removes stake from the dao contract account into the coldkey.
    ///
    /// # Args:
    /// 	* 'origin': (<T as frame_system::Config>Origin):
    /// 		- The signature of the caller's coldkey.
    ///
    /// 	* 'stake_to_be_added' (u64):
    /// 		- The amount of stake to be added to the dao contract.
    ///
    pub fn do_remove_dao_stake( origin: T::Origin, daokey: T::AccountId, stake_to_be_removed: u64 ) -> dispatch::DispatchResult {

        // --- 1. We check the transaction is signed by the caller
        // and retrieve the T::AccountId pubkey information.
        let coldkey = ensure_signed( origin )?;

        // --- 2. We check that the coldkey has enough dao stake to withdrawl.
        ensure!( DaoStake::<T>::get( &daokey, &coldkey ) >= stake_to_be_removed, Error::<T>::NotEnoughStaketoWithdraw );
        
        // --- 3. We convert the stake_to_be_removed to a balance.
        let stake_to_be_added_as_currency = Self::u64_to_balance( stake_to_be_removed );
        ensure!( stake_to_be_added_as_currency.is_some(), Error::<T>::CouldNotConvertToBalance );

        // --- 4. Remove funds from the dao staking account.
        Self::remove_stake_from_dao_account( &daokey, &coldkey, stake_to_be_removed );

        // --- 5. Add funds to coldkey account. 
        Self::add_balance_to_coldkey_account( &coldkey, stake_to_be_added_as_currency.unwrap() );

        // --- 6. Emit the unstaking event.
        Self::deposit_event( Event::DaoStakeRemoved( daokey, coldkey, stake_to_be_removed ) );

        // --- 7. Done and ok.
        Ok( () )
    }

    // Distributes the rao_emission onto the dao associated with the daokey.
    pub fn distribute_rao_emission_to_dao( daokey: &T::AccountId, rao_emission: u64 ) {
        
        // 1. Get the total sum of stake on dao.
        let total_dao_stake: u64 = Self::get_total_dao_stake( daokey );

        // 2. Determine the amount taken by the dao.
        let dao_take: u64 = Self::get_dao_take( daokey, rao_emission );

        // 3. Add the dao take to the daokey account.
        Self::add_stake_to_dao_account( daokey, daokey, dao_take );

        // 3. Add the dao take to the daokey account.
        let remaining_rao_emission: u64 = rao_emission.saturating_sub( dao_take );

        // 4. Iterate and distribute the remaining funds to the dao donators.
        for ( coldkey, dao_stake ) in < DaoStake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64 >>::iter_prefix( daokey ) {
            // Determine the proprotion of emission due to this coldkey.
            let rao_distribution: u64 = get_dao_stake_proportional_emission( 
                remaining_rao_emission, 
                dao_stake, 
                total_dao_stake 
            );

            // Add stake to accounts.
            Self::add_stake_to_dao_account( daokey, &coldkey, rao_distribution );
         }

    }

    /// Increase the amount of stake in a dao coldkey account by the passed amount.
    pub fn add_stake_to_dao_account( daokey: &T::AccountId, coldkey: &T::AccountId, amount: u64 ) {

        // Get previous dao stake balance.
        let prev_coldkey_dao_stake: u64 = DaoStake::<T>::get( daokey, coldkey );

        // Add amount to previous dao stake.
        let new_coldkey_dao_stake = prev_coldkey_dao_stake.saturating_add( amount );
        
        // Sink result to map under coldkey,
        DaoStake::<T>::insert( daokey, coldkey, new_coldkey_dao_stake );

        // Increase total DAO stake counter.
        Self::increase_total_dao_stake( daokey, amount );
    }

    /// Decreases the amount of stake in a dao coldkey account by the passed amount.
    pub fn remove_stake_from_dao_account( daokey: &T::AccountId, coldkey: &T::AccountId, amount: u64 ) {
        
        // Get previous dao stake balance.
        let prev_coldkey_dao_stake = DaoStake::<T>::get( daokey, coldkey );

        // Subtract amount from prev stake.
        let new_coldkey_dao_stake = prev_coldkey_dao_stake.saturating_sub( amount );
        
        // Sink result to map under coldkey,
        DaoStake::<T>::insert( daokey, coldkey, new_coldkey_dao_stake );

        // Decrease total DAO stake counter.
        Self::decrease_total_dao_stake( daokey, amount );
    }

    /// Increases the amount of dao stake.
    pub fn increase_total_dao_stake( daokey: &T::AccountId, increment: u64 ) {
        let prev_total_dao_stake: u64 = Self::get_total_dao_stake( daokey );
        TotalDaoStake::<T>::insert( daokey, prev_total_dao_stake.saturating_add( increment ) );
    }

    /// Decreases the amount of dao stake.
    pub fn decrease_total_dao_stake( daokey: &T::AccountId, increment: u64 ) {
        let prev_total_dao_stake: u64 = Self::get_total_dao_stake( daokey );
        TotalDaoStake::<T>::insert( daokey, prev_total_dao_stake.saturating_sub( increment ) );
    }

    /// Get total dao stake under daokey.
    pub fn get_total_dao_stake( daokey: &T::AccountId ) -> u64 {
        TotalDaoStake::<T>::get( daokey )
    }

    /// Get the dao take (the amount of the emission taken by the dao.)
    pub fn get_dao_take( daokey: &T::AccountId, rao_emission: u64 ) -> u64 {
        let dao_take_proportion: I64F64 = I64F64::from_num( DaoFundTake::<T>::get( daokey ) ) / I64F64::from_num( u64::MAX );
        let dao_take: I64F64 = I64F64::from_num( rao_emission ) * dao_take_proportion;
        dao_take.to_num::<u64>()
    }
    
}

// Returns the emission portion due to coldkey based on its dao_stake.
#[allow(dead_code)]
pub fn get_dao_stake_proportional_emission( rao_emission: u64, dao_stake: u64, total_dao_stake: u64 ) -> u64 { 
    // Proportion: rao_emission * ( dao_stake / total_dao_stake )
    let dao_stake_proportion: I64F64 = I64F64::from_num( dao_stake ) / I64F64::from_num( total_dao_stake );
    let emission_proportion: I64F64 = I64F64::from_num( rao_emission ) * dao_stake_proportion;
    emission_proportion.to_num::<u64>()
}
