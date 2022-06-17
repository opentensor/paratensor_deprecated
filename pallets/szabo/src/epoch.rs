use super::*;

#[cfg(feature = "no_std")]
extern crate nalgebra as na;

impl<T: Config> Pallet<T> {
    pub fn epoch( netuid: u16, emission: u64 ) {

        let stake: nalgebra::DVector<f32> = Self::get_stake_as_float_vector( netuid );
        let weights: nalgebra::DMatrix<f32> = Self::get_weights_as_float_matrix( netuid );
        let bonds: nalgebra::DMatrix<f32> = Self::get_bonds_as_float_matrix( netuid );
        let ranks = weights * stake;
        Self::set_rank_from_vector( netuid, ranks );
    }
}

