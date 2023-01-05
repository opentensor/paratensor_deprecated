use super::*;
use serde::{Serialize, Deserialize};
use frame_support::storage::IterableStorageDoubleMap;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use alloc::vec::Vec;

#[derive(Decode, Encode, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct NeuronInfo {
    hotkey: DeAccountId,
    coldkey: DeAccountId,
    uid: u16,
    netuid: u16,
    active: bool,
    axon_info: AxonInfo,
    stake: Vec<(DeAccountId, u64)>, // map of coldkey to stake on this neuron/hotkey (includes delegations)
    rank: u16,
    emission: u64,
    incentive: u16,
    consensus: u16,
    trust: u16,
    dividends: u16,
    last_update: u64,
    weights: Vec<(u16, u16)>, // map of uid to weight
    bonds: Vec<(u16, u16)>, // map of uid to bond
    pruning_score: u16
}

#[derive(Decode, Encode, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
struct DeAccountId { // allows us to de/serialize the account id as a u8 vec
    #[serde(with = "serde_bytes")]
    id: Vec<u8>
}

impl From<Vec<u8>> for DeAccountId {
    fn from(v: Vec<u8>) -> Self {
        DeAccountId {
            id: v.clone()
        }
    }
}


impl<T: Config> Pallet<T> {
	pub fn get_neurons(netuid: u16) -> Vec<NeuronInfo> {
        if !Self::if_subnet_exist(netuid) {
            return Vec::new();
        }

        let mut neurons = Vec::new();
        let n = SubnetworkN::<T>::get( netuid ); 
        for uid_i in 0..n {
            let uid = uid_i;
            let netuid = netuid;

            let axon_ = Axons::<T>::get( netuid, uid_i as u16 );
            let axon_info;
            if axon_.is_some() {
                axon_info = axon_.unwrap();
            } else {
                axon_info = AxonInfo::default();
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
            
            let mut stakes = Vec::<(DeAccountId, u64)>::new();
            for ( coldkey, stake ) in < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >::iter_prefix( hotkey.clone() ) {
                stakes.push( (coldkey.clone().encode().into(), stake) );
            }

            let stake = stakes;

            let neuron = NeuronInfo {
                hotkey: hotkey.clone().encode().into(),
                coldkey: coldkey.clone().encode().into(),
                uid,
                netuid,
                active,
                axon_info,
                stake,
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

