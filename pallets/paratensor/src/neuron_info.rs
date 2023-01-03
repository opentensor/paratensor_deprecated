use super::*;
use serde::{Serialize, Deserialize};
use frame_support::storage::IterableStorageDoubleMap;
use frame_support::pallet_prelude::{Decode, Encode};
use sp_std::vec::Vec;

#[derive(Decode, Encode, Serialize, Deserialize)]
pub struct NeuronInfo {//<T:Config> {
    //hotkey: T::AccountId,
    //coldkey: T::AccountId,
    uid: u16,
    netuid: u16,
    active: bool,
    axon_metadata: AxonMetadata,
    //stake: Vec<(T::AccountId, u64)>, // map of coldkey to stake on this neuron/hotkey (includes delegations)
    rank: u16,
    emission: u64,
    incentive: u16,
    consensus: u16,
    trust: u16,
    dividends: u16,
    last_update: u64,
    weights: Vec<(u16, u16)>,
    bonds: Vec<(u16, u16)>,
    pruning_score: u16
}

impl<T: Config> Pallet<T> {
	pub fn get_neurons(netuid: u16) -> Vec<NeuronInfo> {//<T>> {
        if !Self::if_subnet_exist(netuid) {
            return Vec::new()
        }

        let mut neurons = Vec::new();
        let n = SubnetworkN::<T>::get( netuid ); 
        for uid_i in 0..n {
            let uid = uid_i;
            let netuid = netuid;

            let axons_metadata = AxonsMetaData::<T>::get( netuid, uid_i as u16 );
            let axon_metadata;
            if axons_metadata.is_some() {
                axon_metadata = axons_metadata.unwrap();
            } else {
                axon_metadata = AxonMetadata::default();
            }

            let hotkey = Keys::<T>::get( netuid, uid_i as u16 ).clone();
            let coldkey = Owner::<T>::get( hotkey.clone() ).clone();

            let last_update = LastUpdate::<T>::get( netuid, uid_i as u16 );
            
            // TODO: replace with last_update check if we remove Active storage
            let active = Active::<T>::get( netuid, uid_i as u16 );

            let rank = Rank::<T>::get( netuid, uid_i as u16 );
            let emission = Emission::<T>::get( netuid, uid_i as u16 );
            let incentive = Incentive::<T>::get( netuid, uid_i as u16 );
            let consensus = Consensus::<T>::get( netuid, uid_i as u16 );
            let trust = Trust::<T>::get( netuid, uid_i as u16 );
            let dividends = Dividends::<T>::get( netuid, uid_i as u16 );
            let pruning_score = PruningScores::<T>::get( netuid, uid_i as u16 );
            
            let weights = Weights::<T>::get( netuid, uid_i as u16 );
            let bonds = Bonds::<T>::get( netuid, uid_i as u16 );
            
            let mut stakes = Vec::<(T::AccountId, u64)>::new();
            for ( coldkey, stake ) in < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >::iter_prefix( hotkey.clone() ) {
                stakes.push( (coldkey.clone(), stake) );
            }
            let stake = stakes;

            let neuron = NeuronInfo {
                //hotkey,
                //coldkey,
                uid,
                netuid,
                active,
                axon_metadata,
                //stake,
                rank,
                emission,
                incentive,
                consensus,
                trust,
                dividends,
                last_update,
                weights,
                bonds,
                pruning_score
            };
            
            neurons.push( neuron );
        }
        neurons
	}
}

