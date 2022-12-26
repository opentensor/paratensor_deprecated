use super::*;
use frame_support::{ pallet_prelude::DispatchResult};
use sp_std::convert::TryInto;
use sp_core::{H256, U256};
use sp_runtime::sp_std::if_std;
use sp_io::hashing::sha2_256;
use sp_io::hashing::keccak_256;
use frame_system::{ensure_signed};
use sp_std::vec::Vec;
use frame_support::storage::IterableStorageDoubleMap;

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
        ensure_signed( origin )?;        
    
        // --- 2. Ensure the passed network is valid.
        ensure!( Self::if_subnet_exist( netuid ), Error::<T>::NetworkDoesNotExist ); 

        // --- 3. Ensure we are not exceeding the max allowed registrations per block.
        ensure!( Self::get_registrations_this_block( netuid ) < Self::get_max_registratations_per_block(), Error::<T>::TooManyRegistrationsThisBlock );

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

        // --- 9. If the network account does not exist we will create it here.
        Self::create_account_if_non_existent( &hotkey, &coldkey );         

        // --- 10. Ensure that the pairing is correct.
        ensure!( Self::account_belongs_to_coldkey( &hotkey, &coldkey ), Error::<T>::NonAssociatedColdKey );

        // --- 11. Append neuron or prune it.
        let subnetwork_uid: u16;
        let current_subnetwork_n: u16 = Self::get_subnetwork_n( netuid );
        if current_subnetwork_n < Self::get_max_allowed_uids( netuid ) {

            // --- 11.a No replacement required, the uid appends the subnetwork.
            // We increment the subnetwork count here but not below.
            subnetwork_uid = current_subnetwork_n;
            Self::increment_subnetwork_n( netuid );
            
        } else {

            // --- 11.b Replacement required.
            // We take the neuron with the lowest pruning score here.
            subnetwork_uid = Self::get_neuron_to_prune( netuid );
            Self::decrement_subnets_for_hotkey( netuid, &Keys::<T>::get( netuid, subnetwork_uid ) );
            Self::prune_uid_from_subnetwork( subnetwork_uid, netuid );
        }

        
        // --- 12. Sets the neuron information on the network under the specified uid with coldkey and hotkey.
        // The function ensures the the global account is created if not already existent.
        Self::fill_new_neuron_account_in_subnetwork( netuid, subnetwork_uid, &coldkey, &hotkey);

        // --- 13. Record the registration and increment block and interval counters.
        BlockAtRegistration::<T>::insert( netuid, subnetwork_uid, current_block_number );
        RegistrationsThisInterval::<T>::mutate( netuid, |val| *val += 1 );
        RegistrationsThisBlock::<T>::mutate( netuid, |val| *val += 1 );
    
        // --- 14. Deposit successful event.
        Self::deposit_event( Event::NeuronRegistered( subnetwork_uid ) );

        // --- 15. Ok and done.
        Ok(())
    }

    // Sets new neuron information on the network under the specified uid with coldkey and hotkey information.
    // The function ensures the the global account is created if not already existent.
    pub fn fill_new_neuron_account_in_subnetwork( netuid: u16, uid: u16 , coldkey: &T::AccountId, hotkey: &T::AccountId ) {
        NeuronsMetaData::<T>::insert( netuid, uid, NeuronMetadata { version: 0, ip: 0, port: 0, ip_type: 0 } );
        Active::<T>::insert( netuid, uid, true );
        Keys::<T>::insert( netuid, uid, hotkey.clone() ); 
        Uids::<T>::insert( netuid, hotkey.clone(), uid );
        Self::increment_subnets_for_hotkey( netuid, hotkey );
        Self::add_hotkey_stake_for_network( netuid, hotkey );
    }

    // Removes a uid from a subnetwork by erasing all its data.
    // The function sets all terms to default state 0, false, or None.
    pub fn prune_uid_from_subnetwork( netuid: u16, uid_to_prune: u16 ) {
        Uids::<T>::remove( netuid, Keys::<T>::get( netuid, uid_to_prune ).clone() );
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
        PruningScores::<T>::remove( netuid, uid_to_prune );
        NeuronsMetaData::<T>::remove( netuid, uid_to_prune );
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
        let mut min_score : u16 = u16::MAX;
        let mut uid_with_min_score = 0;
        for (uid_i, pruning_score) in <PruningScores<T> as IterableStorageDoubleMap<u16, u16, u16 >>::iter_prefix( netuid ) {
            if min_score > pruning_score { 
                min_score = pruning_score; 
                uid_with_min_score = uid_i;
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
    pub fn add_hotkey_stake_for_network(netuid: u16,  hotkey: &T::AccountId){
        
        let stake = Stake::<T>::get(&hotkey);
        let uid_to_prune ;
        match Self::get_neuron_for_net_and_hotkey(netuid, &hotkey) {
            Ok(k) => uid_to_prune = k,
            Err(e) => panic!("Error: {:?}", e),
        } 
        //
        S::<T>::insert(netuid, uid_to_prune, stake);
    }

}