
use super::*;
use frame_support::inherent::Vec;
use frame_support::sp_std::vec;
use substrate_fixed::types::I65F63;
use frame_support::storage::IterableStorageDoubleMap;

#[cfg(feature = "no_std")]
use ndarray::{ndarray::Array1, ndarray::Array2, ndarray::arr1};

impl<T: Config> Pallet<T> {

    /// ====================
	/// ==== Common u16 ====
	/// ====================
    pub fn fixed_to_u16( x: I65F63 ) -> u16 { x.to_num::<u16>(); }
    pub fn u16_to_fixed( x: u16 ) -> I65F63 { I65F63::from_num( x ) }
    pub fn u16_proportion_to_fixed( x: u16 ) -> I65F63 { I65F63::from_num( x ) / I65F63::from_num( u16::MAX ) }
    pub fn fixed_proportion_to_u16( x: I65F63 ) -> u16 { Self::fixed_to_u16( x * I65F63::from_num( u16::MAX )) }
    pub fn vec_fixed_to_u16( vec: Vec<I65F63> ) -> Vec<u16> { vec.into_iter().map(|e| Self::fixed_to_u16(e) ).collect(); }
    pub fn vec_u16_to_fixed( vec: Vec<u16> ) -> Vec<I65F63> { vec.into_iter().map( |e| Self::u16_to_fixed(e) ).collect(); }
    pub fn vec_u16_proportions_to_fixed( vec: Vec<u16> ) -> Vec<I65F63> { vec.into_iter().map(|e| Self::u16_proportion_to_fixed(e) ).collect(); }
    pub fn vec_fixed_proportions_to_u16( vec: Vec<I65F63> ) -> Vec<u64> { vec.into_iter().map(|e| Self::fixed_proportion_to_u16(e) ).collect(); }
    pub fn sparse_vec_u16_proportions_to_sparse_fixed( vec: &Vec<(u16, u16)> ) -> Vec<(u16, I65F63)> {
        vec.into_iter().map(|  (coli, ai) | (coli, Self::u16_proportion_to_fixed(e) )  ).collect().sort_by( |a, b| a.0.cmp(&b.0) )
    }

    /// ====================
	/// ==== Common u64 ====
	/// ====================
    pub fn fixed_to_u64( x: I65F63 ) -> u64 { x.to_num::<u64>(); }
    pub fn u64_to_fixed( x: u64 ) -> I65F63 { I65F63::from_num( x ) }
    pub fn u64_to_fixed_proportion( x: u64 ) -> I65F63 { I65F63::from_num( x ) / I65F63::from_num( u64::MAX ) }
    pub fn fixed_proportion_to_u64( x: I65F63 ) -> u64 { Self::fixed_to_u64(x * I65F63::from_num( u64::MAX )) }
    pub fn vec_fixed_to_u64( vec: Vec<I65F63> ) -> Vec<u16> { vec.into_iter().map(|e| Self::fixed_to_u64(e) ).collect(); }
    pub fn vec_u64_to_fixed( vec: Vec<u64> ) -> Vec<I65F63> { vec.into_iter().map( |e| Self::u64_to_fixed(e) ).collect(); }
    pub fn vec_u64_proportions_to_fixed( vec: Vec<u64> ) -> Vec<I65F63> { vec.into_iter().map(|e| Self::u64_to_fixed_proportion(e) ).collect(); }
    pub fn vec_fixed_proportions_to_u64( vec: Vec<I65F63> ) -> Vec<u16> { vec.into_iter().map(|e| Self::fixed_proportion_to_u64(e) ).collect(); }

    /// =====================
	/// ==== Hyperparams ====
	/// =====================
    pub fn get_rho( netuid: u16 ) -> I65F63 { Self::u16_to_fixed_proportion( Rho::<T>::get( netuid ) )}
    pub fn get_kappa( netuid: u16 ) -> I65F63 { Self::u16_to_fixed_proportion( Kappa::<T>::get( netuid ) )}
    pub fn get_trust_threshold( netuid: u16 ) -> I65F63 { Self::u16_to_fixed_proportion( TrustThreshold::<T>::get( netuid ) )}

    /// ===========================
	/// ==== Consensus Getters ====
	/// ===========================
    pub fn get_ranks( netuid:u16 ) -> Vec<I65F63> {  Self::u16_proportions_to_fixed( Rank::<T>::get( netuid ) ) }
    pub fn get_trust( netuid:u16 ) -> Vec<I65F63> { Self::u16_proportions_to_fixed( Trust::<T>::get( netuid ) ) }
    pub fn get_consensus( netuid:u16 ) -> Vec<I65F63> { Self::u16_proportions_to_fixed( Consensus::<T>::get( netuid ) ) }
    pub fn get_incentive( netuid:u16 ) -> Vec<I65F63> { Self::u16_proportions_to_fixed( Incentive::<T>::get( netuid ) )  }
    pub fn get_dividends( netuid:u16 ) -> Vec<I65F63> { Self::u16_proportions_to_fixed( Dividends::<T>::get( netuid ) )  }
    pub fn get_emission( netuid:u16 ) -> Vec<I65F63> { Self::u64s_to_fixed( Emission::<T>::get( netuid ) )  }
    pub fn get_stake( netuid:u16 ) -> Vec<I65F63> { 
        let mut stake: Vec<u64> = vec![ 0; Self::get_subnetwork_n( netuid ) as usize ];
        for ( uid_i, hotkey_i ) in <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix( netuid ){ 
            stake [ uid_i as usize ] = Stake::<T>::get( hotkey_i );
        }
        Self::u64s_to_fixed( stake );
    }
    pub fn get_weights( netuid:u16 ) -> Vec<Vec<(u16, I65F63)>> { 
        let mut weights: Vec<u16> = vec![];
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, weight_ij) in weights_i.iter() { weights [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *weight_ij;}
        }
        return ndarray::Array2::from_shape_vec( (n, n), weights ).unwrap();
    } 
   
    /// ===========================
	/// ==== Consensus Setters ====
	/// ===========================
    pub fn set_rank( netuid:u16, rank: Vec<I65F63> ) { return Rank::<T>::insert( netuid, Self::vec_u16_proportions_to_fixed( rank )  ) }
    pub fn set_trust( netuid:u16, trust: Vec<I65F63>) { return Trust::<T>::insert( netuid, Self::vec_u16_proportions_to_fixed( trust )  )  }
    pub fn set_incentive( netuid:u16, incentive: Vec<I65F63>) { return Incentive::<T>::insert( netuid, Self::vec_u16_proportions_to_fixed( incentive )  )  }
    pub fn set_consensus( netuid:u16, consensus: Vec<I65F63> ) { return Consensus::<T>::insert( netuid, Self::vec_u16_proportions_to_fixed( consensus )  ) }
    pub fn set_dividends( netuid:u16, dividends: Vec<I65F63> ) { return Dividends::<T>::insert( netuid, Self::vec_u16_proportions_to_fixed( dividends ) ) }
    pub fn set_emission( netuid:u16, emission: Vec<I65F63> ) { return Emission::<T>::insert( netuid, Self::vec_u64_to_fixed( emission ) )  }

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
    pub fn set_weights_from_float_matrix( netuid:u16, weights: ndarray::Array2<f32> ) { 
        for i in 0..weights.nrows() { 
            Self::set_weights_for_uid( netuid, i as u16, weights.row(i).into_iter().cloned().collect() );
        }
    }
    pub fn get_weights_as_matrix( netuid:u16 ) -> ndarray::Array2<u16> { 
        let n = Self::get_subnetwork_n( netuid ) as usize;
        let mut weights: Vec<u16> = vec![ 0; n * n ];
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, weight_ij) in weights_i.iter() { weights [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *weight_ij;}
        }
        return ndarray::Array2::from_shape_vec( (n, n), weights ).unwrap();
    } 
    pub fn get_weights_as_float_matrix( netuid:u16 ) -> ndarray::Array2<f32> { 
        let n = Self::get_subnetwork_n( netuid ) as usize;
        let mut weights: Vec<f32> = vec![ 0.0; n * n ];
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, weight_ij) in weights_i.iter() { weights [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *weight_ij as f32 / u16::MAX as f32;}
        }
        return ndarray::Array2::from_shape_vec( (n, n), weights ).unwrap();
    } 

    /// ===============
	/// ==== Bonds ====
	/// ===============
    pub fn set_bonds_for_uid( netuid:u16, uid:u16, bonds: Vec<f32> ) { 
        let mut zipped_bonds: Vec<(u8,u8)> = vec![];
        for (i, fw) in bonds.iter().enumerate() {
            let bij: u8 = (*fw * (u8::MAX as f32)) as u8;
            if bij != 0 { zipped_bonds.push((i as u8, bij)) }
        }
        Bonds::<T>::insert( netuid, uid, zipped_bonds );
    }
    pub fn set_bonds_from_float_matrix( netuid:u16, bonds: ndarray::Array2<f32> ) { 
        for i in 0..bonds.nrows() { 
            Self::set_bonds_for_uid( netuid, i as u16, bonds.row(i).into_iter().cloned().collect() );
        }
    }
    pub fn get_bonds_as_matrix( netuid:u16 ) -> ndarray::Array2<u8> { 
        let n = Self::get_subnetwork_n( netuid ) as usize;
        let mut bonds: Vec<u8> = vec![ 0; n * n ];
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u8, u8)> >>::iter_prefix( netuid ) {
            for (uid_j, bonds_ij) in bonds_i.iter() { bonds [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *bonds_ij;}
        }
        return ndarray::Array2::from_shape_vec( (n, n), bonds ).unwrap();
    } 
    pub fn get_bonds_as_float_matrix( netuid:u16 ) -> ndarray::Array2<f32> { 
        let n = Self::get_subnetwork_n( netuid ) as usize;
        let mut bonds: Vec<f32> = vec![ 0.0; n * n ];
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u8, u8)> >>::iter_prefix( netuid ) {
            for (uid_j, bonds_ij) in bonds_i.iter() { bonds [ ( n * uid_i as usize ) + (*uid_j as usize) ] = *bonds_ij as f32 / u8::MAX as f32;}
        }
        return ndarray::Array2::from_shape_vec( (n, n), bonds ).unwrap();
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
