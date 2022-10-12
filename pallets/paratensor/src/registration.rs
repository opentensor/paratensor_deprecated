use super::*;
use substrate_fixed::types::I65F63;
use frame_support::{ pallet_prelude::DispatchResult};
use sp_std::convert::TryInto;
use sp_core::{H256, U256};
use sp_io::hashing::sha2_256;
use sp_io::hashing::keccak_256;
use frame_system::{ensure_signed};
use sp_std::vec::Vec;
use frame_support::storage::IterableStorageDoubleMap;



const LOG_TARGET: &'static str = "runtime::paratensor::registration";

impl<T: Config> Pallet<T> {

    pub fn do_registration ( 
        origin: T::Origin,
        netuid: u16,  //subnetwork id 
        block_number: u64, 
        nonce: u64, 
        work: Vec<u8>,
        hotkey: T::AccountId, 
        coldkey: T::AccountId 
    ) -> DispatchResult {

        // --- Check the callers hotkey signature.
        ensure_signed(origin)?;

        /// TO DO:
        /// 1.  --- Check that registrations per block and hotkey in this network
        /// 2. --- Check block number validity.
        /// 3.  --- Check for repeat work,
        /// 4. --- Check difficulty.
        /// 5. --- Check work.
        /// 6. --- Check that the hotkey has not already been registered in this network.
        /// 7. --- check to see if the uid limit has been reached.
        ///     a. YES: 
        ///         - find a replacement
        ///         - add this prunned peer to NeuronsToPruneAtNextEpoch.
        ///         - unstake all the funds that this peer had staked if node is not registered in any other network
        ///         - update all relevant data structures
        ///     b. NO:
        ///         - increment the uid.
        ///         - create a new entry in the table with the new metadata.
        ///         - update appropriate parameters.
        ///         -  add new neuron to neurons. hotkeys, and works
        //
        // 1. check registration per block 
        let registrations_this_block: u16 = Self:: get_registrations_this_block();
        ensure! (registrations_this_block < Self:: get_max_registratations_per_block(), Error::<T>::ToManyRegistrationsThisBlock); // Number of registrations this block exceeded.
        ensure! (!Uids::<T>::contains_key(&netuid, &hotkey), Error::<T>::AlreadyRegistered); // Hotkey has already registered.

        /* 2. Check block number validity. */
        let current_block_number: u64 = Self::get_current_block_as_u64();
        ensure! (block_number <= current_block_number, Error::<T>::InvalidWorkBlock);
        ensure! (current_block_number - block_number < 3, Error::<T>::InvalidWorkBlock ); // Work must have been done within 3 blocks (stops long range attacks).

        // 3. Check for repeat work,
        ensure!( !UsedWork::<T>::contains_key( &work.clone() ), Error::<T>::WorkRepeated );  // Work has not been used before.

        // 4. Check difficulty.
        let difficulty: U256 = Self::get_difficulty(netuid);
        let work_hash: H256 = Self::vec_to_hash( work.clone() );
        ensure! ( Self::hash_meets_difficulty( &work_hash, difficulty ), Error::<T>::InvalidDifficulty ); // Check that the work meets difficulty.
        
        // 5. Check Work.
        let seal: H256 = Self::create_seal_hash( block_number, nonce );
        ensure! ( seal == work_hash, Error::<T>::InvalidSeal ); // Check that this work matches hash and nonce.
        
        // 6. Check that the hotkey has not already been registered.
        ensure! (!Uids::<T>::contains_key(netuid, &hotkey), Error::<T>::AlreadyRegistered); // Hotkey has already registered.
        
        // 7. check to see if the uid limit has been reached.
        let uid_to_set_in_metagraph: u16; // To be filled, we either are prunning or setting with get_next_uid.
        let max_allowed_uids: u16 = Self::get_max_allowed_uids(netuid); // Get uid limit.
        let neuron_count: u16 = Self::get_neuron_count(netuid); // Current number of uids for netuid network.
        let current_block: u64 = Self::get_current_block_as_u64();
        //let immunity_period: u16 = Self::get_immunity_period(netuid); // Num blocks uid cannot be pruned since registration.
        if neuron_count < max_allowed_uids {

            // 7.b. NO:  The metagraph is not full and we simply increment the uid.
            uid_to_set_in_metagraph = Self::get_next_uid();
        } else {
            // 7.a. YES:
                // - compute the pruning score
            let uid_to_prune: u16 = Self::find_neuron_to_prune(netuid); // uid_to_prune is the index (neuron UID)
            uid_to_set_in_metagraph = uid_to_prune;
            let hotkey_to_prune = Keys::<T>::get(netuid, uid_to_prune);
            /* TO DO: function to check if the hotkey is deregistred from all networks, 
            if so, then we need to transfer stake from hotkey to cold key */
            let vec_subnets_for_pruning_hotkey: Vec<u16> = Subnets::<T>::get(&hotkey_to_prune); // a list of subnets that hotkey is registered on.
            if vec_subnets_for_pruning_hotkey.len() == 1 {
                if vec_subnets_for_pruning_hotkey[0] == netuid {
                    // we need to remove all stakes since this hotkey is not staked in any other networks
                    // These funds are deposited back into the coldkey account so that no funds are destroyed. 
                    let stake_to_remove: u64 = S::<T>::take(netuid, uid_to_prune);
                    //
                    let coldkey_to_add_stake = Coldkeys::<T>::get(&hotkey_to_prune);
                    Self::add_balance_to_coldkey_account( &coldkey_to_add_stake, Self::u64_to_balance(stake_to_remove).unwrap());
                    Self::decrease_total_stake( stake_to_remove );
                    //
                    if Stake::<T>::contains_key(&hotkey_to_prune){
                        Stake::<T>::remove(&hotkey_to_prune);
                    }
                    if Keys::<T>::contains_key(netuid, uid_to_prune){
                        Keys::<T>::remove(netuid, uid_to_prune);
                    }
                    if Uids::<T>::contains_key(netuid, &hotkey_to_prune){
                        Uids::<T>::remove(netuid, &hotkey_to_prune);
                    }
                    if Hotkeys::<T>::contains_key(&hotkey_to_prune) {
                        Hotkeys::<T>::remove( &hotkey_to_prune );
                    }
                    if Coldkeys::<T>::contains_key(&coldkey_to_add_stake){
                        Coldkeys::<T>::remove(&coldkey_to_add_stake);
                    }
                    Subnets::<T>::remove(&hotkey_to_prune);

                } 
            } 
            //
            // Next we will add this prunned peer to NeuronsToPruneAtNextEpoch.
            // We record this set because we need to remove all bonds owned in this uid.
            // neuron.bonds records all bonds this peer owns which will be removed by default. 
            // However there are other peers with bonds in this peer, these need to be cleared as well. 
            NeuronsToPruneAtNextEpoch::<T>::insert( netuid, uid_to_prune ); // Subtrate does not contain a set storage item.
        }
        
        // next, we add new registered node to all structures
        BlockAtRegistration::<T>::insert( uid_to_set_in_metagraph, current_block ); // Set immunity momment. 
        SubnetworkN::<T>::mutate(netuid, |val| *val += 1 ); //increment the number of registered uids i the subnet
        Hotkeys::<T>::insert(&hotkey, &coldkey);
        Coldkeys::<T>::insert(&coldkey, &hotkey);
        if Subnets::<T>::contains_key(&hotkey){ //update the list of subnets that hotkey is registered on
            let mut vec_new_hotkey_subnets: Vec<u16> = Subnets::<T>::take(&hotkey);
            //Subnets::<T>::remove(&hotkey);
            vec_new_hotkey_subnets.push(netuid);
            Subnets::<T>::insert(&hotkey, vec_new_hotkey_subnets);
        }
        Keys::<T>::insert(netuid, uid_to_set_in_metagraph, &hotkey);
        Uids::<T>::insert(netuid, &hotkey, uid_to_set_in_metagraph);
        UsedWork::<T>::insert( &work.clone(), current_block ); // Add the work to current + block. So we can prune at a later date.
        // --- Update avg registrations per 1000 block.
        RegistrationsThisInterval::<T>::mutate( netuid, |val| *val += 1 );
        RegistrationsThisBlock::<T>::mutate( |val| *val += 1 );
        //
        Self::deposit_event(Event::NeuronRegistered( uid_to_set_in_metagraph ));
        //
        Ok(())
    }

    pub fn get_current_block_as_u64_here( ) -> u64 {
        let block_as_u64: u64 = TryInto::try_into( system::Pallet::<T>::block_number() ).ok().expect("blockchain will not exceed 2^64 blocks; QED.");
        block_as_u64
    }

    pub fn vec_to_hash( vec_hash: Vec<u8> ) -> H256 {
        let de_ref_hash = &vec_hash; // b: &Vec<u8>
        let de_de_ref_hash: &[u8] = &de_ref_hash; // c: &[u8]
        let real_hash: H256 = H256::from_slice( de_de_ref_hash );
        return real_hash
    }

    /// Determine whether the given hash satisfies the given difficulty.
    /// The test is done by multiplying the two together. If the product
    /// overflows the bounds of U256, then the product (and thus the hash)
    /// was too high.
    pub fn hash_meets_difficulty(hash: &H256, difficulty: U256) -> bool {
        let bytes: &[u8] = &hash.as_bytes();
        let num_hash: U256 = U256::from( bytes );
        let (value, overflowed) = num_hash.overflowing_mul(difficulty);

		log::trace!(
			target: LOG_TARGET,
			"Difficulty: hash: {:?}, hash_bytes: {:?}, hash_as_num: {:?}, difficulty: {:?}, value: {:?} overflowed: {:?}",
			hash,
			bytes,
			num_hash,
			difficulty,
			value,
			overflowed
		);
        !overflowed
    }

    pub fn get_block_hash_from_u64 ( block_number: u64 ) -> H256 {
        let block_number: T::BlockNumber = TryInto::<T::BlockNumber>::try_into( block_number ).ok().expect("convert u64 to block number.");
        let block_hash_at_number: <T as frame_system::Config>::Hash = system::Pallet::<T>::block_hash( block_number );
        let vec_hash: Vec<u8> = block_hash_at_number.as_ref().into_iter().cloned().collect();
        let deref_vec_hash: &[u8] = &vec_hash; // c: &[u8]
        let real_hash: H256 = H256::from_slice( deref_vec_hash );

        log::trace!(
			target: LOG_TARGET,
			"block_number: {:?}, vec_hash: {:?}, real_hash: {:?}",
			block_number,
			vec_hash,
			real_hash
		);

        return real_hash;
    }

    pub fn create_seal_hash( block_number_u64: u64, nonce_u64: u64 ) -> H256 {
        let nonce = U256::from( nonce_u64 );
        let block_hash_at_number: H256 = Self::get_block_hash_from_u64( block_number_u64 );
        let block_hash_bytes: &[u8] = block_hash_at_number.as_bytes();
        let full_bytes: &[u8; 40] = &[
            nonce.byte(0),  nonce.byte(1),  nonce.byte(2),  nonce.byte(3),
            nonce.byte(4),  nonce.byte(5),  nonce.byte(6),  nonce.byte(7),

            block_hash_bytes[0], block_hash_bytes[1], block_hash_bytes[2], block_hash_bytes[3],
            block_hash_bytes[4], block_hash_bytes[5], block_hash_bytes[6], block_hash_bytes[7],
            block_hash_bytes[8], block_hash_bytes[9], block_hash_bytes[10], block_hash_bytes[11],
            block_hash_bytes[12], block_hash_bytes[13], block_hash_bytes[14], block_hash_bytes[15],

            block_hash_bytes[16], block_hash_bytes[17], block_hash_bytes[18], block_hash_bytes[19],
            block_hash_bytes[20], block_hash_bytes[21], block_hash_bytes[22], block_hash_bytes[23],
            block_hash_bytes[24], block_hash_bytes[25], block_hash_bytes[26], block_hash_bytes[27],
            block_hash_bytes[28], block_hash_bytes[29], block_hash_bytes[30], block_hash_bytes[31],
        ];
        let sha256_seal_hash_vec: [u8; 32] = sha2_256( full_bytes );
        let keccak_256_seal_hash_vec: [u8; 32] = keccak_256( &sha256_seal_hash_vec );
        let seal_hash: H256 = H256::from_slice( &keccak_256_seal_hash_vec );

		 log::trace!(
			"\nblock_number: {:?}, \nnonce_u64: {:?}, \nblock_hash: {:?}, \nfull_bytes: {:?}, \nsha256_seal_hash_vec: {:?},  \nkeccak_256_seal_hash_vec: {:?}, \nseal_hash: {:?}",
			block_number_u64,
			nonce_u64,
			block_hash_at_number,
			full_bytes,
			sha256_seal_hash_vec,
            keccak_256_seal_hash_vec,
			seal_hash
		);

        return seal_hash;
    }
    /// --- calculate pruning score and find the neuron that should be prunned in the next epoch
    pub fn find_neuron_to_prune(netuid: u16) -> u16 {
        let mut uid_to_prune: u16 = 0; // To be filled. Default to zero but will certainly be filled.
        let mut min_prunning_score: I65F63 = I65F63::from_num( u64::MAX ); // Start min score as max.
        let current_block: u64 = Self::get_current_block_as_u64();
        let immunity_period: u64 = Self::get_immunity_period(netuid) as u64; // Num blocks uid cannot be pruned since registration.
        //
        for (uid_i, stake_i) in <S<T> as IterableStorageDoubleMap<u16, u16, u64>>::iter_prefix(netuid) {
            //
            let incentive : u16 = Incentive::<T>::get(netuid, uid_i);
            // If a neuron has more than stake_pruning_min they are ranked based on stake
            // otherwise we prune based on incentive.   
            let mut prunning_score: I65F63;
            if stake_i >= Self::get_stake_pruning_min(netuid) as u64 {
                if Self::get_total_stake() > 0 { //in case stake pruning min == 0
                    prunning_score = I65F63::from_num( stake_i) / I65F63::from_num( Self::get_total_stake() );
                } else {
                    prunning_score = I65F63::from_num( 0 );
                }
            }   else {
                        prunning_score = I65F63::from_num( incentive ) / I65F63::from_num( u64::MAX );
            } 
            // Neurons that have registered within an immunity period should not be counted in this pruning
            // unless there are no other peers to prune. This allows new neurons the ability to gain incentive before they are cut. 
            // We use block_at_registration which sets the prunning score above any possible value for stake or incentive.
            // This also preferences later registering peers if we need to tie break.
            let block_at_registration = BlockAtRegistration::<T>::get( netuid);  // Default value is 0.
            if current_block - block_at_registration < immunity_period { // Check for immunity.
                // Note that adding block_at_registration to the pruning score give peers who have registered later a better score.
                prunning_score = prunning_score + I65F63::from_num( block_at_registration + 1); // Prunning score now on range (0, current_block)
            } 
            // Find the min purnning score. We will remove this peer first. 
            if prunning_score < min_prunning_score {
                // Update the min
                uid_to_prune = uid_i;
                min_prunning_score = prunning_score;
            }
        }
        uid_to_prune
    }

}