use super::*;
use substrate_fixed::types::I110F18;
use frame_support::storage::IterableStorageMap;

impl<T: Config> Pallet<T> { 

    pub fn block_step() {
        // Adjust difficulties.
		Self::adjust_difficulty();
		// Distribute emission.
		Self::distribute_emission();
        // Run epochs.
        Self::run_epochs();
    }

    /// Runs each network epoch function based on tempo.
    ///
    pub fn run_epochs() {
        let block_number = Self::get_current_block_as_u64();  
        for ( netuid_i, tempo_i )  in <Tempo<T> as IterableStorageMap<u16, u16>>::iter() {
            if ( block_number + 1 ) % ( tempo_i as u64 + 1 ) == 0 {
                // We are going to drain this pending emission because the modulo tempo is zero.
                let net_emission:u64 = PendingEmission::<T>::get( netuid_i );
                // Distribute the emission through the epoch.
                Self::epoch( netuid_i, net_emission, true );
                // drain the pending emission at this step.
                PendingEmission::<T>::mutate( netuid_i, |val| *val *= 0 );
            } 
        }
    }

    /// Distributes pending emission onto each network based on the emission vector.
    ///
    pub fn distribute_emission() {
        for (netuid_i, _) in <SubnetworkN<T> as IterableStorageMap<u16, u16>>::iter(){ 
            if PendingEmission::<T>::contains_key(netuid_i) == false {
                PendingEmission::<T>::insert(netuid_i, 0);
            }
            let pending_emission = EmissionValues::<T>::get(netuid_i);
            PendingEmission::<T>::mutate(netuid_i, |val| *val += pending_emission);
        }
    }

    /// Adjusts the network difficulty of every active network. Reseting state parameters.
    ///
    pub fn adjust_difficulty() {
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