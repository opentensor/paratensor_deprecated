use super::*;
use substrate_fixed::types::I65F63;
use frame_support::{IterableStorageMap};
use sp_std::convert::TryInto;
use sp_core::{H256, U256};
use sp_io::hashing::sha2_256;
use sp_io::hashing::keccak_256;
use frame_system::{ensure_signed};

const LOG_TARGET: &'static str = "runtime::subtensor::registration";

impl<T: Config> Pallet<T> {

    /// ---- Adds an account to this network under the uid.
    pub fn add_account_under_uid( uid: u16, hotkey: &T::AccountId, coldkey: &T::AccountId ) {
        if !Hotkeys::<T>::contains_key( uid ) { 
            Hotkeys::<T>::insert( uid, hotkey.clone() );
            Coldkeys::<T>::insert( uid, coldkey.clone() );
            Uids::<T>::insert( hotkey.clone(), uid );
        }
    }
    
    /// ---- Removes an account from this network under the uid.
    pub fn remove_account_under_uid( uid: u16 ) {
        if !Hotkeys::<T>::contains_key( uid ) { 
            let hotkey = Hotkeys::<T>::get( uid );
            Hotkeys::<T>::remove( uid );
            Coldkeys::<T>::remove( uid );
            Uids::<T>::remove( hotkey );
        }
    }

    pub fn do_registration( 
        origin: T::Origin, 
        block_number: u64, 
        nonce: u64, 
        work: Vec<u8>,
        hotkey: T::AccountId, 
        coldkey: T::AccountId 
    ) -> dispatch::DispatchResult {

        // --- Check the callers hotkey signature.
        ensure_signed(origin)?;

        // --- Check that registrations per block and hotkey.
        let registrations_this_block: u64 = Self::get_registrations_this_block();
        ensure! ( registrations_this_block < Self::get_max_registratations_per_block(), Error::<T>::ToManyRegistrationsThisBlock ); // Number of registrations this block exceeded.
        ensure!( !Hotkeys::<T>::contains_key(&hotkey), Error::<T>::AlreadyRegistered );  // Hotkey has already registered.

        // --- Check block number validity.
        let current_block_number: u64 = Self::get_current_block_as_u64_here();
        ensure! ( block_number <= current_block_number, Error::<T>::InvalidWorkBlock ); // Can't work on future block.
        ensure! ( current_block_number - block_number < 3, Error::<T>::InvalidWorkBlock ); // Work must have been done within 3 blocks (stops long range attacks).

        // --- Check for repeat work,
        ensure!( !UsedWork::<T>::contains_key( &work.clone() ), Error::<T>::WorkRepeated );  // Work has not been used before.

        // --- Check difficulty.
        let difficulty: U256 = Self::get_difficulty();
        let work_hash: H256 = Self::vec_to_hash( work.clone() );
        ensure! ( Self::hash_meets_difficulty( &work_hash, difficulty ), Error::<T>::InvalidDifficulty ); // Check that the work meets difficulty.

        // --- Check work.
        let seal: H256 = Self::create_seal_hash( block_number, nonce );
        ensure! ( seal == work_hash, Error::<T>::InvalidSeal ); // Check that this work matches hash and nonce.
        
        // Check that the hotkey has not already been registered.
        ensure!( !Hotkeys::<T>::contains_key(&hotkey), Error::<T>::AlreadyRegistered );
        
        // Above this line all relevant checks that the registration is legitimate have been met. 
        // --- registration does not exceed limit.
        // --- registration meets difficulty.
        // --- registration is not a duplicate.
        // Next we will check to see if the uid limit has been reached.
        // If we have reached our limit we need to find a replacement. 
        // The replacement peer is the peer with the lowest replacement score.
        let uid_to_set_in_metagraph: u32; // To be filled, we either are prunning or setting with get_next_uid.
        let max_allowed_uids: u64 = Self::get_max_allowed_uids(); // Get uid limit.
        let neuron_count: u64 = Self::get_neuron_count() as u64; // Current number of uids.
        let current_block: u64 = Self::get_current_block_as_u64();
        let immunity_period: u64 = Self::get_immunity_period(); // Num blocks uid cannot be pruned since registration.
        if neuron_count < max_allowed_uids {
            // --- The metagraph is not full and we simply increment the uid.
            uid_to_set_in_metagraph = Self::get_next_uid();
        } else {
            // TODO( const ): this should be a function and we should be able to purge peers down to a set number.
            // We iterate over neurons in memory and find min score.
            // Pruning score values have already been computed at the previous mechanism step.
            let mut uid_to_prune: u32 = 0; // To be filled. Default to zero but will certainly be filled.
            let mut min_prunning_score: I65F63 = I65F63::from_num( u64::MAX ); // Start min score as max.
            for ( uid_i, neuron_i ) in <Neurons<T> as IterableStorageMap<u32, NeuronMetadataOf<T>>>::iter() {

                // Compute the neuron prunning score.
                // The prunning score is given by max( stake_proportion, incentive_proportion )
                // This allows users to buy their way into the network by holding more stake than 
                // the min incentive proportion. 
                // Calculate stake proportion with zero check.        
                let mut stake_proportion: I65F63;
                if Self::get_total_stake() == 0 {
                    stake_proportion = I65F63::from_num( u64::min_value() );
                } else {
                    stake_proportion = I65F63::from_num( neuron_i.stake ) / I65F63::from_num( Self::get_total_stake() ); // Stake proportion (0, 1)
                }
                let mut incentive_proportion: I65F63 = I65F63::from_num( neuron_i.incentive ) / I65F63::from_num( u64::MAX ); // Incentive proportion (0, 1)

                // Multiply through proportions, this is how we weight between different components.
                stake_proportion = stake_proportion * I65F63::from_num( 1 / Self::get_stake_pruning_denominator() );
                incentive_proportion = incentive_proportion * I65F63::from_num( 1 / Self::get_incentive_pruning_denominator() );

                // Take max(stake_proportion, incentive_proportion).
                let mut prunning_score;
                if incentive_proportion > stake_proportion {
                    prunning_score = incentive_proportion;
                } else {
                    prunning_score = stake_proportion;
                }
                // Neurons that have registered within an immunity period should not be counted in this pruning
                // unless there are no other peers to prune. This allows new neurons the ability to gain incentive before they are cut. 
                // We use block_at_registration which sets the prunning score above any possible value for stake or incentive.
                // This also preferences later registering peers if we need to tie break.
                let block_at_registration = BlockAtRegistration::<T>::get( uid_i );  // Default value is 0.
                if current_block - block_at_registration < immunity_period { // Check for immunity.
                    // Note that adding block_at_registration to the pruning score give peers who have registered later a better score.
                    prunning_score = prunning_score + I65F63::from_num( block_at_registration ); // Prunning score now on range (0, current_block)
                } 
                // Find the min purnning score. We will remove this peer first. 
                if prunning_score < min_prunning_score {
                    // Update the min
                    uid_to_prune = neuron_i.uid;
                    min_prunning_score = prunning_score;
                }
            }
            // Remember which uid is min so we can replace it in the graph.
            let neuron_to_prune: NeuronMetadataOf<T> = Neurons::<T>::get( uid_to_prune ).unwrap();
            uid_to_set_in_metagraph = neuron_to_prune.uid;
            let hotkey_to_prune = neuron_to_prune.hotkey;

            // Next we will add this prunned peer to NeuronsToPruneAtNextEpoch.
            // We record this set because we need to remove all bonds owned in this uid.
            // neuron.bonds records all bonds this peer owns which will be removed by default. 
            // However there are other peers with bonds in this peer, these need to be cleared as well.
            // NOTE(const): In further iterations it will be beneficial to build bonds as a double
            // iterable set so that deletions become easier. 
            NeuronsToPruneAtNextEpoch::<T>::insert( uid_to_set_in_metagraph, uid_to_set_in_metagraph ); // Subtrate does not contain a set storage item.
            // Finally, we need to unstake all the funds that this peer had staked. 
            // These funds are deposited back into the coldkey account so that no funds are destroyed. 
            let stake_to_be_added_on_coldkey = Self::u64_to_balance( neuron_to_prune.stake );
            Self::add_balance_to_coldkey_account( &neuron_to_prune.coldkey, stake_to_be_added_on_coldkey.unwrap() );
            Self::decrease_total_stake( neuron_to_prune.stake );

            // Remove hotkey from hotkeys set, 
            // and to clean up and prune whatever extra hotkeys there are on top of the existing max_allowed_uids
            if Hotkeys::<T>::contains_key(&hotkey_to_prune) {
                Hotkeys::<T>::remove( hotkey_to_prune );
            }
        }

        // --- Next we create a new entry in the table with the new metadata.
        let neuron = NeuronMetadataOf::<T> {
            version: 0,
            ip: 0,
            port: 0,
            ip_type: 0,
            uid: uid_to_set_in_metagraph,
            modality: 0,
            hotkey: hotkey.clone(),
            coldkey: coldkey.clone(),
            active: 1,
            last_update: current_block, 
            priority: 0,
            stake: 0,
            rank: 0,
            trust: 0,
            consensus: 0,
            incentive: 0,
            emission: 0,
            dividends: 0,
            bonds: vec![],
            weights: vec![(uid_to_set_in_metagraph, u32::MAX)], // self weight set to 1.
        };

        // --- Update avg registrations per 1000 block.
        RegistrationsThisInterval::<T>::mutate( |val| *val += 1 );
        RegistrationsThisBlock::<T>::mutate( |val| *val += 1 );

        // --- We deposit the neuron registered event.
        BlockAtRegistration::<T>::insert( uid_to_set_in_metagraph, current_block ); // Set immunity momment.
        Neurons::<T>::insert( uid_to_set_in_metagraph, neuron ); // Insert neuron info under uid.
        Hotkeys::<T>::insert( &hotkey, uid_to_set_in_metagraph ); // Add hotkey into hotkey set.
        UsedWork::<T>::insert( &work.clone(), current_block ); // Add the work to current + block. So we can prune at a later date.
        Self::deposit_event(Event::NeuronRegistered( uid_to_set_in_metagraph ));

        Ok(())
    }
}
