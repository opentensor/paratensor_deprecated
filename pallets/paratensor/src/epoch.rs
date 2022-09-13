use super::*;
use sp_runtime::sp_std::if_std;
use frame_support::sp_std::vec;
use frame_support::inherent::Vec;
use substrate_fixed::transcendental::exp;
use substrate_fixed::types::I32F32;
use frame_support::storage::IterableStorageDoubleMap;

impl<T: Config> Pallet<T> {
    pub fn epoch( netuid: u16, _total_emission: u64, debug: bool ) {

        // Access network stake as normalized vector.
        let mut stake: Vec<I32F32> = Self::get_stake( netuid );
        Self::inplace_normalize( &mut stake );
        if debug { if_std! { println!( "S:\n{:?}\n", stake.clone() );}}

        // Access network weights row normalized.
        let weights: Vec<Vec<I32F32>> = Self::get_weights( netuid );
        if debug { if_std! { println!( "W:\n{:?}\n", weights.clone() );}}

        // Acess network bonds row normalized.
        let bonds: Vec<Vec<I32F32>> = Self::get_bonds( netuid );
        if debug { if_std! { println!( "B:\n{:?}\n", bonds.clone() );}}

        // Compute ranks.
        let ranks: Vec<I32F32> = Self::matmul( &weights, &stake );
        if debug { if_std! { println!( "R:\n{:?}\n", ranks.clone() );}}

        // Compute thresholded weights.
        let upper: I32F32 = I32F32::from_num( 1.0 );
        let lower: I32F32 = I32F32::from_num( 0.0 );
        let threshold: I32F32 = I32F32::from_num( 0.01 );
        let clipped_weights: Vec<Vec<I32F32>> = Self::clip( &weights, threshold, upper, lower );
        if debug { if_std! { println!( "tW:\n{:?}\n", clipped_weights.clone() );}}

        // Compute trust scores.
        let trust: Vec<I32F32> = Self::matmul( &clipped_weights, &stake );
        if debug { if_std! { println!( "T:\n{:?}\n", trust.clone() );}}

        // Compute consensus.
        let one: I32F32 = I32F32::from_num(1.0);
        let rho: I32F32 = I32F32::from_num(10.0);
        let kappa: I32F32 = I32F32::from_num(0.5);
        let exp_trust: Vec<I32F32> = trust.iter().map( |t|  exp( -rho * (t - kappa) ).expect("") ).collect();
        let consensus: Vec<I32F32> = exp_trust.iter().map( |t|  one /(one + t) ).collect();
        if debug { if_std! { println!( "C:\n{:?}\n", consensus.clone() );}}

        // Compute incentive.
        let mut incentive: Vec<I32F32> = ranks.iter().zip( consensus ).map( |(ri, ci)| ri * ci ).collect();
        Self::inplace_normalize( &mut incentive );
        if debug { if_std! { println!( "I:\n{:?}\n", incentive.clone() );}}

        // Compute dividends.
        let dividends: Vec<I32F32> = Self::matmul( &bonds, &incentive );
        if debug { if_std! { println!( "D:\n{:?}\n", dividends.clone() );}}
    
        // Compute bonds moving average.
        let alpha: I32F32 = I32F32::from_num( 0.9 );
        let ema_bonds: Vec<Vec<I32F32>> = Self::mat_ema( &weights, &bonds, alpha );
        if debug { if_std! { println!( "emaB:\n{:?}\n", ema_bonds.clone() );}}

    }

    pub fn sum( x: &Vec<I32F32> ) -> I32F32 {
        x.iter().sum()
    }

    pub fn normalize( x: &Vec<I32F32> ) -> Vec<I32F32> {
        let x_sum: I32F32 = Self::sum( x );
        if x_sum != I32F32::from_num( 0.0 as f32 ) {
            return x.iter().map( |xi| xi / x_sum ).collect();
        } else {
            return x.clone();
        }
    }

    pub fn inplace_normalize( x: &mut Vec<I32F32> ) {
        let x_sum: I32F32 = x.iter().sum();
        if x_sum == I32F32::from_num( 0.0 as f32 ){ return }
        for i in 0..x.len() {
            x[i] = x[i]/x_sum;
        }
    }

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

    pub fn sparse_matmul( w: &Vec<Vec<(u16, I32F32)>>, x: &Vec<I32F32> ) -> Vec<I32F32> {
        let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); x.len() ];
        for row in w.iter() {
            for r_i in row.iter() {
                result [ r_i.0 as usize ] = r_i.1 * x[ r_i.0 as usize ]; 
            }
        }
        result
    }

    pub fn clip( x: &Vec<Vec<I32F32>>, threshold: I32F32, upper: I32F32, lower: I32F32) -> Vec<Vec<I32F32>> {
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

    pub fn fixed_to_u16( x: I32F32 ) -> u16 { x.to_num::<u16>() }
    pub fn u16_to_fixed( x: u16 ) -> I32F32 { I32F32::from_num( x ) }
    pub fn u16_proportion_to_fixed( x: u16 ) -> I32F32 { I32F32::from_num( x ) / I32F32::from_num( u16::MAX ) }
    pub fn fixed_proportion_to_u16( x: I32F32 ) -> u16 { Self::fixed_to_u16( x * I32F32::from_num( u16::MAX )) }
    pub fn vec_u16_proportions_to_fixed( vec: Vec<u16> ) -> Vec<I32F32> { vec.into_iter().map(|e| Self::u16_proportion_to_fixed(e) ).collect() }
    pub fn vec_fixed_proportions_to_u16( vec: Vec<I32F32> ) -> Vec<u16> { vec.into_iter().map(|e| Self::fixed_proportion_to_u16(e) ).collect() }

    pub fn set_ranks( netuid:u16, ranks:Vec<I32F32> ) { Rank::<T>::insert( netuid, Self::vec_fixed_proportions_to_u16( ranks ) ) }
    pub fn set_trust( netuid:u16, trust:Vec<I32F32> ) { Trust::<T>::insert( netuid, Self::vec_fixed_proportions_to_u16( trust ) ) }
    pub fn set_consensus( netuid:u16, consensus:Vec<I32F32> ) { Consensus::<T>::insert( netuid, Self::vec_fixed_proportions_to_u16( consensus ) ) }
    pub fn set_incentives( netuid:u16, incentive:Vec<I32F32> ) { Incentive::<T>::insert( netuid, Self::vec_fixed_proportions_to_u16( incentive ) ) }
    pub fn set_dividends( netuid:u16, dividends:Vec<I32F32> ) { Dividends::<T>::insert( netuid, Self::vec_fixed_proportions_to_u16( dividends ) ) }

    pub fn get_ranks( netuid:u16 ) -> Vec<I32F32> {  Self::vec_u16_proportions_to_fixed( Rank::<T>::get( netuid ) ) }
    pub fn get_trust( netuid:u16 ) -> Vec<I32F32> { Self::vec_u16_proportions_to_fixed( Trust::<T>::get( netuid ) ) }
    pub fn get_consensus( netuid:u16 ) -> Vec<I32F32> { Self::vec_u16_proportions_to_fixed( Consensus::<T>::get( netuid ) ) }
    pub fn get_incentives( netuid:u16 ) -> Vec<I32F32> { Self::vec_u16_proportions_to_fixed( Incentive::<T>::get( netuid ) )  }
    pub fn get_dividends( netuid:u16 ) -> Vec<I32F32> { Self::vec_u16_proportions_to_fixed( Dividends::<T>::get( netuid ) )  }

    pub fn get_stake( netuid:u16 ) -> Vec<I32F32> {
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut stake: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); n ]; 
        for ( uid_i, _ ) in <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix( netuid ){ 
            stake [ uid_i as usize ] = I32F32::from_num( 0.0 );
        }
        stake
    }

    pub fn get_weights_sparse( netuid:u16 ) -> Vec<Vec<(u16, I32F32)>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut weights: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n ]; 
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, weight_ij) in weights_i.iter() { 
                weights [ uid_i as usize ].push( ( *uid_j, Self::u16_proportion_to_fixed( *weight_ij ) ));
            }
        }
        weights
    } 

    pub fn get_weights( netuid:u16 ) -> Vec<Vec<I32F32>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut weights: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(0.0); n ]; n ]; 
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, weight_ij) in weights_i.iter() { 
                weights [ uid_i as usize ] [ *uid_j as usize ] = Self::u16_proportion_to_fixed(  *weight_ij );
            }
        }
        weights
    } 

    pub fn get_bonds_sparse( netuid:u16 ) -> Vec<Vec<(u16, I32F32)>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut bonds: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n ]; 
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, bonds_ij) in bonds_i.iter() { 
                bonds [ uid_i as usize ].push( ( *uid_j, Self::u16_proportion_to_fixed( *bonds_ij ) ));
            }
        }
        bonds
    } 

    pub fn get_bonds( netuid:u16 ) -> Vec<Vec<I32F32>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut bonds: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(0.0); n ]; n ]; 
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, bonds_ij) in bonds_i.iter() { 
                bonds [ uid_i as usize ] [ *uid_j as usize ] = Self::u16_proportion_to_fixed( *bonds_ij );
            }
        }
        bonds
    } 
}