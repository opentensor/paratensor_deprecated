use super::*;
use substrate_fixed::types::I110F18;
use frame_support::inherent::Vec;
use frame_support::storage::IterableStorageMap;
use frame_support::storage::IterableStorageDoubleMap;

impl<T: Config> Pallet<T> { 

    pub fn block_step() {
        // Adjust difficulties.
		Self::adjust_registration_difficulties();
		// Distribute emission.
		Self::distribute_pending_emission_onto_networks();
        // Run epochs.
        Self::run_epochs_and_emit();
    }

    /// Distributes pending emission onto each network based on the emission vector.
    ///
    pub fn distribute_pending_emission_onto_networks() {
        // --- 1. We iterate across each network and add the emission value onto the network's pending emission.
        // The pending emission will acrue until this network runs its epoch function.
        for (netuid_i, _) in <SubnetworkN<T> as IterableStorageMap<u16, u16>>::iter(){ 
            // --- 2. Get the emission value for this network which is a value < block emission
            // and all emission values sum to block_emission() 
            let pending_emission = EmissionValues::<T>::get(netuid_i);
            PendingEmission::<T>::mutate(netuid_i, |val| *val += pending_emission);
        }
    }

    /// Runs each network epoch function based on tempo.
    ///
    pub fn run_epochs_and_emit() {
        // --- 1. First get the current block number which will be used to determine which networks 
        // we will be draining of pending emission.
        let block_number = Self::get_current_block_as_u64();  

        // --- 2. Next we will iterate over all active networks via tempo and distribute the 
        // emission if it is the networks time to run the epoch.
        for ( netuid_i, tempo_i )  in <Tempo<T> as IterableStorageMap<u16, u16>>::iter() {
            
            // --- 3. Check to see if this network has hit its tempo.
            // NOTE(const): Because we check ( block_number + 1 ) % ( tempo_i as u64 + 1 ) == 0
            // We begin on the first block.
            // tempo = 0, run every block.
            // tempo = 1, skip 1 block then run
            // tempo = 2, skip 2 blocks then run ...
            if ( block_number + 1 ) % ( tempo_i as u64 + 1 ) == 0 {

                // --- 4. We are running this network so we attain the pending emission
                // and drain it. These tokens will be run through the mechanism and potentially 
                // create a remainder.
                let pending_emission:u64 = PendingEmission::<T>::get( netuid_i );

                // --- 5. Run the mechanism for this network updating consensus parameters
                // and returns the tao_emission, a positive valued u64. The sum of these value 
                // should equal pending_emission.
                let tao_emission: Vec<u64> = Self::epoch_sparse( netuid_i, pending_emission, true );

                // --- 6. We now distribute the tao emission onto the subnetwork hotkey staking accounts.
                // The remainder will be added back onto the pending emission for this network
                let tao_remainder: u64 = Self::distribute_emission_to_accounts_with_remainder( netuid_i, tao_emission, pending_emission );

                // --- 7. Add the remainder back to the pending.
                PendingEmission::<T>::insert( netuid_i, tao_remainder );
            } 
        }
    }

    /// Distributes pending emission onto each network based on the emission vector.
    ///
    /// # Args:
    /// 	* 'netuid': ( u16 ):
    ///         - The network to distribute the emission onto.
    /// 		
    /// 	* 'tao_emission': ( Vec<u64> ):
    ///         - The emission to distribute onto the accounts.
    ///
    /// 	* 'pending_emission' (u16):
    /// 		- The total allowed emission onto these accounts.
    ///    
    pub fn distribute_emission_to_accounts_with_remainder( netuid: u16, tao_emission: Vec<u64>, allowed_pending: u64 ) -> u64 {
        let len_tao_emission: u16 = tao_emission.len() as u16;

        // --- 1. If the network is empty return all the pending.
        if 0 == Self::get_subnetwork_n( netuid ) { return allowed_pending; }

        // --- 2. Check that the tao emission has an entry for each key. 
        // Otherwise return all pending emission.
        if len_tao_emission != Self::get_subnetwork_n( netuid ) { return allowed_pending; }

        // --- 3. Check that the sum of the tao emission is not greater than the 
        // allowed pending. 
        // NOTE(const): We are performing a sum on u128 to ensure we dont overflow.
        let emission_sum: u128 = tao_emission.iter().map( |x| *x as u128 ).sum();
        if emission_sum > allowed_pending as u128 { return allowed_pending; }

        // --- 4. If the sum is less than the allowed pending we can return this as the 
        // remainder. NOTE(const): this must be on the u64 range because allowed >= sum and allowed < u64::MAX.
        let remainder: u64 = allowed_pending - emission_sum as u64;

        // --- 5. Now lets distribute the tao emission onto the keys.
        for (uid_i, hotkey_i) in <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix(netuid) { 
            // Check uids.
            let stake_to_add: u64 = tao_emission[ uid_i as usize ];
            Self::emit_inflation_through_hotkey_account( &hotkey_i, stake_to_add );
        }

        remainder
    }


    /// Adjusts the network difficulty of every active network. Reseting state parameters.
    ///
    pub fn adjust_registration_difficulties() {
        for ( netuid, _ )  in <NetworksAdded<T> as IterableStorageMap<u16, bool>>::iter(){
            let last_adjustment_block: u64 = Self::get_last_adjustment_block( netuid );
            let adjustment_interval: u16 = Self::get_adjustment_interval( netuid );
            let current_block: u64 = Self::get_current_block_as_u64( ); 
            if ( current_block - last_adjustment_block ) >= adjustment_interval as u64 {
                let current_difficulty: u64 = Self::get_difficulty_as_u64( netuid );
                let get_registrations_this_interval: u16 = Self::get_registrations_this_interval( netuid );
                let target_registrations_this_interval: u16 = Self::get_target_registrations_per_interval( netuid );
                let adjusted_difficulty: u64 = Self::get_next_difficulty( 
                    netuid,
                    current_difficulty,
                    get_registrations_this_interval,
                    target_registrations_this_interval
                );
                Self::set_difficulty( netuid, adjusted_difficulty );
                Self::set_last_adjustment_block( netuid, current_block );
                Self::set_registrations_this_interval( netuid, 0 );

            }
            Self::set_registrations_this_block( netuid, 0 );
        }
    }

    /// Performs the difficutly adjustment by multiplying the current difficulty by the ratio ( registrations_this_interval + target_registrations_per_interval / target_registrations_per_interval * target_registrations_per_interval )
    /// We use I110F18 to avoid any overflows on u64.
    ///
    pub fn get_next_difficulty( 
        netuid: u16,
        current_difficulty: u64, 
        registrations_this_interval: u16, 
        target_registrations_per_interval: u16 
    ) -> u64 {
        let next_value: I110F18 = I110F18::from_num( current_difficulty ) * I110F18::from_num( registrations_this_interval + target_registrations_per_interval ) / I110F18::from_num( target_registrations_per_interval + target_registrations_per_interval );
        if next_value >= I110F18::from_num( Self::get_max_difficulty( netuid ) ){
            return Self::get_max_difficulty( netuid );
        } else if next_value <= I110F18::from_num( Self::get_min_difficulty( netuid ) ) {
            return Self::get_min_difficulty( netuid );
        } else {
            return next_value.to_num::<u64>();
        }
    }

}