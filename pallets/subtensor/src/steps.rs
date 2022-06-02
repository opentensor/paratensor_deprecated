use super::*;
use sp_std::convert::TryInto;
use substrate_fixed::types::I65F63;
use substrate_fixed::transcendental::exp;
use substrate_fixed::transcendental::log2;
use frame_support::IterableStorageMap;

const LOG_TARGET: &'static str = "runtime::subtensor::step";

impl<T: Config> Pallet<T> {
    //
    pub fn get_current_block_as_u64( ) -> u64 {
        let block_as_u64: u64 = TryInto::try_into( system::Pallet::<T>::block_number() ).ok().expect("blockchain will not exceed 2^64 blocks; QED.");
        block_as_u64
    }
}