use super::*;
use sp_runtime::sp_std::if_std;
use substrate_fixed::types::I110F18;
use frame_support::storage::IterableStorageMap;

impl<T: Config> Pallet<T> { 


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