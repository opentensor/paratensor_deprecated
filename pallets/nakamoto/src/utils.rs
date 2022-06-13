
use super::*;
use frame_support::inherent::Vec;
use frame_support::IterableStorageMap;
use frame_support::sp_std::vec;

#[cfg(feature = "no_std")]
extern crate nalgebra;

impl<T: Config> Pallet<T> {

    /// Misc
    pub fn is_hotkey_active( hotkey: &T::AccountId ) -> bool { return Uids::<T>::contains_key( hotkey ) }
    pub fn get_uid( hotkey: &T::AccountId ) -> u16 { return Uids::<T>::get( hotkey ) }
    pub fn is_uid_active( uid: u16 ) -> bool { return Active::<T>::get()[uid as usize] }

    /// Hyper-params
    pub fn get_min_allowed_weights() -> u16 { return MinAllowedWeights::<T>::get() }
    pub fn get_max_allowed_max_min_ratio() -> u16 { return MaxAllowedMaxMinRatio::<T>::get() }

    /// Consensus term getters.
    pub fn get_n() -> u16 { return N::<T>::get() }
    pub fn get_rank() -> Vec<u16> { return Rank::<T>::get() }
    pub fn get_trust() -> Vec<u16> { return Trust::<T>::get() }
    pub fn get_incentive() -> Vec<u16> { return Incentive::<T>::get() }
    pub fn get_consensus() -> Vec<u16> { return Consensus::<T>::get() }
    pub fn get_dividends() -> Vec<u16> { return Dividends::<T>::get() }
    pub fn get_emission() -> Vec<u64> { return Emission::<T>::get() }

    /// Consensus vector term getters.
    pub fn get_rank_as_vector() -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Rank::<T>::get() ) }
    pub fn get_trust_as_vector() -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Trust::<T>::get() ) }
    pub fn get_conensus_as_vector() -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Consensus::<T>::get() ) }
    pub fn get_incentive_as_vector() -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Incentive::<T>::get() ) }
    pub fn get_dividends_as_vector() -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Dividends::<T>::get() ) }
    pub fn get_emission_as_vector() -> nalgebra::DVector<u64> { nalgebra::DVector::from_vec( Emission::<T>::get() ) }

    /// Consensus float vector term getters.
    pub fn get_rank_as_float_vector() -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get().iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
    pub fn get_trust_as_float_vector() -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get().iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
    pub fn get_consensus_as_float_vector() -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get().iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
    pub fn get_incentive_as_float_vector() -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get().iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
    pub fn get_dividends_as_float_vector() -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get().iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
    
    /// Weights
    pub fn get_weights_as_matrix() -> nalgebra::DMatrix<u16> { 
        let n = Self::get_n() as usize;
        let mut weights: Vec<u16> = vec![ 0; n * n ];
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageMap<u16, Vec<(u16, u16)> >>::iter() {
            for (uid_j, weight_ij) in weights_i.iter() { weights [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *weight_ij;}
        }
        nalgebra::DMatrix::from_vec(n, n, weights ) 
    } 
    pub fn get_weights_as_float_matrix() -> nalgebra::DMatrix<f32> { 
        let n = Self::get_n() as usize;
        let mut weights: Vec<f32> = vec![ 0.0; n * n ];
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageMap<u16, Vec<(u16, u16)> >>::iter() {
            for (uid_j, weight_ij) in weights_i.iter() { weights [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *weight_ij as f32 / u16::MAX as f32;}
        }
        nalgebra::DMatrix::from_vec(n, n, weights ) 
    } 

    /// Bonds
    pub fn get_bonds_as_matrix() -> nalgebra::DMatrix<u16> { 
        let n = Self::get_n() as usize;
        let mut bonds: Vec<u16> = vec![ 0; n * n ];
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageMap<u16, Vec<(u16, u16)> >>::iter() {
            for (uid_j, bonds_ij) in bonds_i.iter() { bonds [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *bonds_ij;}
        }
        nalgebra::DMatrix::from_vec(n, n, bonds ) 
    } 
    pub fn get_bonds_as_float_matrix() -> nalgebra::DMatrix<f32> { 
        let n = Self::get_n() as usize;
        let mut bonds: Vec<f32> = vec![ 0.0; n * n ];
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageMap<u16, Vec<(u16, u16)> >>::iter() {
            for (uid_j, bonds_ij) in bonds_i.iter() { bonds [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *bonds_ij as f32 / u16::MAX as f32;}
        }
        nalgebra::DMatrix::from_vec(n, n, bonds ) 
    } 

    /// Consensus term uid getters.
    pub fn get_rank_for_uid( uid: u16 ) -> u16 { return Rank::<T>::get()[uid as usize] }
    pub fn get_trust_for_uid( uid: u16 ) -> u16 { return Trust::<T>::get()[uid as usize] }
    pub fn get_incentive_for_uid( uid: u16 ) -> u16 { return Incentive::<T>::get()[uid as usize] }
    pub fn get_consensus_for_uid( uid: u16 ) -> u16 { return Consensus::<T>::get()[uid as usize] }
    pub fn get_dividends_for_uid( uid: u16 ) -> u16 { return Dividends::<T>::get()[uid as usize] }
    pub fn get_emission_for_uid( uid: u16 ) -> u64 { return Emission::<T>::get()[uid as usize] }

    /// Consensus term setters.
    pub fn set_rank( rank: Vec<u16> ) { return Rank::<T>::put( rank ) }
    pub fn set_trust( trust: Vec<u16> ) { return Trust::<T>::put( trust ) }
    pub fn set_incentive( incentive: Vec<u16> ) { return Incentive::<T>::put( incentive ) }
    pub fn set_consensus( consensus: Vec<u16> ) { return Consensus::<T>::put( consensus ) }
    pub fn set_dividends( dividends: Vec<u16> ) { return Dividends::<T>::put( dividends ) }
    pub fn set_emission( emission: Vec<u64> ) { return Emission::<T>::put( emission ) }

    /// Consensus term uid setters.
    pub fn set_rank_for_uid( uid: u16, rank: u16 ) { 
        let mut vec: Vec<u16> = Self::get_rank(); 
        vec[uid as usize] = rank;
        Rank::<T>::put( vec ) 
    }
    pub fn set_trust_for_uid( uid: u16, trust: u16 ) { 
        let mut vec: Vec<u16> = Self::get_trust(); 
        vec[uid as usize] = trust;
        Trust::<T>::put( vec ) 
    }
    pub fn set_incentive_for_uid( uid: u16, incentive: u16 ) { 
        let mut vec: Vec<u16> = Self::get_incentive(); 
        vec[uid as usize] = incentive;
        Incentive::<T>::put( vec ) 
    }
    pub fn set_consensus_for_uid( uid: u16, consensus: u16 ) { 
        let mut vec: Vec<u16> = Self::get_consensus(); 
        vec[uid as usize] = consensus;
        Consensus::<T>::put( vec ) 
    }
    pub fn set_dividends_for_uid( uid: u16, dividends: u16 ) { 
        let mut vec: Vec<u16> = Self::get_dividends(); 
        vec[uid as usize] = dividends;
        Dividends::<T>::put( vec ) 
    }
    pub fn set_emission_for_uid( uid: u16, emission: u64 ) { 
        let mut vec: Vec<u64> = Self::get_emission(); 
        vec[uid as usize] = emission;
        Emission::<T>::put( vec ) 
    }

    /// Accounts
    pub fn add_account_under_uid( uid: u16, hotkey: &T::AccountId, coldkey: &T::AccountId ) {
        if !Hotkeys::<T>::contains_key( uid ) { 
            Hotkeys::<T>::insert( uid, hotkey.clone() );
            Coldkeys::<T>::insert( uid, coldkey.clone() );
            Uids::<T>::insert( hotkey.clone(), uid );
        }
    }    
    pub fn remove_account_under_uid( uid: u16 ) {
        if !Hotkeys::<T>::contains_key( uid ) { 
            let hotkey = Hotkeys::<T>::get( uid );
            Hotkeys::<T>::remove( uid );
            Coldkeys::<T>::remove( uid );
            Uids::<T>::remove( hotkey );
        }
    }

}
