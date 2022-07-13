use super::*;
use fast_math;
use sp_runtime::sp_std::if_std;

use array;

#[cfg(feature = "no_std")]
use ndarray::{Array1, Array2};

impl<T: Config> Pallet<T> {
    
    pub fn epoch( netuid: u16, total_emission: u64, debug: bool ) -> ndarray::Array1<u64> {

        // Pull netork state.
        let stake: ndarray::Array1<f32> = Self::get_stake_as_float_array( netuid );
        
        let mut weights: ndarray::Array2<f32> = Self::get_weights_as_float_matrix( netuid );
        let mut bonds: ndarray::Array2<f32> = Self::get_bonds_as_float_matrix( netuid );
        Self::matrix_row_normalize( &mut weights );
        Self::matrix_row_normalize( &mut bonds );
        if debug { if_std! { println!( "S:\n{}\n", stake.clone() );}}
        if debug { if_std! { println!( "W:\n{}\n", weights.clone() );}}
        if debug { if_std! { println!( "B:\n{}\n", bonds.clone() );}}

        // Compute ranks.
        let mut ranks: ndarray::Array1<f32> = weights.t().dot( &stake );
        Self::vector_normalize( &mut ranks );
        Self::set_rank_from_array( netuid, ranks.clone() );
        if debug { if_std! { println!( "Wt * S = R:\n{}\n", ranks.clone() );}}

        // Compute thresholded weights.
        let threshold: f32 = Self::get_trust_threshold( netuid );
        let weights_non_zero: ndarray::Array2<f32> = weights.mapv( |x| if x > 0.2 { 1.0 } else { 0.0 } );
        if debug { if_std! { println!( "W > {} = W#:\n{}\n", threshold, weights_non_zero);}}

        // Compute trust.
        let trust: ndarray::Array1<f32> = weights_non_zero.t().dot( &stake );
        Self::set_trust_from_array( netuid, trust.clone() );
        if debug { if_std! { println!( "W# * S = T:\n{}\n", trust.clone());}}

        // Compute consensus.
        let rho: f32 = Self::get_rho( netuid );
        let kappa: f32 = Self::get_kappa( netuid );
        let mut consensus: ndarray::Array1<f32> = trust.mapv_into( |t| 1.0 / ( 1.0 + fast_math::exp( -rho * (t - kappa) ) ));
        Self::vector_normalize( &mut consensus );
        Self::set_consensus_from_array( netuid, consensus.clone() );
        if debug { if_std! { println!( " sig( -rho (T - kappa) ) = C:\n{}\n", consensus.clone());}}

        // Compute incentive.
        let mut incentive: ndarray::Array1<f32> = ranks * consensus;
        Self::vector_normalize( &mut incentive );
        Self::set_incentive_from_array( netuid, incentive.clone() );
        if debug { if_std! { println!( " R * C = I:\n{}\n", incentive.clone());}}

        // Compute dividends
        let mut dividends: ndarray::Array1<f32> = bonds.t().dot( &incentive );
        Self::vector_normalize( &mut dividends );
        Self::set_dividends_from_array( netuid, dividends.clone() );
        if debug { if_std! { println!( " B * I = D:\n{}\n", dividends.clone());}}
           
        // Compute delta bonds
        let alpha: f32 = 0.90;
        let next_bonds: ndarray::Array2<f32> = alpha * bonds + ( 1.0 - alpha ) * weights;
        Self::set_bonds_from_float_matrix( netuid, next_bonds.clone() );
        if debug { if_std! { println!( " {}B + (1 - {})W = dB:\n{}\n", alpha, alpha, next_bonds.clone());}}

        // Compute emission
        let emission: ndarray::Array1<u64> = dividends.map( |d| (d * total_emission as f32) as u64 );
        Self::set_emission_from_array( netuid, emission.clone() );
        if debug { if_std! { println!( " D * {} = E:\n{}\n", total_emission, emission.clone());}}

        emission
    }
}

