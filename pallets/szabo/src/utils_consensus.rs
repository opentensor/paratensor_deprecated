
use super::*;
use frame_support::inherent::Vec;
use frame_support::sp_std::vec;
use substrate_fixed::types::I65F63;
use frame_support::storage::IterableStorageDoubleMap;

#[cfg(feature = "no_std")]
extern crate nalgebra;

impl<T: Config> Pallet<T> {
    /// =====================
	/// ==== Hyperparams ====
	/// =====================
    pub fn get_min_allowed_weights( netuid:u16 ) -> u16 { return MinAllowedWeights::<T>::get( netuid ) }
    pub fn get_max_allowed_max_min_ratio( netuid:u16 ) -> u16 { return MaxAllowedMaxMinRatio::<T>::get( netuid ) }

    /// ================================
	/// ==== Consensus term getters ====
	/// ================================
    pub fn get_subnetwork_n( netuid:u16 ) -> u16 { return SubnetworkN::<T>::get( netuid ) }
    pub fn get_rank( netuid:u16 ) -> Vec<u16> { return Rank::<T>::get( netuid ) }
    pub fn get_trust( netuid:u16 ) -> Vec<u16> { return Trust::<T>::get( netuid ) }
    pub fn get_incentive( netuid:u16 ) -> Vec<u16> { return Incentive::<T>::get( netuid ) }
    pub fn get_consensus( netuid:u16 ) -> Vec<u16> { return Consensus::<T>::get( netuid ) }
    pub fn get_dividends( netuid:u16 ) -> Vec<u16> { return Dividends::<T>::get( netuid ) }
    pub fn get_emission( netuid:u16 ) -> Vec<u64> { return Emission::<T>::get( netuid ) }

    /// =======================================
	/// ==== Consensus vector term getters ====
	/// =======================================
    pub fn get_rank_as_vector( netuid:u16 ) -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Self::get_rank( netuid ) ) }
    pub fn get_trust_as_vector( netuid:u16 ) -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Self::get_trust( netuid ) ) }
    pub fn get_conensus_as_vector( netuid:u16 ) -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Self::get_consensus( netuid ) ) }
    pub fn get_incentive_as_vector( netuid:u16 ) -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Self::get_dividends( netuid ) ) }
    pub fn get_dividends_as_vector( netuid:u16 ) -> nalgebra::DVector<u16> { nalgebra::DVector::from_vec( Self::get_dividends( netuid ) ) }
    pub fn get_emission_as_vector( netuid:u16 ) -> nalgebra::DVector<u64> { nalgebra::DVector::from_vec( Self::get_emission( netuid ) ) }

    /// =============================================
	/// ==== Consensus float vector term getters ====
	/// =============================================
    pub fn get_rank_as_float_vector( netuid:u16 ) -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get( netuid ).iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
    pub fn get_trust_as_float_vector( netuid:u16 ) -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get( netuid ).iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
    pub fn get_consensus_as_float_vector( netuid:u16 ) -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get( netuid ).iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
    pub fn get_incentive_as_float_vector( netuid:u16 ) -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get( netuid ).iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
    pub fn get_dividends_as_float_vector( netuid:u16 ) -> nalgebra::DVector<f32> { nalgebra::DVector::from_vec( Rank::<T>::get( netuid ).iter().map(|&e| (e as f32 / u16::MAX as f32) ).collect() ) }
   
    /// =============================================
	/// ==== Consensus term float vector setters ====
	/// =============================================
    pub fn set_rank_from_vector( netuid:u16, rank: nalgebra::DVector<f32> ) { let vec: Vec<u16> = rank.into_iter().cloned().map(|e| (e * u16::MAX as f32) as u16 ).collect(); return Rank::<T>::insert( netuid, vec ) }
    pub fn set_trust_from_vector( netuid:u16, trust: nalgebra::DVector<f32> ) { let vec: Vec<u16> = trust.into_iter().cloned().map(|e| (e * u16::MAX as f32) as u16 ).collect(); return Trust::<T>::insert( netuid, vec )  }
    pub fn set_incentive_from_vector( netuid:u16, incentive: nalgebra::DVector<f32> ) { let vec: Vec<u16> = incentive.into_iter().cloned().map(|e| (e * u16::MAX as f32) as u16 ).collect(); return Incentive::<T>::insert( netuid, vec )  }
    pub fn set_consensus_from_vector( netuid:u16, consensus: nalgebra::DVector<f32> ) { let vec: Vec<u16> = consensus.into_iter().cloned().map(|e| (e * u16::MAX as f32) as u16 ).collect(); return Consensus::<T>::insert( netuid, vec ) }
    pub fn set_dividends_from_vector( netuid:u16, dividends: nalgebra::DVector<f32> ) { let vec: Vec<u16> = dividends.into_iter().cloned().map(|e| (e * u16::MAX as f32) as u16 ).collect(); return Dividends::<T>::insert( netuid, vec ) }
    pub fn set_emission_from_vector( netuid:u16, emission: nalgebra::DVector<u64> ) { let vec: Vec<u64> = emission.into_iter().cloned().collect(); return Emission::<T>::insert( netuid, vec )  }

    /// ===============
	/// ==== Stake ====
	/// ===============
    pub fn get_stake( netuid:u16 ) -> Vec<u64> { 
        let n = Self::get_subnetwork_n( netuid ) as usize;
        let mut stake: Vec<u64> = vec![ 0; n ];
        for ( uid_i, hotkey_i ) in <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix( netuid ){ 
            stake [ uid_i as usize ] = Stake::<T>::get( hotkey_i );
        }
        return stake
    }
    pub fn get_stake_as_vector( netuid:u16 ) -> nalgebra::DVector<u64> { nalgebra::DVector::from_vec( Self::get_stake( netuid ) ) }
    pub fn get_stake_as_float_vector( netuid:u16 ) -> nalgebra::DVector<f32> { 
        let n = Self::get_subnetwork_n( netuid ) as usize;
        let subnetwork_stake: Vec<u64> = Self::get_stake( netuid );
        let subnetwork_stake_sum: I65F63 = I65F63::from_num( subnetwork_stake.iter().sum::<u64>() );
        let mut normalized_stake_vector: Vec<f32> = vec![ 0.0; n ];
        for (uid_i, stake_i) in Self::get_stake( netuid ).iter().enumerate() {
            let stake_fraction_i: I65F63 = I65F63::from_num( *stake_i ) / subnetwork_stake_sum;
            normalized_stake_vector[ uid_i ] = stake_fraction_i.to_num::<f32>();
        }
        return nalgebra::DVector::from_vec( normalized_stake_vector );
    }

    /// =================
	/// ==== Weights ====
	/// =================
    pub fn set_weights_for_uid( netuid:u16, uid:u16, weights: Vec<f32> ) { 
        let mut zipped_weights: Vec<(u16,u16)> = vec![];
        for (i, fw) in weights.iter().enumerate() {
            let wij: u16 = (*fw * (u16::MAX as f32)) as u16;
            if wij != 0 { zipped_weights.push((i as u16, wij)) }
        }
        Weights::<T>::insert( netuid, uid, zipped_weights );
    }
    pub fn set_weights_from_float_matrix( netuid:u16, weights: nalgebra::DMatrix<f32> ) { 
        for i in 0..weights.nrows() { 
            Self::set_weights_for_uid( netuid, i as u16, weights.row(i).into_iter().cloned().collect() );
        }
    }
    pub fn get_weights_as_matrix( netuid:u16 ) -> nalgebra::DMatrix<u16> { 
        let n = Self::get_subnetwork_n( netuid ) as usize;
        let mut weights: Vec<u16> = vec![ 0; n * n ];
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, weight_ij) in weights_i.iter() { weights [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *weight_ij;}
        }
        nalgebra::DMatrix::from_vec(n, n, weights ) 
    } 
    pub fn get_weights_as_float_matrix( netuid:u16 ) -> nalgebra::DMatrix<f32> { 
        let n = Self::get_subnetwork_n( netuid ) as usize;
        let mut weights: Vec<f32> = vec![ 0.0; n * n ];
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
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
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, bonds_ij) in bonds_i.iter() { bonds [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *bonds_ij;}
        }
        nalgebra::DMatrix::from_vec(n, n, bonds ) 
    } 
    pub fn get_bonds_as_float_matrix( netuid:u16 ) -> nalgebra::DMatrix<f32> { 
        let n = Self::get_subnetwork_n( netuid ) as usize;
        let mut bonds: Vec<f32> = vec![ 0.0; n * n ];
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
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
    pub fn get_stake_for_uid( netuid:u16, uid: u16 ) -> u64 { return Stake::<T>::get( &Keys::<T>::get( netuid, uid ) ) }


    /// ====================================
	/// ==== Consensus term uid setters ====
	/// ====================================
    pub fn set_rank_for_uid( netuid:u16, uid: u16, rank: u16 ) { 
        let mut vec: Vec<u16> = Self::get_rank( netuid ); 
        vec[uid as usize] = rank;
        Rank::<T>::insert( netuid, vec ) 
    }
    pub fn set_trust_for_uid( netuid:u16, uid: u16, trust: u16 ) { 
        let mut vec: Vec<u16> = Self::get_trust( netuid ); 
        vec[uid as usize] = trust;
        Trust::<T>::insert( netuid, vec ) 
    }
    pub fn set_incentive_for_uid( netuid:u16, uid: u16, incentive: u16 ) { 
        let mut vec: Vec<u16> = Self::get_incentive( netuid ); 
        vec[uid as usize] = incentive;
        Incentive::<T>::insert( netuid, vec ) 
    }
    pub fn set_consensus_for_uid( netuid:u16, uid: u16, consensus: u16 ) { 
        let mut vec: Vec<u16> = Self::get_consensus( netuid ); 
        vec[uid as usize] = consensus;
        Consensus::<T>::insert( netuid, vec ) 
    }
    pub fn set_dividends_for_uid( netuid:u16, uid: u16, dividends: u16 ) { 
        let mut vec: Vec<u16> = Self::get_dividends( netuid ); 
        vec[uid as usize] = dividends;
        Dividends::<T>::insert( netuid, vec ) 
    }
    pub fn set_emission_for_uid( netuid:u16, uid: u16, emission: u64 ) { 
        let mut vec: Vec<u64> = Self::get_emission( netuid ); 
        vec[uid as usize] = emission;
        Emission::<T>::insert( netuid, vec ) 
    }


}
