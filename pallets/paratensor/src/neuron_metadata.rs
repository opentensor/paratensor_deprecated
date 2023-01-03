use super::*;
use frame_support::storage::IterableStorageDoubleMap;
use sp_std::vec::Vec;

#[derive(Default)]
struct NeuronMetadata<T:Config> {
    hotkey: T::AccountId,
    coldkey: T::AccountId,
    uid: u16,
    netuid: u16,
    active: bool,
    axon_metadata: AxonMetadata,
    stake: Vec<(T::AccountId, u64)>, // map of coldkey to stake on this neuron/hotkey (includes delegations)
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
	pub fn get_neurons(netuid: u16) -> Vec<NeuronMetadata<T>> {
        if !Self::if_subnet_exist(netuid) {
            return Vec::new()
        }

        let mut neurons = Vec::new();
        let n = SubnetworkN::<T>::get( netuid ); 
        for uid_i in 0..n {
            let mut neuron: NeuronMetadata<T> = NeuronMetadata::<T>::default();

            neuron.uid = uid_i;
            neuron.netuid = netuid;

            let axons_metadata = AxonsMetaData::<T>::get( netuid, uid_i as u16 );
            if axons_metadata.is_some() {
                neuron.axon_metadata = axons_metadata.unwrap();
            } else {
                neuron.axon_metadata = AxonMetadata::default();
            }

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
            neuron.pruning_score = PruningScores::<T>::get( netuid, uid_i as u16 );
            
            neuron.weights = Weights::<T>::get( netuid, uid_i as u16 );
            neuron.bonds = Bonds::<T>::get( netuid, uid_i as u16 );
            
            let mut stakes = Vec::<(T::AccountId, u64)>::new();
            for ( coldkey, stake ) in < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >::iter_prefix( neuron.hotkey.clone() ) {
                stakes.push( (coldkey.clone(), stake) );
            }
            neuron.stake = stakes;
            
            neurons.push( neuron );
        }
        neurons
	}
}

