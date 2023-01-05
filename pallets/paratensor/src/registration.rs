use super::*;
use frame_support::{ pallet_prelude::DispatchResult};
use sp_std::convert::TryInto;
use sp_core::{H256, U256};
use crate::system::ensure_root;
use sp_io::hashing::sha2_256;
use sp_io::hashing::keccak_256;
use frame_system::{ensure_signed};
use sp_std::vec::Vec;
use frame_support::storage::IterableStorageDoubleMap;
use substrate_fixed::types::I32F32;

const LOG_TARGET: &'static str = "runtime::paratensor::registration";

impl<T: Config> Pallet<T> {

    /// ---- The implementation for the extrinsic do_registration.
    ///
    /// # Args:
    /// 	* 'origin': (<T as frame_system::Config>Origin):
    /// 		- The signature of the calling hotkey.
    ///
    /// 	* 'netuid' (u16):
    /// 		- The u16 network identifier.
    ///
    /// 	* 'block_number' ( u64 ):
    /// 		- Block hash used to prove work done.
    ///
    /// 	* 'nonce' ( u64 ):
    /// 		- Positive integer nonce used in POW.
    ///
    /// 	* 'work' ( Vec<u8> ):
    /// 		- Vector encoded bytes representing work done.
    ///
    /// 	* 'hotkey' ( T::AccountId ):
    /// 		- Hotkey to be registered to the network.
    ///
    /// 	* 'coldkey' ( T::AccountId ):
    /// 		- Associated coldkey account.
    ///
    /// # Event:
    /// 	* NeuronRegistered;
    /// 		- On successfully registereing a uid to a neuron slot on a subnetwork.
    ///
    /// # Raises:
    /// 	* 'NetworkDoesNotExist':
    /// 		- Attempting to registed to a non existent network.
    ///
    /// 	* 'TooManyRegistrationsThisBlock':
    /// 		- This registration exceeds the total allowed on this network this block.
    ///
    /// 	* 'AlreadyRegistered':
    /// 		- The hotkey is already registered on this network.
    ///
    /// 	* 'InvalidWorkBlock':
    /// 		- The work has been performed on a stale, future, or non existent block.
    ///
    /// 	* 'WorkRepeated':
    /// 		- This work for block has already been used.
    ///
    /// 	* 'InvalidDifficulty':
    /// 		- The work does not match the difficutly.
    ///
    /// 	* 'InvalidSeal':
    /// 		- The seal is incorrect.
    ///
    pub fn do_registration( 
        origin: T::Origin,
        netuid: u16, 
        block_number: u64, 
        nonce: u64, 
        work: Vec<u8>,
        hotkey: T::AccountId, 
        coldkey: T::AccountId 
    ) -> DispatchResult {

        // --- 1. Check that the caller has signed the transaction. 
        // TODO( const ): This not be the hotkey signature or else an exterior actor can register the hotkey and potentially control it?
        let signing_origin = ensure_signed( origin )?;        
        log::info!("do_registration( origin:{:?} netuid:{:?} hotkey:{:?}, coldkey:{:?} )", signing_origin, netuid, hotkey, coldkey );

        // --- 2. Ensure the passed network is valid.
        ensure!( Self::if_subnet_exist( netuid ), Error::<T>::NetworkDoesNotExist ); 

        // --- 3. Ensure we are not exceeding the max allowed registrations per block.
        ensure!( Self::get_registrations_this_block( netuid ) < Self::get_max_registratations_per_block( netuid ), Error::<T>::TooManyRegistrationsThisBlock );

        // --- 4. Ensure that the key is not already registered.
        ensure!( !Uids::<T>::contains_key( netuid, &hotkey ), Error::<T>::AlreadyRegistered );

        // --- 5. Ensure the passed block number is valid, not in the future or too old.
        // Work must have been done within 3 blocks (stops long range attacks).
        let current_block_number: u64 = Self::get_current_block_as_u64();
        ensure! (block_number <= current_block_number, Error::<T>::InvalidWorkBlock);
        ensure! (current_block_number - block_number < 3, Error::<T>::InvalidWorkBlock ); 

        // --- 6. Ensure the passed work has not already been used.
        ensure!( !UsedWork::<T>::contains_key( &work.clone() ), Error::<T>::WorkRepeated ); 

        // --- 7. Ensure the supplied work passes the difficulty.
        let difficulty: U256 = Self::get_difficulty( netuid );
        let work_hash: H256 = Self::vec_to_hash( work.clone() );
        ensure! ( Self::hash_meets_difficulty( &work_hash, difficulty ), Error::<T>::InvalidDifficulty ); // Check that the work meets difficulty.
        
        // --- 8. Check Work is the product of the nonce and the block number. Add this as used work.
        let seal: H256 = Self::create_seal_hash( block_number, nonce );
        ensure! ( seal == work_hash, Error::<T>::InvalidSeal );
        UsedWork::<T>::insert( &work.clone(), current_block_number );

        // --- 9. Ensure that the key passes the registration requirement
        ensure!( Self::passes_network_connection_requirement( netuid, &hotkey ), Error::<T>::DidNotPassConnectedNetworkRequirement );

        // --- 10. If the network account does not exist we will create it here.
        Self::create_account_if_non_existent( &coldkey, &hotkey);         

        // --- 11. Ensure that the pairing is correct.
        ensure!( Self::coldkey_owns_hotkey( &coldkey, &hotkey ), Error::<T>::NonAssociatedColdKey );

        // --- 12. Append neuron or prune it.
        let subnetwork_uid: u16;
        let current_subnetwork_n: u16 = Self::get_subnetwork_n( netuid );
        if current_subnetwork_n < Self::get_max_allowed_uids( netuid ) {

            // --- 12.a No replacement required, the uid appends the subnetwork.
            // We increment the subnetwork count here but not below.
            subnetwork_uid = current_subnetwork_n;
            Self::increment_subnetwork_n( netuid );

        } else {

            // --- 12.b Replacement required.
            // We take the neuron with the lowest pruning score here.
            subnetwork_uid = Self::get_neuron_to_prune( netuid );
            Self::prune_uid_from_subnetwork( netuid, subnetwork_uid );
        }
        
        // --- 13. Sets the neuron information on the network under the specified uid with coldkey and hotkey.
        // The function ensures the the global account is created if not already existent.
        Self::fill_new_neuron_account_in_subnetwork( netuid, subnetwork_uid, &hotkey, current_block_number );

        // --- 14. Record the registration and increment block and interval counters.
        RegistrationsThisInterval::<T>::mutate( netuid, |val| *val += 1 );
        RegistrationsThisBlock::<T>::mutate( netuid, |val| *val += 1 );
    
        // --- 15. Deposit successful event.
        log::info!("NeuronRegistered( netuid:{:?} uid:{:?} hotkey:{:?}  ) ", netuid, subnetwork_uid, hotkey );
        Self::deposit_event( Event::NeuronRegistered( netuid, subnetwork_uid, hotkey ) );

        // --- 16. Ok and done.
        Ok(())
    }

    /// --- Checks if the hotkey passes the topk prunning requirement in all connected networks.
    ///
    pub fn passes_network_connection_requirement( netuid_a: u16, hotkey: &T::AccountId ) -> bool {
        // --- 1. We are iterating over all networks to see if there is a registration connection.
        for (netuid_b, exists) in NetworksAdded::<T>::iter() {

            // --- 2. If the network exists and the registration connection requirement exists we will
            // check to see if we pass it.
            if exists && Self::network_connection_requirement_exists( netuid_a, netuid_b ){

                // --- 3. We cant be in the top percentile of an empty network.
                if Self::get_subnetwork_n( netuid_b ) == 0 { return false; }

                // --- 4. First check to see if this hotkey is already registered on this network.
                // If we are not registered we trivially fail the requirement.
                if !Self::is_hotkey_registered_on_network( netuid_b, hotkey ) { return false; }
                let uid_b: u16 = Self::get_uid_for_net_and_hotkey( netuid_b, hotkey ).unwrap();

                // --- 5. Next, count how many keys on the connected network have a better prunning score than
                // our target network.
                let mut n_better_prunning_scores: u16 = 0;
                let our_prunning_score_b: u16 = PruningScores::<T>::get( netuid_b, uid_b );
                for ( other_uid, other_runing_score_b ) in <PruningScores<T> as IterableStorageDoubleMap<u16, u16, u16 >>::iter_prefix( netuid_b ) {
                    if other_uid != uid_b && other_runing_score_b > our_prunning_score_b { n_better_prunning_scores = n_better_prunning_scores + 1; }
                }

                // --- 6. Using the n_better count we check to see if the target key is in the topk percentile.
                // The percentile is stored in NetworkConnect( netuid_i, netuid_b ) as a u16 normalized value (0, 1), 1 being top 100%.
                let topk_percentile_requirement: I32F32 = I32F32::from_num( Self::get_network_connection_requirement( netuid_a, netuid_b ) ) / I32F32::from_num( u16::MAX );
                let topk_percentile_value: I32F32 = I32F32::from_num( n_better_prunning_scores ) / I32F32::from_num( Self::get_subnetwork_n( netuid_b ) );
                if topk_percentile_value > topk_percentile_requirement { return false }
            }
        }
        // --- 7. If we pass all the active registration requirments we return true allowing the registration to 
        // continue to the normal difficulty check.s
        return true;
    }

    /// Returns true if the items contain duplicates hotkeys.
    ///
    fn has_duplicate_hotkeys(items: &Vec<T::AccountId>) -> bool {
        let mut parsed: Vec<T::AccountId> = Vec::new();
        for item in items {
            if parsed.contains(&item) { return true; }
            parsed.push(item.clone());
        }
        return false;
    }

    /// --- Sets new neuron information on the network under the specified uid with coldkey and hotkey information.
    /// The function ensures the the global account is created if not already existent.
    ///
    pub fn fill_new_neuron_account_in_subnetwork( netuid: u16, uid: u16, hotkey: &T::AccountId, current_block_number: u64 ) {
        log::debug!("fill_new_neuron_account_in_subnetwork( netuid: {:?}, uid: {:?}, hotkey: {:?}, current_block_number: {:?} ) ", netuid, uid, hotkey, current_block_number );
        Active::<T>::insert( netuid, uid, true ); // Set to active by default.
        Keys::<T>::insert( netuid, uid, hotkey.clone() ); // Make hotkey - uid association.
        Uids::<T>::insert( netuid, hotkey.clone(), uid ); // Make uid - hotkey association.
        PruningScores::<T>::insert( netuid, uid, u16::MAX ); // Set to infinite pruning score.
        BlockAtRegistration::<T>::insert( netuid, uid, current_block_number ); // Fill block at registration.
        IsNetworkMember::<T>::insert( hotkey.clone(), netuid, true ); // Fill network owner.
    }

    /// --- Removes a uid from a subnetwork by erasing all its data.
    /// The function sets all terms to default state 0, false, or None.
    ///
    pub fn prune_uid_from_subnetwork( netuid: u16, uid_to_prune: u16 ) {
        let hotkey: T::AccountId = Keys::<T>::get( netuid, uid_to_prune );
        log::debug!("prune_uid_from_subnetwork( netuid: {:?} uid_to_prune: {:?} hotkey: {:?} ) ", netuid, uid_to_prune, hotkey );
        Uids::<T>::remove( netuid, hotkey.clone() );
        IsNetworkMember::<T>::remove( hotkey.clone(), netuid);
        Keys::<T>::remove( netuid, uid_to_prune ); 
        Rank::<T>::remove( netuid, uid_to_prune );
        Trust::<T>::remove( netuid, uid_to_prune );
        Bonds::<T>::remove( netuid, uid_to_prune );
        Active::<T>::remove( netuid, uid_to_prune );
        Weights::<T>::remove( netuid, uid_to_prune );
        Emission::<T>::remove( netuid, uid_to_prune );
        Dividends::<T>::remove( netuid, uid_to_prune );
        Consensus::<T>::remove( netuid, uid_to_prune );
        Incentive::<T>::remove( netuid, uid_to_prune );
        ValidatorPermit::<T>::remove( netuid, uid_to_prune );
        PruningScores::<T>::remove( netuid, uid_to_prune );
    }

    pub fn vec_to_hash( vec_hash: Vec<u8> ) -> H256 {
        let de_ref_hash = &vec_hash; // b: &Vec<u8>
        let de_de_ref_hash: &[u8] = &de_ref_hash; // c: &[u8]
        let real_hash: H256 = H256::from_slice( de_de_ref_hash );
        return real_hash
    }

    /// Determine which peer to prune from the network by finding the element with the lowest pruning score.
    /// This function will always return an element to prune.
    pub fn get_neuron_to_prune(netuid: u16) -> u16 {
        let mut min_block_at_registration: u64 = u64::MAX; // Far Future.
        let mut min_score : u16 = u16::MAX;
        let mut uid_with_min_score = 0;
        for (neuron_uid_i, pruning_score) in <PruningScores<T> as IterableStorageDoubleMap<u16, u16, u16 >>::iter_prefix( netuid ) {
            let block_at_registration: u64 = Self::get_neuron_block_at_registration( netuid, neuron_uid_i );
            if min_score == pruning_score {
                // Break ties with block at registration.
                if min_block_at_registration > block_at_registration{
                    min_score = pruning_score; 
                    min_block_at_registration = block_at_registration;
                    uid_with_min_score = neuron_uid_i;
                }
            }
            // Find min pruning score.
            else if min_score > pruning_score { 
                min_score = pruning_score; 
                min_block_at_registration = block_at_registration;
                uid_with_min_score = neuron_uid_i;
            }
        }
        // We replace the pruning score here with u16 max to ensure that all peers always have a 
        // pruning score. In the event that every peer has been pruned this function will prune
        // the last element in the network continually.
        PruningScores::<T>::insert(netuid, uid_with_min_score, u16::MAX );
        uid_with_min_score
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

    pub fn hash_to_vec( hash: H256 ) -> Vec<u8> {
        let hash_as_bytes: &[u8] = hash.as_bytes();
        let hash_as_vec: Vec<u8> = hash_as_bytes.iter().cloned().collect();
        return hash_as_vec
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

    // Helper function for creating nonce and work.
    pub fn create_work_for_block_number( netuid:u16, block_number: u64, start_nonce: u64 ) -> (u64, Vec<u8>) {
        let difficulty: U256 = Self::get_difficulty(netuid);
        let mut nonce: u64 = start_nonce;
        let mut work: H256 = Self::create_seal_hash( block_number, nonce );
        while !Self::hash_meets_difficulty(&work, difficulty) {
            nonce = nonce + 1;
            work = Self::create_seal_hash( block_number, nonce );
        }
        let vec_work: Vec<u8> = Self::hash_to_vec( work );
        return (nonce, vec_work)
    }

     /// ---- The implementation for the extrinsic bulk_register.
    ///
    /// # Args:
    /// 	* 'origin': (<T as frame_system::Config>Origin):
    /// 		- Must be sudo.
    ///
    /// 	* 'netuid' (u16):
    /// 		- The network to bulk register the hotkeys on. Must exist.
    ///    
    /// 	* 'hotkeys' ( Vec<T::AccountId> ):
    /// 		- Hotkeys to register to the network. Note the hotkeys must be in order of uid.
    ///
    /// 	* 'coldkeys' ( Vec<T::AccountId> ):
    /// 		- Associated coldkeys in order.
    ///
    /// # Event:
    /// 	* BulkNeuronsRegistered;
    /// 		- On successfully registering a bulk of neurons to the network.
    ///
    /// # Raises:
    /// 	* 'NetworkDoesNotExist':
    /// 		- Attempting to registed to a non existent network.
    ///
    ///     * 'WeightVecNotEqualSize':
    ///         - Attempting to register a hot-cold key list of non equal size. 
    ///         - Or the lists do not equal the network size.
    ///
    ///     * 'NonAssociatedColdKey':
    ///         - The hot-cold pair cannot be associated because it already exists. 
    ///
    ///
    pub fn do_bulk_register(
        origin: T::Origin, 
        netuid: u16, 
        hotkeys: Vec<T::AccountId>, 
        coldkeys: Vec<T::AccountId> 
    ) -> DispatchResult {

        // --- 1. Ensure the caller is sudo.
        ensure_root( origin )?;

        // --- 2. Ensure the passed network is valid and exists.
        ensure!( Self::if_subnet_exist( netuid ), Error::<T>::NetworkDoesNotExist ); 

        // --- 3. Ensure the coldkeys match the hotkeys in length.
        ensure!( hotkeys.len() == coldkeys.len(), Error::<T>::WeightVecNotEqualSize ); 

        // --- 4. Ensure the passed hotkeys do not contain duplicates.
        ensure!( !Self::has_duplicate_hotkeys( &hotkeys ), Error::<T>::DuplicateUids );

        // --- 5. Check the network size to hotkey length.
        ensure!( hotkeys.len() as u16 == Self::get_max_allowed_uids( netuid ), Error::<T>::NotSettingEnoughWeights);

        // --- 6. Create all accounts for the passed hot - cold pair.
        for (h, c) in hotkeys.iter().zip( coldkeys.clone() ) {
            // --- 6.1 If the network account does not exist we will create it here.
            Self::create_account_if_non_existent( &c, &h );         

            // --- 6.2 Ensure that the pairing is correct.
            ensure!( Self::coldkey_owns_hotkey( &c, &h ), Error::<T>::NonAssociatedColdKey );
        }

        // --- 7. Fill all the slots and erase the previous owners.
        let current_block_number: u64 = Self::get_current_block_as_u64();
        for (uid_i, new_hotkey) in hotkeys.iter().enumerate() {
            let pruned_hotkey: T::AccountId = Keys::<T>::get( netuid, uid_i as u16 );
            Uids::<T>::remove( netuid, pruned_hotkey.clone() );
            IsNetworkMember::<T>::remove( pruned_hotkey.clone(), netuid);
            Keys::<T>::remove( netuid, uid_i as u16 ); 
            Rank::<T>::remove( netuid, uid_i as u16 );
            Trust::<T>::remove( netuid, uid_i as u16 );
            Bonds::<T>::remove( netuid, uid_i as u16 );
            Active::<T>::remove( netuid, uid_i as u16 );
            Weights::<T>::remove( netuid, uid_i as u16 );
            Emission::<T>::remove( netuid, uid_i as u16 );
            Dividends::<T>::remove( netuid, uid_i as u16 );
            Consensus::<T>::remove( netuid, uid_i as u16 );
            Incentive::<T>::remove( netuid, uid_i as u16 );
            PruningScores::<T>::remove( netuid, uid_i as u16 );
            Active::<T>::insert( netuid, uid_i as u16, true ); // Set to active by default.
            Keys::<T>::insert( netuid, uid_i as u16, new_hotkey.clone() ); // Make hotkey - uid association.
            Uids::<T>::insert( netuid, new_hotkey.clone(), uid_i as u16 ); // Make uid - hotkey association.
            IsNetworkMember::<T>::insert( new_hotkey.clone(), netuid, true ); // Fill network owner.
            PruningScores::<T>::insert( netuid, uid_i as u16, u16::MAX ); // Set to infinite pruning score.
            BlockAtRegistration::<T>::insert( netuid, uid_i as u16, current_block_number ); // Fill block at registration.
        }

        // --- 8. Increase subnetwork n to amount of hotkeys.
        // TODO this is wrong.
        SubnetworkN::<T>::insert( netuid, hotkeys.len() as u16 );

        // --- 9. Deposit successful event.
        Self::deposit_event( Event::BulkNeuronsRegistered( netuid, hotkeys.len() as u16 ) );

        // --- 10. Ok and done.
        Ok(())
    }

}