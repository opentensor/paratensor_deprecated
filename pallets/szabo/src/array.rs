use super::*;
use frame_support::inherent::Vec;
use frame_support::sp_std::vec;
use substrate_fixed::types::I65F63;
use frame_support::storage::IterableStorageDoubleMap;

impl<T: Config> Pallet<T> {

    /*
    pub fn dvec_to_fixed_dvec( a: Vec<u16> ) -> Vec<I65F63> {
        let mut fixed_float_array: Vec<I65F63> = vec![ I65F63::from_num(0.0) ; a.len() ];
        for (i, ai) in a.iter().enumerate() {
            fixed_float_array[i] = I65F63::from_num( *ai ) / I65F63::from_num( u16::MAX );
        }
        fixed_float_array
    }

    pub fn svec_to_fixed_svec( a: &Vec<(u16, u16)> ) -> Vec<(u16, I65F63)> {
        let mut fixed_sparse_vec: Vec<(u16, I65F63)> = vec![];
        for ( i, (coli, ai) ) in a.iter().enumerate() {
            let fixed_float: I65F63 = I65F63::from_num( *ai ) / I65F63::from_num( u16::MAX );
            fixed_sparse_vec.push( (*coli, fixed_float) );
        }
        fixed_sparse_vec.sort_by( |a, b| a.0.cmp(&b.0) );
        fixed_sparse_vec
    }

    pub fn smat_to_fixed_smat( a: &Vec<Vec<(u16, u16)>> ) -> Vec<Vec<(u16, I65F63)>> {
        let mut fixed_sparse_matrix: Vec<Vec<(u16, I65F63)>> = vec![];
        for (i, ai) in a.iter().enumerate() {
            let sparse_row: Vec<(u16, I65F63)> = Self::svec_to_fixed_svec( ai );
            fixed_sparse_matrix.push( sparse_row );
        }
        fixed_sparse_matrix
    }

    pub fn smat_mul_dvec( x: &Vec<Vec<(u16, I65F63)>>, y: &Vec<I65F63> ) -> Vec<I65F63> {
        let mut dense_result: Vec<I65F63> = vec![ I65F63::from_num::<f32>(0.0) ; y.len() ];
        for xi in x.iter() {
            for (colj, xij) in xi.iter() {
                assert!( (*colj as usize) < y.len() );
                dense_result[*colj as usize] = dense_result[*colj as usize] + xij * y[*colj as usize];
            }
        }
        dense_result
    }

    pub fn dmat_mul_dvec( x: &Vec<Vec<I65F63>>, y: &Vec<I65F63> ) -> Vec<I65F63> {
        let mut dense_result: Vec<I65F63> = vec![ I65F63::from_num(0.0) ; y.len() ];
        for xi in x.iter() {
            assert!( xi.len() == y.len() );
            for (j, xij) in xi.iter().enumerate() {
                dense_result[j] = dense_result[j] + xij * y[j];
            }
        }
        dense_result
    }
    pub fn hadamard( a: &Vec<I65F63>, b: &Vec<I65F63>) -> Vec<I65F63> {
        assert!( a.len() != b.len(), "a and b must have the same length.");
        let mut hadamard_result: Vec<I65F63> = vec![ I65F63::from_num(0.0) ; a.len() ];
        for (i, (ai, bi) ) in a.iter().zip(b.iter()).enumerate() {
            hadamard_result[i] = *ai * *bi;
        }
        hadamard_result
    }
    pub fn sum( a: &Vec<I65F63> ) -> I65F63 {
        let mut sum_result: I65F63 = I65F63::from_num( 0.0 );
        for ai in a.iter(){ sum_result += ai; }
        sum_result
    }
    pub fn ssum( a: &Vec<(u16, I65F63)> ) -> I65F63 {
        let mut sum_result: I65F63 = I65F63::from_num( 0.0 );
        for (_, ai) in a.iter(){ sum_result += ai; }
        sum_result
    }
    pub fn normalize( mut a: Vec<I65F63> ) {
        let array_sum: I65F63 = Self::sum( &a );
        for i in 0..a.len() {
            a[i] = a[i] / array_sum;
        }
    }
    pub fn snormalize( mut a: Vec<(u16, I65F63)> ) {
        let array_sum: I65F63 = Self::ssum( &a );
        for i in 0..a.len() {
            a[i] = ( a[i].0, a[i].1 / array_sum ) ;
        }
    }
    */
}