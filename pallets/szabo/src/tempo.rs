use super::*;
use sp_runtime::sp_std::if_std;
use frame_support::inherent::Vec;

impl<T: Config> Pallet<T> {

    pub fn global_step( debug: bool ){
        Self::update_pending_emission( debug );
        Self::run_epochs( debug );
    }

    pub fn update_pending_emission( debug: bool ) {
        // pending_emission: Tokens remaining to be distributed into network i at position i.
        let prev_pending_emission: ndarray::Array1<u64> = Self::get_pending_emission_as_array();
        if debug { if_std! { println!( "prev_pending_emission:\n{}\n", prev_pending_emission.clone() ); } }

        // emission_distribution: The distribution of emission over the subnetworks.
        let mut emission_distribution: ndarray::Array1<f32> = Self::get_emission_distribution_as_float_array();
        Self::vector_normalize( &mut emission_distribution );
        if debug { if_std! { println!( "emission_distribution:\n{}\n", emission_distribution.clone() ); } }

        // block_emission: The total number of new tokens to emit this block.
        // Usually a single token 1_000_000_000 Until the halving.
        let block_emission: u64 = Self::get_block_emission();
        if debug { if_std! { println!( "block_emission:\n{}\n", block_emission); } }

        // network_emission_f32: The number of newly minted tokens (as floats) being minted into each network.
        let network_emission_f32: ndarray::Array1<f32> = (block_emission as f32) * emission_distribution;
        if debug { if_std! { println!( "network_emission_f32:\n{}\n", network_emission_f32.clone() ); } }

        // network_emission_u64: The number of newly minted tokens (as ints) being minted into each network.
        let network_emission_u64: ndarray::Array1<u64> = network_emission_f32.map( |e| (*e as u64) );
        if debug { if_std! { println!( "network_emission_u64:\n{}\n", network_emission_u64.clone() ); } }

        // next_pending_emission: Pending emission for each network.
        let next_pending_emission: ndarray::Array1<u64> = prev_pending_emission + network_emission_u64;
        if debug { if_std! { println!( "next_pending_emission:\n{}\n", next_pending_emission.clone() ); } }
        Self::set_pending_emission_from_array( next_pending_emission );
    }

    pub fn run_epochs( debug: bool ) {
        // Iterate through sub networks and apply epoch if we are on their tempo.
        let block_number: u64 = Self::get_current_block_as_u64() + 1;
        let tempo: ndarray::Array1<u64> = Self::get_tempo_as_array();
        let mut pending_emission: ndarray::Array1<u64> = Self::get_pending_emission_as_array();
        if debug { if_std! { println!( "tempo:\n{}\n", tempo.clone()); } }
        if debug { if_std! { println!( "pending_emission(n):\n{}\n", pending_emission.clone()); } }

        // Iterate through the network uids, if the tempo mod block number is zero
        // We distribute the pending emission.
        for (netuid, tempo_i) in tempo.iter().enumerate() {

            // Check if tempo is reached.
            let netuid: u16 = netuid as u16;
            if debug { if_std! { println!( "Netuid:\n{}\nTempo:\n{}\nEmit:\n{}%{}={}\n", netuid, tempo_i, block_number, tempo_i, block_number % tempo_i == 0 ); } }
            if block_number % tempo_i == 0 {
                // Get the pending emission for this network.
                let network_emission: u64 = pending_emission [ netuid as usize ];
                if debug { if_std! { println!( "network_emission:\n{}\n", network_emission); } }

                // Run epoch step.
                let stake_emission_for_network: ndarray::Array1<u64> = Self::epoch( netuid, network_emission, debug );
                if debug { if_std! { println!( "stake_emission_for_network:\n{}\n", stake_emission_for_network); } }

                // Apply stake update.
                if debug { if_std! { println!( "get_stake_as_array(n):\n{}\n", Self::get_stake_as_array( netuid ) ); } }
                Self::increment_stake_from_emission( netuid, stake_emission_for_network );
                if debug { if_std! { println!( "get_stake_as_array(n+1):\n{}\n", Self::get_stake_as_array( netuid )); } }

                // Remove distributed emission
                pending_emission[ netuid as usize ] = 0;
            }
        }   
        if debug { if_std! { println!( "pending_emission(n+1): \n{}\n", pending_emission.clone()); } }
        Self::set_pending_emission_from_array( pending_emission );
    }
}
