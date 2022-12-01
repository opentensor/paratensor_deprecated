use super::*;
use sp_runtime::sp_std::if_std;
use frame_support::sp_std::vec;
use frame_support::inherent::Vec;
use substrate_fixed::transcendental::exp;
use substrate_fixed::types::I32F32;
use frame_support::storage::IterableStorageDoubleMap;

impl<T: Config> Pallet<T> {
    pub fn epoch( netuid: u16, rao_emission: u64, debug: bool ) -> Vec<I32F32> {
        /*TO DO (no particular order):
        1. DONE calculate pruning scores
        2. DONE (const) update all other nodes consensus parameters including bonds and weights 
        3. update weights and bonds for node that is identified to be pruned in registration process
        3. DONE set priority
        4. reset Bonds */

        // Get subnetwork size.
        let n: u16 = Self::get_subnetwork_n( netuid );

        // Active vector = 1.0 if block - last_update < activity_cutoff.
        let active: Vec<I32F32> = Self::get_active( netuid );
        if debug { if_std! { println!( "A:\n{:?}\n", active.clone() );}}

        // Access network stake as normalized vector.
        let mut stake: Vec<I32F32> = Self::get_stake( netuid );
        inplace_normalize( &mut stake );
        if debug { if_std! { println!( "S:\n{:?}\n", stake.clone() );}}

        // Access network weights row normalized.
        let weights: Vec<Vec<I32F32>> = Self::get_prunned_weights( netuid );
        if debug { if_std! { println!( "W:\n{:?}\n", weights.clone() );}}

        // Compute ranks.
        let mut ranks: Vec<I32F32> = matmul( &weights, &stake );
        inplace_normalize( &mut ranks );
        if debug { if_std! { println!( "R:\n{:?}\n", ranks.clone() );}}

        // Compute thresholded weights.
        let upper: I32F32 = I32F32::from_num( 1.0 );
        let lower: I32F32 = I32F32::from_num( 0.0 );
        let threshold: I32F32 = I32F32::from_num( 0.01 );
        let clipped_weights: Vec<Vec<I32F32>> = clip( &weights, threshold, upper, lower );
        if debug { if_std! { println!( "tW:\n{:?}\n", clipped_weights.clone() );}}

        // Compute trust scores.
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
        let mut bonds: Vec<Vec<I32F32>> = Self::get_prunned_bonds_dense( netuid );
        inplace_col_normalize( &mut bonds ); // sum_i b_ij = 1
        if debug { if_std! { println!( "B:\n{:?}\n", bonds.clone() );}}        

        // Compute bonds delta column normalized.
        let mut bonds_delta: Vec<Vec<I32F32>> = hadamard( &weights, &stake ); // ΔB = W◦S
        inplace_col_normalize( &mut bonds_delta ); // sum_i b_ij = 1
        if debug { if_std! { println!( "ΔB:\n{:?}\n", bonds_delta.clone() );}}
    
        // Compute bonds moving average.
        let alpha: I32F32 = I32F32::from_num( 0.1 );
        let ema_bonds: Vec<Vec<I32F32>> = mat_ema( &bonds_delta, &bonds, alpha );
        if debug { if_std! { println!( "emaB:\n{:?}\n", ema_bonds.clone() );}}

        // Compute dividends.
        let dividends: Vec<I32F32> = matmul( &ema_bonds, &incentive );
        if debug { if_std! { println!( "D:\n{:?}\n", dividends.clone() );}}

        // Compute emission scores.
        let float_rao_emission: I32F32 = I32F32::from_num( rao_emission );
        let mut normalized_emission: Vec<I32F32> = incentive.iter().zip( dividends.clone() ).map( |(ii, di)| ii + di ).collect();
        inplace_normalize( &mut normalized_emission );
        let emission: Vec<I32F32> = normalized_emission.iter().map( |e| e * float_rao_emission ).collect();
        if debug { if_std! { println!( "E:\n{:?}\n", emission.clone() );}}

        // Compute prunnind scores.
        let mut prunning: Vec<I32F32> = incentive.iter().zip( dividends.clone() ).map( |(ii, di)| ii + di ).collect();
        inplace_normalize( &mut prunning );
        if debug { if_std! { println!( "P:\n{:?}\n", prunning.clone() );}}

        // Sync parameter updates.
        for i in 0..n {
            Self::set_rank( netuid, i, fixed_proportion_to_u16( ranks[i as usize] ) );
            Self::set_trust( netuid, i, fixed_proportion_to_u16( trust[i as usize] ) );
            Self::set_consensus( netuid, i, fixed_proportion_to_u16( consensus[i as usize] ) );
            Self::set_incentive( netuid, i, fixed_proportion_to_u16( incentive[i as usize] ) );
            Self::set_dividend( netuid, i, fixed_proportion_to_u16( dividends[i as usize] ) );
            Self::set_prunning( netuid, i, fixed_proportion_to_u16( prunning[i as usize] ) );
            Self::set_emission( netuid, i, fixed_to_u64( emission[i as usize] ) );
            //Self::set_bonds( netuid, i, Self::vec_fixed_proportions_to_u16( ema_bonds[i as usize] ) );
        }  

        // Remove peers to prune.
        Self::clear_neurons_to_prune_for_subnet(netuid);

        emission
    }

    // Testing function.
    pub fn set_stake_for_testing( hotkey: &T::AccountId, stake:u64 ) { 
        Stake::<T>::insert( hotkey, stake );
    }
    pub fn set_weights_for_testing( netuid: u16, uid: u16, weights: Vec<(u16,u16)>) {
        Weights::<T>::insert(netuid, uid, weights);
    }
    pub fn set_bonds_for_testing( netuid: u16, uid: u16, bonds: Vec<(u16,u16)>) {
        Bonds::<T>::insert(netuid, uid, bonds);
    }

    pub fn set_rank( netuid:u16, neuron_uid: u16, rank:u16 ) { Rank::<T>::insert( netuid, neuron_uid, rank) }
    pub fn set_trust( netuid:u16, neuron_uid:u16, trust:u16) { Trust::<T>::insert( netuid, neuron_uid, trust ) }
    pub fn set_consensus( netuid:u16, neuron_uid:u16, consensus:u16) { Consensus::<T>::insert( netuid, neuron_uid, consensus ) }
    pub fn set_incentive( netuid:u16, neuron_uid:u16, incentive:u16) { Incentive::<T>::insert( netuid, neuron_uid, incentive ) }
    pub fn set_dividend( netuid:u16, neuron_uid:u16, dividend:u16) { Dividends::<T>::insert( netuid, neuron_uid, dividend ) }
    pub fn set_prunning( netuid:u16, neuron_uid:u16, prunning:u16) { PrunningScores::<T>::insert( netuid, neuron_uid, prunning ) }
    pub fn set_emission( netuid:u16, neuron_uid:u16, emission:u64) { Emission::<T>::insert( netuid, neuron_uid, emission ) }
    pub fn set_bonds( netuid:u16, neuron_uid:u16, bonds:Vec<(u16,u16)>) { Bonds::<T>::insert( netuid, neuron_uid, bonds ) }

    pub fn get_float_rho( netuid:u16 ) -> I32F32 { I32F32::from_num( Self::get_rho( netuid ) )  }
    pub fn get_float_kappa( netuid:u16 ) -> I32F32 { I32F32::from_num( Self::get_kappa( netuid )  ) / I32F32::from_num( u16::MAX ) }
    pub fn get_last_update( netuid:u16, neuron_uid: u16 ) -> u64 { LastUpdate::<T>::get( netuid, neuron_uid ) }
    pub fn get_block_at_registration( neuron_uid: u16 ) -> u64 { BlockAtRegistration::<T>::get( neuron_uid ) }
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

    pub fn get_active( netuid:u16 ) -> Vec<I32F32> {
        let block: u64 = Self::get_current_block_as_u64();
        let activity_cutoff: u64 = Self::get_activity_cutoff( netuid ) as u64;
        let n: usize = Self::get_subnetwork_n( netuid ) as usize;
        let mut active: Vec<I32F32> = vec![  I32F32::from_num(0.0); n ];
        for neuron_uid in 0..n {
            if Keys::<T>::contains_key( netuid, neuron_uid as u16 ){
                let last_update: u64 = Self::get_last_update( netuid, neuron_uid as u16 );
                if block - last_update < activity_cutoff {
                    active[neuron_uid as usize] = I32F32::from_num( 1.0 );
                }
            }
        }
        active
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

    pub fn get_prunned_weights( netuid:u16 ) -> Vec<Vec<I32F32>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize;
        let mut weights: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(0.0); n ]; n ];
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            let last_update: u64 = Self::get_last_update( netuid, uid_i );
            for (uid_j, weight_ij) in weights_i.iter() {
                let block_at_registration: u64 = Self::get_block_at_registration( *uid_j );
                if block_at_registration < last_update || !NeuronsShouldPruneAtNextEpoch::<T>::contains_key( netuid, *uid_j as u16 ) {
                    weights [ uid_i as usize ] [ *uid_j as usize ] = u16_proportion_to_fixed( *weight_ij );
                }
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

    pub fn get_prunned_bonds_sparse( netuid:u16 ) -> Vec<Vec<(u16, I32F32)>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut bonds: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n ]; 
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            let last_update: u64 = Self::get_last_update( netuid, uid_i );
            for (uid_j, bonds_ij) in bonds_i.iter() { 
                let block_at_registration: u64 = Self::get_block_at_registration( *uid_j );
                if block_at_registration < last_update || !NeuronsShouldPruneAtNextEpoch::<T>::contains_key( netuid, *uid_j as u16 ) {
                    bonds [ uid_i as usize ].push( ( *uid_j, u16_proportion_to_fixed( *bonds_ij ) ));
                }
            }
        }
        bonds
    } 

    pub fn get_prunned_bonds_dense( netuid:u16 ) -> Vec<Vec<I32F32>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut bonds: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(0.0); n ]; n ]; 
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            let last_update: u64 = Self::get_last_update( netuid, uid_i );
            for (uid_j, bonds_ij) in bonds_i.iter() { 
                let block_at_registration: u64 = Self::get_block_at_registration( *uid_j );
                if block_at_registration < last_update || !NeuronsShouldPruneAtNextEpoch::<T>::contains_key( netuid, *uid_j as u16 ) {
                    bonds [ uid_i as usize ] [ *uid_j as usize ] = u16_proportion_to_fixed( *bonds_ij );
                }
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
/// matrix-vector hadamard product
pub fn hadamard( w: &Vec<Vec<I32F32>>, x: &Vec<I32F32> ) -> Vec<Vec<I32F32>> {
    if w.len() == 0 { return vec![ vec![] ] }
    if w[0].len() == 0 { return vec![ vec![] ] }
    let mut result: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num( 0.0 ); x.len() ]; w[0].len() ];
    for (i, w_row) in w.iter().enumerate() {
        for (j, x_i) in x.iter().enumerate() {
            result [ i ][ j ] = x_i * w_row [ j ]
        }
    }
    result
}

#[allow(dead_code)]
pub fn matmul( w: &Vec<Vec<I32F32>>, x: &Vec<I32F32> ) -> Vec<I32F32> {
    if w.len() == 0 { return vec![] }
    if w[0].len() == 0 { return vec![] }
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); x.len() ];
    for (i, w_row) in w.iter().enumerate() {
        for (j, x_i) in x.iter().enumerate() {
            result [ i ] += x_i * w_row [ j ] 
        }
    }
    result
}

#[allow(dead_code)]
pub fn sparse_matmul( w: &Vec<Vec<(u16, I32F32)>>, x: &Vec<I32F32> ) -> Vec<I32F32> {
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); x.len() ];
    for row in w.iter() {
        for r_i in row.iter() {
            result [ r_i.0 as usize ] = r_i.1 * x[ r_i.0 as usize ]; 
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
/// * `a` - new observation
/// * `b` - old observation
/// * `alpha` - EMA coefficient, typically small, higher alpha discounts older observations faster
pub fn mat_ema( a: &Vec<Vec<I32F32>>, b: &Vec<Vec<I32F32>>, alpha: I32F32 ) -> Vec<Vec<I32F32>> {
    if a.len() == 0 { return vec![vec![];1] }
    if a[0].len() == 0 { return vec![vec![];1] }
    let one: I32F32 = I32F32::from_num( 1.0 );
    let one_minus_alpha:I32F32 = I32F32::from_num( 1.0 ) - alpha;
    let mut result: Vec<Vec<I32F32>> = vec![ vec![ one; a[0].len() ]; a.len() ]; 
    assert!(a.len() == b.len());
    for i in 0..a.len() {
        assert!(a[i].len() == b[i].len());
        for j in 0..a[i].len() {
            result[i][j] = alpha * a[i][j] + one_minus_alpha * b[i][j] 
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