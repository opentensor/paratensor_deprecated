use super::*;
use frame_support::storage::IterableStorageMap;
use frame_support::{sp_std::vec};
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    struct NeuronMetadata {
        hotkey: T::AccountId,
        coldkey: T::AccountId,
        uid: u16,
        netuid: u16,
        active: bool,
        axonMetadata: T::AxonMetadata,
        stake: Vec<(T::AccountId, u64)>, // map of coldkey to stake on this neuron/hotkey (includes delegations)
        rank: u16,
        emission: u16,
        incentive: u16,
        consensus: u16,
        trust: u16,
        dividends: u16,
        last_update: u64,
        weights: Vec<(u16, u16)>,
        bonds: Vec<(u16, u16)>,
        pruningScore: u16,
    }
    
	pub fn get_neurons(netuid: u16) -> NeuronMetadata {
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);

        let mut neurons = Vec::new();
        let n = SubnetworkN::<T>::get( netuid ); 
        for uid_i in 0..n {
            let mut neuron = NeuronMetadata {
                uid: uid_i,
                netuid: netuid,
            }

            neuron.axonMetadata = AxonsMetaData::<T>::get( netuid, uid_i as u16 );
            neuron.hotkey = Keys::<T>::get( netuid, uid_i as u16 ).clone();
            neuron.coldkey = Owner::<T>::get( neuron.hotkey ).clone();

            neuron.last_update = LastUpdate::<T>::get( netuid, uid_i as u16 );
            
            // TODO: replace with last_update check if we remove Active storage
            neuron.active = Active::<T>::get( netuid, uid_i as u16 );

            neuron.rank = Rank::<T>::get( netuid, uid_i as u16 );
            neuron.emission = Emission::<T>::get( netuid, uid_i as u16 );
            neuron.incentive = Incentive::<T>::get( netuid, uid_i as u16 );
            neuron.consensus = Consensus::<T>::get( netuid, uid_i as u16 );
            neuron.trust = Trust::<T>::get( netuid, uid_i as u16 );
            neuron.dividends = Dividends::<T>::get( netuid, uid_i as u16 );
            neuron.pruningScore = PruningScores::<T>::get( netuid, uid_i as u16 );
            
            neuron.weights = Weights::<T>::get( netuid, uid_i as u16 );
            neuron.bonds = Bonds::<T>::get( netuid, uid_i as u16 );
            
            let mut stakes = Vec<(AccountId, u64)>::new();
            for ( coldkey, stake ) in < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >>::iter_prefix( neuron.hotkey ) {
                stakes.push( (coldkey.clone(), stake) );
            }
            neuron.stake = stakes;
            
            neurons.push( neuron );
        }

	}
}

