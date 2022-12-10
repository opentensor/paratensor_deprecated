use super::*;
use sp_runtime::sp_std::if_std;
use frame_support::sp_std::vec;
use frame_support::inherent::Vec;
use substrate_fixed::transcendental::exp;
use substrate_fixed::types::I32F32;
use frame_support::storage::IterableStorageDoubleMap;

impl<T: Config> Pallet<T> {
    pub fn epoch( netuid: u16, rao_emission: u64, debug: bool ) -> Vec<I32F32> {
  
        // Get subnetwork size.
        let n: u16 = Self::get_subnetwork_n( netuid );
        if debug { if_std! { println!( "n:\n{:?}\n", n );}}

        // Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        if debug { if_std! { println!( "current_block:\n{:?}\n", current_block );}}

        // Get activity cutoff.
        let activity_cutoff: u64 = Self::get_activity_cutoff( netuid ) as u64;
        if debug { if_std! { println!( "activity_cutoff:\n{:?}\n", activity_cutoff );}}

        // Last update vector.
        let last_update: Vec<u64> = Self::get_last_update( netuid );
        if debug { if_std! { println!( "Last update:\n{:?}\n", last_update.clone() );}}

        // Active mask.
        let active: Vec<bool> = last_update.iter().map(| updated | current_block <= *updated + activity_cutoff ).collect();
        if debug { if_std! { println!( "Active:\n{:?}\n", active.clone() );}}

        // Access network stake as normalized vector.
        let mut stake: Vec<I32F32> = Self::get_stake( netuid );
        inplace_mask_vector( &active, &mut stake );
        inplace_normalize( &mut stake );
        if debug { if_std! { println!( "S:\n{:?}\n", stake.clone() );}}

        // Block at registration vector (block when each neuron was most recently registered).
        let block_at_registration: Vec<u64> = Self::get_block_at_registration( netuid );
        if debug { if_std! { println!( "Block at registration:\n{:?}\n", block_at_registration.clone() );}}

        // Updated matrix, updated_ij=True if i has last updated (weights) after j has last registered.
        let updated: Vec<Vec<bool>> = block_at_registration.iter().map(| registered | last_update.iter().map(| updated | registered < updated ).collect() ).collect();

        // Access network weights row normalized.
        let mut weights: Vec<Vec<I32F32>> = Self::get_weights( netuid );
        inplace_diag_mask( &mut weights ); // remove self-weight by masking diagonal
        inplace_mask_matrix( &updated, &mut weights ); // remove weights referring to deregistered neurons
        inplace_row_normalize( &mut weights );
        if debug { if_std! { println!( "W:\n{:?}\n", weights.clone() );}}

        // Compute ranks: r_j = SUM(i) w_ij * s_i
        let mut ranks: Vec<I32F32> = matmul( &weights, &stake );
        inplace_normalize( &mut ranks );
        if debug { if_std! { println!( "R:\n{:?}\n", ranks.clone() );}}

        // Compute thresholded weights.
        let upper: I32F32 = I32F32::from_num( 1.0 );
        let lower: I32F32 = I32F32::from_num( 0.0 );
        let threshold: I32F32 = I32F32::from_num( 0.01 );
        let clipped_weights: Vec<Vec<I32F32>> = clip( &weights, threshold, upper, lower );
        if debug { if_std! { println!( "tW:\n{:?}\n", clipped_weights.clone() );}}

        // Compute trust scores: t_j = SUM(i) w_ij * s_i
        let trust: Vec<I32F32> = matmul( &clipped_weights, &stake );
        if debug { if_std! { println!( "T:\n{:?}\n", trust.clone() );}}

        // Compute consensus.
        let one: I32F32 = I32F32::from_num(1.0); 
        let rho: I32F32 = Self::get_float_rho( netuid );
        let kappa: I32F32 = Self::get_float_kappa( netuid );
        let exp_trust: Vec<I32F32> = trust.iter().map( |t|  exp( -rho * (t - kappa) ).expect("") ).collect();
        let consensus: Vec<I32F32> = exp_trust.iter().map( |t|  one /(one + t) ).collect();
        if debug { if_std! { println!( "C:\n{:?}\n", consensus.clone() );}}

        // Compute incentive.
        let mut incentive: Vec<I32F32> = ranks.iter().zip( consensus.clone() ).map( |(ri, ci)| ri * ci ).collect();
        inplace_normalize( &mut incentive );
        if debug { if_std! { println!( "I:\n{:?}\n", incentive.clone() );}}

        // Access network bonds column normalized.
        let mut bonds: Vec<Vec<I32F32>> = Self::get_bonds( netuid );
        inplace_mask_matrix( &updated, &mut bonds );
        inplace_col_normalize( &mut bonds ); // sum_i b_ij = 1
        if debug { if_std! { println!( "B:\n{:?}\n", bonds.clone() );}}        

        // Compute bonds delta column normalized.
        let mut bonds_delta: Vec<Vec<I32F32>> = hadamard( &weights, &stake ); // ΔB = W◦S
        inplace_mask_matrix( &updated, &mut bonds_delta );
        inplace_col_normalize( &mut bonds_delta ); // sum_i b_ij = 1
        if debug { if_std! { println!( "ΔB:\n{:?}\n", bonds_delta.clone() );}}
    
        // Compute bonds moving average.
        let alpha: I32F32 = I32F32::from_num( 0.1 );
        let mut ema_bonds: Vec<Vec<I32F32>> = mat_ema( &bonds_delta, &bonds, alpha );
        inplace_col_normalize( &mut ema_bonds ); // sum_i b_ij = 1
        if debug { if_std! { println!( "emaB:\n{:?}\n", ema_bonds.clone() );}}

        // Compute dividends: d_i = SUM(j) b_ij * inc_j
        let dividends: Vec<I32F32> = matmul_transpose( &ema_bonds, &incentive );
        if debug { if_std! { println!( "D:\n{:?}\n", dividends.clone() );}}

        // Compute emission scores.
        let float_rao_emission: I32F32 = I32F32::from_num( rao_emission );
        let mut normalized_emission: Vec<I32F32> = incentive.iter().zip( dividends.clone() ).map( |(ii, di)| ii + di ).collect();
        inplace_normalize( &mut normalized_emission );
        let emission: Vec<I32F32> = normalized_emission.iter().map( |e| e * float_rao_emission ).collect();
        if debug { if_std! { println!( "E:\n{:?}\n", emission.clone() );}}

        // Compute pruning scores.
        let mut pruning: Vec<I32F32> = incentive.iter().zip( dividends.clone() ).map( |(ii, di)| ii + di ).collect();
        inplace_normalize( &mut pruning );
        if debug { if_std! { println!( "P:\n{:?}\n", pruning.clone() );}}

        // Sync parameter updates.
        for i in 0..n {
            Self::set_rank( netuid, i, fixed_proportion_to_u16( ranks[i as usize] ) );
            Self::set_trust( netuid, i, fixed_proportion_to_u16( trust[i as usize] ) );
            Self::set_consensus( netuid, i, fixed_proportion_to_u16( consensus[i as usize] ) );
            Self::set_incentive( netuid, i, fixed_proportion_to_u16( incentive[i as usize] ) );
            Self::set_dividend( netuid, i, fixed_proportion_to_u16( dividends[i as usize] ) );
            Self::set_pruning( netuid, i, fixed_proportion_to_u16( pruning[i as usize] ) );
            Self::set_emission( netuid, i, fixed_to_u64( emission[i as usize] ) );
            Self::set_bonds( netuid, i, (0..n).zip( vec_fixed_proportions_to_u16( ema_bonds[i as usize].clone() ) ).collect() );
        }  

        emission
    }

    pub fn epoch_sparse( netuid: u16, rao_emission: u64, debug: bool ) -> Vec<I32F32> {
        // Get subnetwork size.
        let n: u16 = Self::get_subnetwork_n( netuid );
        if debug { if_std! { println!( "n:\n{:?}\n", n );}}

        // Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        if debug { if_std! { println!( "current_block:\n{:?}\n", current_block );}}

        // Get activity cutoff.
        let activity_cutoff: u64 = Self::get_activity_cutoff( netuid ) as u64;
        if debug { if_std! { println!( "activity_cutoff:\n{:?}\n", activity_cutoff );}}

        // Last update vector.
        let last_update: Vec<u64> = Self::get_last_update( netuid );
        if debug { if_std! { println!( "Last update:\n{:?}\n", last_update.clone() );}}

        // Active mask.
        let active: Vec<bool> = last_update.iter().map(| updated | current_block <= *updated + activity_cutoff ).collect();
        if debug { if_std! { println!( "Active:\n{:?}\n", active.clone() );}}

        // Access network stake as normalized vector.
        let mut stake: Vec<I32F32> = Self::get_stake( netuid );
        inplace_mask_vector( &active, &mut stake );
        inplace_normalize( &mut stake );
        if debug { if_std! { println!( "S:\n{:?}\n", stake.clone() );}}

        // Block at registration vector (block when each neuron was most recently registered).
        let block_at_registration: Vec<u64> = Self::get_block_at_registration( netuid );
        if debug { if_std! { println!( "Block at registration:\n{:?}\n", block_at_registration.clone() );}}

        // Updated matrix, updated_ij=True if i has last updated (weights) after j has last registered.
        // let updated: Vec<Vec<bool>> = block_at_registration.iter().map(| registered | last_update.iter().map(| updated | registered < updated ).collect() ).collect();

        // Access network weights row normalized.
        let mut weights: Vec<Vec<(u16, I32F32)>> = Self::get_weights_sparse( netuid );
        diag_mask_sparse( &weights ); // remove self-weight by masking diagonal
        weights = vec_mask_sparse_matrix( &weights, &block_at_registration, &last_update, &| registered, updated | updated <= registered ); // remove weights referring to deregistered neurons
        inplace_row_normalize_sparse( &mut weights );
        if debug { if_std! { println!( "W:\n{:?}\n", weights.clone() );}}

        // Compute ranks: r_j = SUM(i) w_ij * s_i
        let mut ranks: Vec<I32F32> = sparse_matmul( &weights, &stake );
        inplace_normalize( &mut ranks );
        if debug { if_std! { println!( "R:\n{:?}\n", ranks.clone() );}}

        // Compute thresholded weights.
        let upper: I32F32 = I32F32::from_num( 1.0 );
        let lower: I32F32 = I32F32::from_num( 0.0 );
        let threshold: I32F32 = I32F32::from_num( 0.01 );
        let clipped_weights: Vec<Vec<(u16, I32F32)>> = sparse_clip( &weights, threshold, upper, lower );
        if debug { if_std! { println!( "tW:\n{:?}\n", clipped_weights.clone() );}}

        // Compute trust scores: t_j = SUM(i) w_ij * s_i
        let trust: Vec<I32F32> = sparse_matmul( &clipped_weights, &stake );
        if debug { if_std! { println!( "T:\n{:?}\n", trust.clone() );}}

        // Compute consensus.
        let one: I32F32 = I32F32::from_num(1.0); 
        let rho: I32F32 = Self::get_float_rho( netuid );
        let kappa: I32F32 = Self::get_float_kappa( netuid );
        let exp_trust: Vec<I32F32> = trust.iter().map( |t|  exp( -rho * (t - kappa) ).expect("") ).collect();
        let consensus: Vec<I32F32> = exp_trust.iter().map( |t|  one /(one + t) ).collect();
        if debug { if_std! { println!( "C:\n{:?}\n", consensus.clone() );}}

        // Compute incentive.
        let mut incentive: Vec<I32F32> = ranks.iter().zip( consensus.clone() ).map( |(ri, ci)| ri * ci ).collect();
        inplace_normalize( &mut incentive );
        if debug { if_std! { println!( "I:\n{:?}\n", incentive.clone() );}}

        // Access network bonds column normalized.
        let mut bonds: Vec<Vec<(u16, I32F32)>> = Self::get_bonds_sparse( netuid );
        bonds = vec_mask_sparse_matrix( &bonds, &block_at_registration, &last_update, &| registered, updated | updated <= registered ); // remove bonds referring to deregistered neurons
        inplace_col_normalize_sparse( &mut bonds ); // sum_i b_ij = 1
        if debug { if_std! { println!( "B:\n{:?}\n", bonds.clone() );}}        

        // Compute bonds delta column normalized.
        let mut bonds_delta: Vec<Vec<(u16, I32F32)>> = sparse_hadamard( &weights, &stake ); // ΔB = W◦S
        bonds_delta = vec_mask_sparse_matrix( &bonds_delta, &block_at_registration, &last_update, &| registered, updated | updated <= registered ); // remove bonds referring to deregistered neurons
        inplace_col_normalize_sparse( &mut bonds_delta ); // sum_i b_ij = 1
        if debug { if_std! { println!( "ΔB:\n{:?}\n", bonds_delta.clone() );}}
    
        // Compute bonds moving average.
        let alpha: I32F32 = I32F32::from_num( 0.1 );
        let mut ema_bonds: Vec<Vec<(u16, I32F32)>> = sparse_mat_ema( &bonds_delta, &bonds, alpha );
        inplace_col_normalize_sparse( &mut ema_bonds ); // sum_i b_ij = 1
        if debug { if_std! { println!( "emaB:\n{:?}\n", ema_bonds.clone() );}}

        // Compute dividends: d_i = SUM(j) b_ij * inc_j
        let dividends: Vec<I32F32> = sparse_matmul_transpose( &ema_bonds, &incentive );
        if debug { if_std! { println!( "D:\n{:?}\n", dividends.clone() );}}

        // Compute emission scores.
        let float_rao_emission: I32F32 = I32F32::from_num( rao_emission );
        let mut normalized_emission: Vec<I32F32> = incentive.iter().zip( dividends.clone() ).map( |(ii, di)| ii + di ).collect();
        inplace_normalize( &mut normalized_emission );
        let emission: Vec<I32F32> = normalized_emission.iter().map( |e| e * float_rao_emission ).collect();
        if debug { if_std! { println!( "E:\n{:?}\n", emission.clone() );}}

        // Compute pruning scores.
        let mut pruning: Vec<I32F32> = incentive.iter().zip( dividends.clone() ).map( |(ii, di)| ii + di ).collect();
        inplace_normalize( &mut pruning );
        if debug { if_std! { println!( "P:\n{:?}\n", pruning.clone() );}}

        // Sync parameter updates.
        for i in 0..n {
            Self::set_rank( netuid, i, fixed_proportion_to_u16( ranks[i as usize] ) );
            Self::set_trust( netuid, i, fixed_proportion_to_u16( trust[i as usize] ) );
            Self::set_consensus( netuid, i, fixed_proportion_to_u16( consensus[i as usize] ) );
            Self::set_incentive( netuid, i, fixed_proportion_to_u16( incentive[i as usize] ) );
            Self::set_dividend( netuid, i, fixed_proportion_to_u16( dividends[i as usize] ) );
            Self::set_pruning( netuid, i, fixed_proportion_to_u16( pruning[i as usize] ) );
            Self::set_emission( netuid, i, fixed_to_u64( emission[i as usize] ) );
            Self::set_bonds( netuid, i, ema_bonds[i as usize].iter().map( |(j, value)| (*j, fixed_proportion_to_u16(*value))).collect())
        }  

        emission
    }

    // Testing function.
    pub fn set_stake_for_testing( hotkey: &T::AccountId, stake:u64 ) { 
        Stake::<T>::insert( hotkey, stake );
    }

    pub fn set_rank( netuid:u16, neuron_uid: u16, rank:u16 ) { Rank::<T>::insert( netuid, neuron_uid, rank) }
    pub fn set_trust( netuid:u16, neuron_uid:u16, trust:u16) { Trust::<T>::insert( netuid, neuron_uid, trust ) }
    pub fn set_consensus( netuid:u16, neuron_uid:u16, consensus:u16) { Consensus::<T>::insert( netuid, neuron_uid, consensus ) }
    pub fn set_incentive( netuid:u16, neuron_uid:u16, incentive:u16) { Incentive::<T>::insert( netuid, neuron_uid, incentive ) }
    pub fn set_dividend( netuid:u16, neuron_uid:u16, dividend:u16) { Dividends::<T>::insert( netuid, neuron_uid, dividend ) }
    pub fn set_pruning( netuid:u16, neuron_uid:u16, pruning:u16) { PruningScores::<T>::insert( netuid, neuron_uid, pruning ) }
    pub fn set_emission( netuid:u16, neuron_uid:u16, emission:u64) { Emission::<T>::insert( netuid, neuron_uid, emission ) }
    pub fn set_bonds( netuid:u16, neuron_uid:u16, bonds:Vec<(u16,u16)>) { Bonds::<T>::insert( netuid, neuron_uid, bonds ) }

    pub fn get_float_rho( netuid:u16 ) -> I32F32 { I32F32::from_num( Self::get_rho( netuid ) )  }
    pub fn get_float_kappa( netuid:u16 ) -> I32F32 { I32F32::from_num( Self::get_kappa( netuid )  ) / I32F32::from_num( u16::MAX ) }
    pub fn get_rank( netuid:u16, neuron_uid: u16) -> u16 {  Rank::<T>::get( netuid,  neuron_uid) }
    pub fn get_trust( netuid:u16, neuron_uid: u16 ) -> u16 { Trust::<T>::get( netuid, neuron_uid )  }
    pub fn get_consensus( netuid:u16, neuron_uid: u16 ) -> u16 { Consensus::<T>::get( netuid, neuron_uid )  }
    pub fn get_incentive( netuid:u16, neuron_uid: u16 ) -> u16 { Incentive::<T>::get( netuid, neuron_uid )   }
    pub fn get_dividend( netuid:u16, neuron_uid: u16 ) -> u16 { Dividends::<T>::get( netuid, neuron_uid )  }
    pub fn get_emission( netuid:u16, neuron_uid: u16 ) -> u64 { Emission::<T>::get( netuid, neuron_uid )  }

    pub fn get_stake( netuid:u16 ) -> Vec<I32F32> {
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut stake: Vec<I32F32> = vec![  I32F32::from_num(0.0); n ]; 
        for neuron_uid in 0..n {
            if Keys::<T>::contains_key( netuid, neuron_uid as u16 ){
                let hotkey: T::AccountId = Keys::<T>::get( netuid, neuron_uid as u16 );
                if Stake::<T>::contains_key( hotkey.clone() ) {
                    stake[neuron_uid as usize] = I32F32::from_num( Stake::<T>::get( hotkey ) ); 
                }
            }
        }
        stake
    }

    pub fn get_block_at_registration( netuid:u16 ) -> Vec<u64> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize;
        let mut block_at_registration: Vec<u64> = vec![ 0; n ];
        for neuron_uid in 0..n {
            if Keys::<T>::contains_key( netuid, neuron_uid as u16 ){
                block_at_registration[ neuron_uid ] = BlockAtRegistration::<T>::get( netuid, neuron_uid as u16 );
            }
        }
        block_at_registration
    }

    pub fn get_last_update( netuid:u16 ) -> Vec<u64> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize;
        let mut last_update: Vec<u64> = vec![ 0; n ];
        for neuron_uid in 0..n {
            if Keys::<T>::contains_key( netuid, neuron_uid as u16 ){
                last_update[ neuron_uid ] = LastUpdate::<T>::get( netuid, neuron_uid as u16 );
            }
        }
        last_update
    }

    pub fn get_weights_sparse( netuid:u16 ) -> Vec<Vec<(u16, I32F32)>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut weights: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n ]; 
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, weight_ij) in weights_i.iter() { 
                weights [ uid_i as usize ].push( ( *uid_j, u16_proportion_to_fixed( *weight_ij ) ));
            }
        }
        weights
    } 

    pub fn get_weights( netuid:u16 ) -> Vec<Vec<I32F32>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut weights: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(0.0); n ]; n ]; 
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, weight_ij) in weights_i.iter() { 
                weights [ uid_i as usize ] [ *uid_j as usize ] = u16_proportion_to_fixed(  *weight_ij );
            }
        }
        weights
    }

    pub fn get_bonds_sparse( netuid:u16 ) -> Vec<Vec<(u16, I32F32)>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut bonds: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n ]; 
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, bonds_ij) in bonds_i.iter() { 
                bonds [ uid_i as usize ].push( ( *uid_j, u16_proportion_to_fixed( *bonds_ij ) ));
            }
        }
        bonds
    } 

    pub fn get_bonds( netuid:u16 ) -> Vec<Vec<I32F32>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut bonds: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(0.0); n ]; n ]; 
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, bonds_ij) in bonds_i.iter() { 
                bonds [ uid_i as usize ] [ *uid_j as usize ] = u16_proportion_to_fixed( *bonds_ij );
            }
        }
        bonds
    }
}

#[allow(dead_code)]
pub fn fixed_to_u16( x: I32F32 ) -> u16 { x.to_num::<u16>() }

#[allow(dead_code)]
pub fn fixed_to_u64( x: I32F32 ) -> u64 { x.to_num::<u64>() }

#[allow(dead_code)]
pub fn u16_to_fixed( x: u16 ) -> I32F32 { I32F32::from_num( x ) }

#[allow(dead_code)]
pub fn u16_proportion_to_fixed( x: u16 ) -> I32F32 { I32F32::from_num( x ) / I32F32::from_num( u16::MAX ) }

#[allow(dead_code)]
pub fn fixed_proportion_to_u16( x: I32F32 ) -> u16 { fixed_to_u16( x * I32F32::from_num( u16::MAX )) }

#[allow(dead_code)]
pub fn vec_u16_proportions_to_fixed( vec: Vec<u16> ) -> Vec<I32F32> { vec.into_iter().map(|e| u16_proportion_to_fixed(e) ).collect() }

#[allow(dead_code)]
pub fn vec_fixed_proportions_to_u16( vec: Vec<I32F32> ) -> Vec<u16> { vec.into_iter().map(|e| fixed_proportion_to_u16(e) ).collect() }

#[allow(dead_code)]
pub fn sum( x: &Vec<I32F32> ) -> I32F32 { x.iter().sum() }

#[allow(dead_code)]
pub fn normalize( x: &Vec<I32F32> ) -> Vec<I32F32> {
    let x_sum: I32F32 = sum( x );
    if x_sum != I32F32::from_num( 0.0 as f32 ) {
        return x.iter().map( |xi| xi / x_sum ).collect();
    } else {
        return x.clone();
    }
}

#[allow(dead_code)]
pub fn inplace_normalize( x: &mut Vec<I32F32> ) {
    let x_sum: I32F32 = x.iter().sum();
    if x_sum == I32F32::from_num( 0.0 as f32 ){ return }
    for i in 0..x.len() {
        x[i] = x[i]/x_sum;
    }
}

#[allow(dead_code)]
pub fn inplace_row_normalize( x: &mut Vec<Vec<I32F32>> ) {
    for i in 0..x.len() {
        let row_sum: I32F32 = x[i].iter().sum();
        if row_sum > I32F32::from_num( 0.0 as f32 ) {
            x[i].iter_mut().for_each(|x_ij| *x_ij /= row_sum);
        }
    }
}

#[allow(dead_code)]
pub fn inplace_row_normalize_sparse( sparse_matrix: &mut Vec<Vec<(u16, I32F32)>> ) {
    for sparse_row in sparse_matrix.iter_mut() {
        let row_sum: I32F32 = sparse_row.iter().map( | (_j, value) | *value ).sum();
        if row_sum > I32F32::from_num( 0.0 ) {
            sparse_row.iter_mut().for_each( | (_j, value) | *value /= row_sum );
        }
    }
}

#[allow(dead_code)]
pub fn inplace_col_normalize_sparse( sparse_matrix: &mut Vec<Vec<(u16, I32F32)>> ) {
    let n = sparse_matrix.len();
    let mut col_sum: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); n ]; // assume square matrix, rows=cols
    for sparse_row in sparse_matrix.iter() {
        for (j, value) in sparse_row.iter() {
            col_sum[*j as usize] += value;
        }
    }
    for sparse_row in sparse_matrix.iter_mut() {
        for (j, value) in sparse_row.iter_mut() {
            *value /= col_sum[*j as usize];
        }
    }
}

#[allow(dead_code)]
pub fn inplace_col_normalize( x: &mut Vec<Vec<I32F32>> ) {
    if x.len() == 0 { return }
    if x[0].len() == 0 { return }
    let cols = x[0].len();
    let mut col_sum: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); cols ];
    for i in 0..x.len() {
        assert_eq!( x[i].len(), cols );
        for j in 0..cols {
            col_sum[j] += x[i][j];
        }
    }
    for j in 0..cols {
        if col_sum[j] == I32F32::from_num( 0.0 as f32 ) { continue }
        for i in 0..x.len() {
            x[i][j] /= col_sum[j];
        }
    }
}

#[allow(dead_code)]
pub fn inplace_mask_vector( mask: &Vec<bool>, vector: &mut Vec<I32F32> ) {
    if mask.len() == 0 { return }
    assert_eq!( mask.len(), vector.len() );
    for i in 0..mask.len() {
        if !mask[i] {
            vector[i] = I32F32::from_num( 0.0 );
        }
    }
}

#[allow(dead_code)]
pub fn inplace_mask_matrix( mask: &Vec<Vec<bool>>, matrix: &mut Vec<Vec<I32F32>> ) {
    if mask.len() == 0 { return }
    if mask[0].len() == 0 { return }
    assert_eq!( mask.len(), matrix.len() );
    for i in 0..mask.len() {
        for j in 0..mask[i].len() {
            if !mask[i][j] {
                matrix[i][j] = I32F32::from_num( 0.0 );
            }
        }
    }
}

#[allow(dead_code)]
pub fn inplace_diag_mask( matrix: &mut Vec<Vec<I32F32>> ) {
    if matrix.len() == 0 { return }
    if matrix[0].len() == 0 { return }
    assert_eq!( matrix.len(), matrix[0].len() );
    for i in 0..matrix.len() {
        matrix[i][i] = I32F32::from_num( 0.0 );
    }
}

#[allow(dead_code)]
pub fn diag_mask_sparse( sparse_matrix: &Vec<Vec<(u16, I32F32)>> ) -> Vec<Vec<(u16, I32F32)>> {
    let n: usize = sparse_matrix.len();
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            if i != *j as usize {
                result[i].push( (*j, *value) );
            }
        }
    }
    result
}

#[allow(dead_code)]
pub fn vec_mask_sparse_matrix( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, first_vector: &Vec<u64>, second_vector: &Vec<u64>, mask_fn: &dyn Fn(u64, u64) -> bool) -> Vec<Vec<(u16, I32F32)>> {
    let n: usize = sparse_matrix.len();
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            if !mask_fn(first_vector[i], second_vector[*j as usize]) {
                result[i].push( (*j, *value) );
            }
        }
    }
    result
}

#[allow(dead_code)]
/// matrix-vector hadamard product
pub fn hadamard( matrix: &Vec<Vec<I32F32>>, vector: &Vec<I32F32> ) -> Vec<Vec<I32F32>> {
    if matrix.len() == 0 { return vec![ vec![] ] }
    if matrix[0].len() == 0 { return vec![ vec![] ] }
    let mut result: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num( 0.0 ); matrix[0].len() ]; matrix.len() ];
    for i in 0..matrix.len() {
        for j in 0..matrix[i].len() {
            result[i][j] = vector[i] * matrix[i][j];
        }
    }
    result
}

#[allow(dead_code)]
/// sparse matrix-vector hadamard product
pub fn sparse_hadamard( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, vector: &Vec<I32F32> ) -> Vec<Vec<(u16, I32F32)>> {
    let mut result: Vec<Vec<(u16, I32F32)>> = sparse_matrix.clone();
    for (i, sparse_row) in result.iter_mut().enumerate() {
        for (_j, value) in sparse_row.iter_mut() {
            *value *= vector[i];
        }
    }
    result
}

#[allow(dead_code)]
pub fn matmul( matrix: &Vec<Vec<I32F32>>, vector: &Vec<I32F32> ) -> Vec<I32F32> {
    if matrix.len() == 0 { return vec![] }
    if matrix[0].len() == 0 { return vec![] }
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); vector.len() ];
    for i in 0..matrix.len() {
        for j in 0..matrix[i].len() {
            // Compute ranks: r_j = SUM(i) w_ij * s_i
            // Compute trust scores: t_j = SUM(i) w_ij * s_i
            // result_j = SUM(i) vector_i * matrix_ij
            result[j] += vector[i] * matrix[i][j];
        }
    }
    result
}

#[allow(dead_code)]
pub fn matmul_transpose( matrix: &Vec<Vec<I32F32>>, vector: &Vec<I32F32> ) -> Vec<I32F32> {
    if matrix.len() == 0 { return vec![] }
    if matrix[0].len() == 0 { return vec![] }
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); vector.len() ];
    for i in 0..matrix.len() {
        for j in 0..matrix[i].len() {
            // Compute dividends: d_j = SUM(i) b_ji * inc_i
            // result_j = SUM(i) vector_i * matrix_ji
            result[j] += vector[i] * matrix[j][i];
        }
    }
    result
}

#[allow(dead_code)]
pub fn sparse_matmul( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, vector: &Vec<I32F32> ) -> Vec<I32F32> {
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); sparse_matrix.len() ];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            // Compute ranks: r_j = SUM(i) w_ij * s_i
            // Compute trust scores: t_j = SUM(i) w_ij * s_i
            // result_j = SUM(i) vector_i * matrix_ij
            result[*j as usize] += vector[i] * value;
        }
    }
    result
}

#[allow(dead_code)]
pub fn sparse_matmul_transpose( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, vector: &Vec<I32F32> ) -> Vec<I32F32> {
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); sparse_matrix.len() ];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            // Compute dividends: d_j = SUM(i) b_ji * inc_i
            // result_j = SUM(i) vector_i * matrix_ji
            // result_i = SUM(j) vector_j * matrix_ij
            result[i] += vector[*j as usize] * value;
        }
    }
    result
}

#[allow(dead_code)]
pub fn sparse_clip( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, threshold: I32F32, upper: I32F32, lower: I32F32) -> Vec<Vec<(u16, I32F32)>> {
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; sparse_matrix.len() ];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            if *value < threshold {
                result[i].push( (*j, lower) );
            }
            else {
                result[i].push( (*j, upper) );
            }
        }
    }
    result
}

#[allow(dead_code)]
pub fn clip( x: &Vec<Vec<I32F32>>, threshold: I32F32, upper: I32F32, lower: I32F32) -> Vec<Vec<I32F32>> {
    // Check Nill length. 
    if x.len() == 0 {
        return vec![ vec![ ] ];
    }
    let mut result: Vec<Vec<I32F32>> = vec![ vec![ lower; x[0].len() ]; x.len() ]; 
    for i in 0..x.len() {
        for j in 0..x[i].len() {
            if x [ i ][ j ] >= threshold {
                result[ i ][ j ] = upper;
            }
        }
    }
    result
}

#[allow(dead_code)]
pub fn inplace_clip( x: &mut Vec<Vec<I32F32>>, threshold: I32F32, upper: I32F32, lower: I32F32 ) {
    for i in 0..x.len() {
        for j in 0..x[i].len() {
            if x [ i ][ j ] >= threshold {
                x[ i ][ j ] = upper;
            } else {
                x[ i ][ j ] = lower;
            }
        }
    }
}

#[allow(dead_code)]
/// Matrix exponential moving average: alpha * a_ij + one_minus_alpha * b_ij
///
/// # Arguments
///
/// * `new` - new observation
/// * `old` - old observation
/// * `alpha` - EMA coefficient, typically small, higher alpha discounts older observations faster
pub fn mat_ema( new: &Vec<Vec<I32F32>>, old: &Vec<Vec<I32F32>>, alpha: I32F32 ) -> Vec<Vec<I32F32>> {
    if new.len() == 0 { return vec![vec![];1] }
    if new[0].len() == 0 { return vec![vec![];1] }
    let one_minus_alpha:I32F32 = I32F32::from_num( 1.0 ) - alpha;
    let mut result: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num( 0.0 ); new[0].len() ]; new.len() ]; 
    assert!(new.len() == old.len());
    for i in 0..new.len() {
        assert!(new[i].len() == old[i].len());
        for j in 0..new[i].len() {
            result[i][j] = alpha * new[i][j] + one_minus_alpha * old[i][j] 
        }
    }
    result
}

#[allow(dead_code)]
pub fn sparse_mat_ema( new: &Vec<Vec<(u16, I32F32)>>, old: &Vec<Vec<(u16, I32F32)>>, alpha: I32F32 ) -> Vec<Vec<(u16, I32F32)>> {
    assert!(new.len() == old.len());
    let n = new.len(); // assume square matrix, rows=cols
    let zero: I32F32 = I32F32::from_num( 0.0 );
    let one_minus_alpha:I32F32 = I32F32::from_num( 1.0 ) - alpha;
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n ];
    for i in 0..new.len() {
        let mut row: Vec<I32F32> = vec![ zero; n];
        for (j, value) in new[i].iter() {
            row[*j as usize] += alpha * value;
        }
        for (j, value) in old[i].iter() {
            row[*j as usize] += one_minus_alpha * value;
        }
        for (j, value) in row.iter().enumerate() {
            if value > &zero {
                result[i].push( (j as u16, *value) )
            }
        }
    }
    result
}

#[allow(dead_code)]
pub fn sparse_threshold( w: &Vec<Vec<(u16, I32F32)>>, threshold: I32F32 ) -> Vec<Vec<(u16, I32F32)>> {
    let mut sparse_threshold_result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; w.len() ]; 
    for ( uid_i, weights_i ) in w.iter().enumerate() {
        for (uid_j, weight_ij) in weights_i.iter() { 
            if *weight_ij > threshold {
                sparse_threshold_result [ uid_i as usize ].push( ( *uid_j, *weight_ij ));
            }
        }
    }
    sparse_threshold_result
}

#[cfg(test)]
mod tests {
    use substrate_fixed::types::I32F32;
    use crate::epoch::{sum, normalize, inplace_normalize, matmul};

    fn assert_float_compare(a: I32F32, b: I32F32, epsilon: I32F32 ) {
        assert!( I32F32::abs( a - b ) < epsilon, "a({:?}) != b({:?})", a, b);
    }
    
    fn assert_vec_compare(va: &Vec<I32F32>, vb: &Vec<I32F32>, epsilon: I32F32) {
        assert!(va.len() == vb.len());
        for i in 0..va.len(){
            assert!( I32F32::abs( va[i] - vb[i] ) < epsilon, "a_{:?}({:?}) != b_{:?}({:?})", i, va[i], i, vb[i]);
        }  
    }

    #[test]
    fn test_math_sum() {
        assert!( sum(&vec![]) == I32F32::from_num(0));
        assert!( sum(&vec![ I32F32::from_num(1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]) == I32F32::from_num(41));
        assert!( sum(&vec![ I32F32::from_num(-1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]) == I32F32::from_num(39));
    }

    #[test]
    fn test_math_normalize() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let x: Vec<I32F32> = vec![]; 
        let y: Vec<I32F32> = normalize(&x);
        assert_vec_compare( &x, &y, epsilon);
        let x: Vec<I32F32> = vec![ I32F32::from_num(1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]; 
        let y: Vec<I32F32> = normalize(&x);
        assert_vec_compare( &y, &vec![ I32F32::from_num(0.0243902437),  I32F32::from_num(0.243902439),  I32F32::from_num(0.7317073171)], epsilon );
        assert_float_compare( sum( &y ), I32F32::from_num(1.0), epsilon);
        let x: Vec<I32F32> = vec![ I32F32::from_num(-1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]; 
        let y: Vec<I32F32> = normalize(&x);
        assert_vec_compare( &y, &vec![ I32F32::from_num(-0.0256410255),  I32F32::from_num(0.2564102563),  I32F32::from_num(0.769230769)], epsilon );
        assert_float_compare( sum( &y ), I32F32::from_num(1.0), epsilon );
    }

    #[test]
    fn test_math_inplace_normalize() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let mut x1: Vec<I32F32> = vec![ I32F32::from_num(1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]; 
        inplace_normalize(&mut x1);
        assert_vec_compare( &x1, &vec![ I32F32::from_num(0.0243902437),  I32F32::from_num(0.243902439),  I32F32::from_num(0.7317073171)], epsilon );
        let mut x2: Vec<I32F32> = vec![ I32F32::from_num(-1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]; 
        inplace_normalize(&mut x2);
        assert_vec_compare( &x2, &vec![ I32F32::from_num(-0.0256410255),  I32F32::from_num(0.2564102563),  I32F32::from_num(0.769230769)], epsilon );
    }

    #[test]
    fn test_math_matmul() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let w: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(1.0);3 ]; 3 ]; 
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(1.0); 3] ), &vec![ I32F32::from_num(3),  I32F32::from_num(3),  I32F32::from_num(3)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(2.0); 3] ), &vec![ I32F32::from_num(6),  I32F32::from_num(6),  I32F32::from_num(6)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(3.0); 3] ), &vec![ I32F32::from_num(9),  I32F32::from_num(9),  I32F32::from_num(9)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(-1.0); 3] ), &vec![ I32F32::from_num(-3),  I32F32::from_num(-3),  I32F32::from_num(-3)], epsilon );
        let w: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(-1.0);3 ]; 3 ]; 
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(1.0); 3] ), &vec![ I32F32::from_num(-3),  I32F32::from_num(-3),  I32F32::from_num(-3)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(2.0); 3] ), &vec![ I32F32::from_num(-6),  I32F32::from_num(-6),  I32F32::from_num(-6)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(3.0); 3] ), &vec![ I32F32::from_num(-9),  I32F32::from_num(-9),  I32F32::from_num(-9)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(-1.0); 3] ), &vec![ I32F32::from_num(3),  I32F32::from_num(3),  I32F32::from_num(3)], epsilon );
        let w: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(1.0);3 ], vec![ I32F32::from_num(2.0); 3], vec![ I32F32::from_num(3.0);3 ] ]; 
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(0.0); 3] ), &vec![ I32F32::from_num(0.0),  I32F32::from_num(0.0),  I32F32::from_num(0.0)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(2.0); 3] ), &vec![ I32F32::from_num(6),  I32F32::from_num(12),  I32F32::from_num(18)], epsilon );
    }

}