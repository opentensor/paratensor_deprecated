use super::*;

#[cfg(feature = "no_std")]
use ndarray::{Array1, Array2};

impl<T: Config> Pallet<T> {
    pub fn get_current_block_as_u64( ) -> u64 {
        let block_as_u64: u64 = TryInto::try_into( system::Pallet::<T>::block_number() ).ok().expect("blockchain will not exceed 2^64 blocks; QED.");
        block_as_u64
    }
    pub fn vector_normalize( vector: &mut ndarray::Array1<f32> ) {
        let vector_sum = vector.sum();
        if vector_sum > 0.0 { vector.mapv( |xi| xi / vector_sum ); }
    }
    pub fn matrix_row_normalize( matrix: &mut ndarray::Array2<f32> ) {
        for row in matrix.rows_mut() { 
            let row_sum = row.sum();
            if row_sum > 0.0 { row.mapv( |xi| xi / row_sum ); }
        }
    }

}

