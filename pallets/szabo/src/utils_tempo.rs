
use super::*;
use frame_support::inherent::Vec;
use frame_support::sp_std::vec;
use frame_support::storage::IterableStorageMap;

#[cfg(feature = "no_std")]
use ndarray::{ndarray::Array1, ndarray::Array2, ndarray::arr1};

impl<T: Config> Pallet<T> {

    /// ==============
	/// ==== Misc ====
	/// ==============
    pub fn get_block_emission() -> u64 { 1_000_000_000 }
    pub fn get_num_subnetworks() -> u16 { NumSubnetworks::<T>::get() as u16 }
    pub fn set_num_subnetworks( n: u16 ) { NumSubnetworks::<T>::put( n ); }
    pub fn increment_num_subnetworks() { let n = Self::get_num_subnetworks();if n < u16::MAX { Self::set_num_subnetworks(n + 1); }}
    pub fn decrement_num_subnetworks() { let n = Self::get_num_subnetworks(); Self::set_num_subnetworks(n - 1); }
    pub fn increment_stake_from_emission( netuid: u16, stake_emission_array: ndarray::Array1<u64> ) {
        for uid in 0..stake_emission_array.len() {
            let hotkey: T::AccountId = Keys::<T>::get( netuid, uid as u16 );
            let prev_stake: u64 = Stake::<T>::get( hotkey.clone() );
            let next_stake: u64 = prev_stake + stake_emission_array[ uid as usize ];
            Stake::<T>::insert( hotkey, next_stake );
        }
    }

    /// =================
	/// ==== Pending ====
	/// =================
    pub fn set_pending_emission_for_network( netuid:u16, pending_emission:u64 ) { PendingEmission::<T>::insert( netuid, pending_emission ) }
    pub fn get_pending_emission_as_vector() -> Vec<(u16, u64)> { 
        let mut tempo_vec: Vec<(u16, u64)> = vec![];
        for ( netuid, tempo ) in < PendingEmission<T> as IterableStorageMap<u16, u64> >::iter() {
            tempo_vec.push( (netuid, tempo ) );
        }
        return tempo_vec
    }
    pub fn get_pending_emission_as_array() -> ndarray::Array1<u64> { 
        let n: u16 = Self::get_num_subnetworks();
        let mut pending_emission = ndarray::Array1::<u64>::zeros( n as usize );
        for ( netuid, pending ) in < PendingEmission<T> as IterableStorageMap<u16, u64> >::iter() {
            pending_emission[netuid as usize] = pending;
        }
        return pending_emission
    }
    pub fn set_pending_emission_from_array( pending_emission: ndarray::Array1<u64> ) { 
        for (netuid, pending) in pending_emission.iter().enumerate() {
            if *pending != 0 {
                PendingEmission::<T>::insert( netuid as u16, pending );
            }
        } 
    }

    /// ======================
	/// ==== Distribution ====
	/// ======================
    pub fn set_emission_distribution_for_netuid( netuid: u16, dist: u16 ) { 
        EmissionDistribution::<T>::insert( netuid, dist )
    }
    pub fn get_emission_distribution_as_float_array() -> ndarray::Array1<f32> { 
        let n: u16 = Self::get_num_subnetworks();
        let mut emission_distribution_array: ndarray::Array1<f32> = ndarray::Array1::<f32>::zeros( n as usize );
        for ( netuid, dist ) in < EmissionDistribution<T> as IterableStorageMap<u16, u16> >::iter() {
            emission_distribution_array[netuid as usize] = (dist as f32) / (u16::MAX as f32);
        }
        // Self::vector_normalize( &mut emission_distribution_array );
        return emission_distribution_array
    }

    /// ===============
	/// ==== Tempo ====
	/// ===============
    pub fn set_tempo_for_network( netuid:u16, tempo:u64 ) { Tempo::<T>::insert( netuid, tempo ) }
    pub fn get_tempo_as_vector() -> Vec<(u16, u64)> { 
        let mut tempo_vec: Vec<(u16, u64)> = vec![];
        for ( netuid, tempo ) in < Tempo<T> as IterableStorageMap<u16, u64> >::iter() {
            tempo_vec.push( (netuid, tempo ) );
        }
        return tempo_vec
    }
    pub fn get_tempo_as_array() -> ndarray::Array1<u64> { 
        let n: u16 = Self::get_num_subnetworks();
        let mut tempo_array = ndarray::Array1::<u64>::zeros( n as usize );
        for ( netuid, tempo ) in < Tempo<T> as IterableStorageMap<u16, u64> >::iter() {
            tempo_array[netuid as usize] = tempo;
        }
        return tempo_array
    }
   



}
