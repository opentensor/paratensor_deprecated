
use super::*;
use frame_support::inherent::Vec;
use frame_support::IterableDoubleStorageMap;
use frame_support::sp_std::vec;

#[cfg(feature = "no_std")]
extern crate nalgebra;

impl<T: Config> Pallet<T> {

	/// ================
	/// ==== Global ====
	/// ================
    pub fn get_global_n() -> u64 { return N::<T>::get() }
    pub fn get_total_issuance() -> u64 { return TotalIssuance::<T>::get() }
    pub fn get_total_stake() -> u64 { return TotalStake::<T>::get() }

    /// ==============
	/// ==== Misc ====
	/// ==============
    pub fn is_hotkey_active( netuid:u16, hotkey: &T::AccountId ) -> bool { return Uids::<T>::contains_key( netuid, hotkey ) }
    pub fn get_uid(  netuid:u16, hotkey: &T::AccountId ) -> u16 { return Uids::<T>::get( netuid, hotkey ) }
    pub fn is_uid_active(  netuid:u16, uid: u16 ) -> bool { return Active::<T>::get()[uid as usize] }

	/// ==================
	/// ==== Accounts ====
	/// ==================
    pub fn add_global_account( hotkey: &T::AccountId, coldkey: &T::AccountId )  {
        if !Hotkeys::<T>::contains_key( &hotkey ) { 
            Hotkeys::<T>::insert( hotkey.clone(), coldkey.clone() );
            Coldkeys::<T>::insert( coldkey.clone(), hotkey.clone() );
            Stake::<T>::insert( hotkey.clone(), 0 );
            Self::increment_n();
        }
    }
    pub fn increment_global_n() { let n = GlobalN::<T>::get(); if n < u64::MAX { GlobalN::<T>::put(n + 1); } }
    pub fn decrement_global_n() { let n = GlobalN::<T>::get(); if n > 0 { GlobalN::<T>::put(n - 1); } }
    pub fn increment_subnetwork_n( netuid:u16 ) { let n = SubnetorkN::<T>::get( netuid ); if n < u64::MAX { SubnetorkN::<T>::put(netuid, n + 1); } }
    pub fn decrement_subnetwork_n( netuid:u16 ) { let n = SubnetorkN::<T>::get( netuid ); if n > 0 { SubnetorkN::<T>::put(netuid, n - 1); } }

    pub fn insert_subnetwork_account( netuid:u16, uid: u16, hotkey: &T::AccountId ) { 
        Keys::<T>::insert( netuid, uid, hotkey.clone() );
        Uids::<T>::insert( netuid, hotkey.clone(), uid );
    }
    pub fn remove_subnetwork_account( netuid:u16, uid: u16 ) { 
        let hotkey = Keys::<T>::get( netuid, uid );
        Uids::<T>::remove( netuid, hotkey.clone() );
        Keys::<T>::remove( netuid, uid ); 
    }

    /// =====================
	/// ==== Hyperparams ====
	/// =====================
    pub fn get_min_allowed_weights( netuid:u16 ) -> u16 { return MinAllowedWeights::<T>::get( netuid ) }
    pub fn get_max_allowed_max_min_ratio( netuid:u16 ) -> u16 { return MaxAllowedMaxMinRatio::<T>::get( netuid ) }

    /// ================================
	/// ==== Consensus term getters ====
	/// ================================
    pub fn get_subnetwork_n( netuid:u16 ) -> u16 { return N::<T>::get( netuid ) }
    pub fn get_rank( netuid:u16 ) -> Vec<u16> { return Rank::<T>::get( netuid ) }
    pub fn get_trust( netuid:u16 ) -> Vec<u16> { return Trust::<T>::get( netuid ) }
    pub fn get_incentive( netuid:u16 ) -> Vec<u16> { return Incentive::<T>::get( netuid ) }
    pub fn get_consensus( netuid:u16 ) -> Vec<u16> { return Consensus::<T>::get( netuid ) }
    pub fn get_dividends( netuid:u16 ) -> Vec<u16> { return Dividends::<T>::get( netuid ) }
    pub fn get_emission( netuid:u16 ) -> Vec<u64> { return Emission::<T>::get( netuid ) }

    /// =======================================
	/// ==== Consensus vector term getters ====
	/// =======================================
    pub fn get_rank_as_vector( netuid:u16 ) -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Rank::<T>::get( netuid ) ) }
    pub fn get_trust_as_vector( netuid:u16 ) -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Trust::<T>::get( netuid ) ) }
    pub fn get_conensus_as_vector( netuid:u16 ) -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Consensus::<T>::get( netuid ) ) }
    pub fn get_incentive_as_vector( netuid:u16 ) -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Incentive::<T>::get( netuid ) ) }
    pub fn get_dividends_as_vector( netuid:u16 ) -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Dividends::<T>::get( netuid ) ) }
    pub fn get_emission_as_vector( netuid:u16 ) -> nalgebra::DVector<u64> { nalgebra::DVector::from_vec( Emission::<T>::get( netuid ) ) }

    /// =============================================
	/// ==== Consensus float vector term getters ====
	/// =============================================
    pub fn get_rank_as_float_vector( netuid:u16 ) -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get( netuid ).iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
    pub fn get_trust_as_float_vector( netuid:u16 ) -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get( netuid ).iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
    pub fn get_consensus_as_float_vector( netuid:u16 ) -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get( netuid ).iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
    pub fn get_incentive_as_float_vector( netuid:u16 ) -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get( netuid ).iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
    pub fn get_dividends_as_float_vector( netuid:u16 ) -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get( netuid ).iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
    
    /// =================
	/// ==== Weights ====
	/// =================
    pub fn get_weights_as_matrix( netuid:u16 ) -> nalgebra::DMatrix<u16> { 
        let n = Self::get_subnetwork_n( netuid ) as usize;
        let mut weights: Vec<u16> = vec![ 0; n * n ];
        for ( uid_i, weights_i ) in < Weights<T> as IterableDoubleStorageMap<u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, weight_ij) in weights_i.iter() { weights [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *weight_ij;}
        }
        nalgebra::DMatrix::from_vec(n, n, weights ) 
    } 
    pub fn get_weights_as_float_matrix( netuid:u16 ) -> nalgebra::DMatrix<f32> { 
        let n = Self::get_subnetwork_n( netuid ) as usize;
        let mut weights: Vec<f32> = vec![ 0.0; n * n ];
        for ( uid_i, weights_i ) in < Weights<T> as IterableDoubleStorageMap<u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, weight_ij) in weights_i.iter() { weights [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *weight_ij as f32 / u16::MAX as f32;}
        }
        nalgebra::DMatrix::from_vec(n, n, weights ) 
    } 

    /// ===============
	/// ==== Bonds ====
	/// ===============
    pub fn get_bonds_as_matrix( netuid:u16 ) -> nalgebra::DMatrix<u16> { 
        let n = Self::get_subnetwork_n( netuid ) as usize;
        let mut bonds: Vec<u16> = vec![ 0; n * n ];
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableDoubleStorageMap<u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, bonds_ij) in bonds_i.iter() { bonds [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *bonds_ij;}
        }
        nalgebra::DMatrix::from_vec(n, n, bonds ) 
    } 
    pub fn get_bonds_as_float_matrix( netuid:u16 ) -> nalgebra::DMatrix<f32> { 
        let n = Self::get_subnetwork_n( netuid ) as usize;
        let mut bonds: Vec<f32> = vec![ 0.0; n * n ];
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableDoubleStorageMap<u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, bonds_ij) in bonds_i.iter() { bonds [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *bonds_ij as f32 / u16::MAX as f32;}
        }
        nalgebra::DMatrix::from_vec(n, n, bonds ) 
    } 

    /// =====================================
	/// ====  Consensus term uid getters ====
	/// =====================================
    pub fn get_rank_for_uid( netuid:u16, uid: u16 ) -> u16 { return Rank::<T>::get( netuid )[uid as usize] }
    pub fn get_trust_for_uid( netuid:u16, uid: u16 ) -> u16 { return Trust::<T>::get( netuid )[uid as usize] }
    pub fn get_incentive_for_uid( netuid:u16, uid: u16 ) -> u16 { return Incentive::<T>::get( netuid )[uid as usize] }
    pub fn get_consensus_for_uid( netuid:u16, uid: u16 ) -> u16 { return Consensus::<T>::get( netuid )[uid as usize] }
    pub fn get_dividends_for_uid( netuid:u16, uid: u16 ) -> u16 { return Dividends::<T>::get( netuid )[uid as usize] }
    pub fn get_emission_for_uid( netuid:u16, uid: u16 ) -> u64 { return Emission::<T>::get( netuid )[uid as usize] }

    /// ================================
	/// ==== Consensus term setters ====
	/// ================================
    pub fn set_rank( netuid:u16, rank: Vec<u16> ) { return Rank::<T>::put( netuid, rank ) }
    pub fn set_trust( netuid:u16, trust: Vec<u16> ) { return Trust::<T>::put( netuid, trust ) }
    pub fn set_incentive( netuid:u16, incentive: Vec<u16> ) { return Incentive::<T>::put( netuid, incentive ) }
    pub fn set_consensus( netuid:u16, consensus: Vec<u16> ) { return Consensus::<T>::put( netuid, consensus ) }
    pub fn set_dividends( netuid:u16, dividends: Vec<u16> ) { return Dividends::<T>::put( netuid, dividends ) }
    pub fn set_emission( netuid:u16, emission: Vec<u64> ) { return Emission::<T>::put( netuid, emission ) }

    /// ====================================
	/// ==== Consensus term uid setters ====
	/// ====================================
    pub fn set_rank_for_uid( netuid:u16, uid: u16, rank: u16 ) { 
        let mut vec: Vec<u16> = Self::get_rank( netuid ); 
        vec[uid as usize] = rank;
        Rank::<T>::put( vec ) 
    }
    pub fn set_trust_for_uid( netuid:u16, uid: u16, trust: u16 ) { 
        let mut vec: Vec<u16> = Self::get_trust( netuid ); 
        vec[uid as usize] = trust;
        Trust::<T>::put( vec ) 
    }
    pub fn set_incentive_for_uid( netuid:u16, uid: u16, incentive: u16 ) { 
        let mut vec: Vec<u16> = Self::get_incentive( netuid ); 
        vec[uid as usize] = incentive;
        Incentive::<T>::put( vec ) 
    }
    pub fn set_consensus_for_uid( netuid:u16, uid: u16, consensus: u16 ) { 
        let mut vec: Vec<u16> = Self::get_consensus( netuid ); 
        vec[uid as usize] = consensus;
        Consensus::<T>::put( vec ) 
    }
    pub fn set_dividends_for_uid( netuid:u16, uid: u16, dividends: u16 ) { 
        let mut vec: Vec<u16> = Self::get_dividends( netuid ); 
        vec[uid as usize] = dividends;
        Dividends::<T>::put( vec ) 
    }
    pub fn set_emission_for_uid( netuid:u16, uid: u16, emission: u64 ) { 
        let mut vec: Vec<u64> = Self::get_emission( netuid ); 
        vec[uid as usize] = emission;
        Emission::<T>::put( vec ) 
    }


}
