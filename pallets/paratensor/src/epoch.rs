use super::*;
use frame_support::sp_std::vec;
use frame_support::inherent::Vec;
use substrate_fixed::transcendental::exp;
use substrate_fixed::types::{I32F32, I64F64, I96F32};
use frame_support::storage::IterableStorageDoubleMap;

impl<T: Config> Pallet<T> {

    /// Calculates reward consensus and returns the emissions for uids/hotkeys in a given `netuid`.
    /// (Dense version used only for testing purposes.)
    pub fn epoch_dense( netuid: u16, rao_emission: u64 ) -> Vec<u64> {
  
        // Get subnetwork size.
        let n: u16 = Self::get_subnetwork_n( netuid );
        log::debug!( "n:\n{:?}\n", n );

        // ======================
        // == Active & updated ==
        // ======================

        // Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        log::debug!( "current_block:\n{:?}\n", current_block );

        // Get activity cutoff.
        let activity_cutoff: u64 = Self::get_activity_cutoff( netuid ) as u64;
        log::debug!( "activity_cutoff:\n{:?}\n", activity_cutoff );

        // Last update vector.
        let last_update: Vec<u64> = Self::get_last_update( netuid );
        log::debug!( "Last update:\n{:?}\n", last_update.clone() );

        // Inactive mask.
        let inactive: Vec<bool> = last_update.iter().map(| updated | *updated + activity_cutoff < current_block ).collect();
        log::debug!( "Inactive:\n{:?}\n", inactive.clone() );

        // Block at registration vector (block when each neuron was most recently registered).
        let block_at_registration: Vec<u64> = Self::get_block_at_registration( netuid );
        log::debug!( "Block at registration:\n{:?}\n", block_at_registration.clone() );

        // Outdated matrix, updated_ij=True if i has last updated (weights) after j has last registered.
        let outdated: Vec<Vec<bool>> = last_update.iter().map(| updated | block_at_registration.iter().map(| registered | updated <= registered ).collect() ).collect();
        log::debug!( "Outdated:\n{:?}\n", outdated.clone() );

        // ===========
        // == Stake ==
        // ===========

        // Access network stake as normalized vector.
        let stake: Vec<I32F32> = Self::get_normalized_stake( netuid );

        // Remove inactive stake.
        let mut active_stake: Vec<I32F32> = stake.clone();
        inplace_mask_vector( &inactive, &mut active_stake );

        // Normalize active stake.
        inplace_normalize( &mut active_stake );
        log::debug!( "S:\n{:?}\n", active_stake.clone() );

        // =======================
        // == Validator permits ==
        // =======================

        // Get validator permits.
        let validator_permits: Vec<bool> = Self::get_validator_permits( netuid );
        log::debug!( "validator_permits: {:?}", validator_permits );

        // Logical negation of validator_permits.
        let validator_forbids: Vec<bool> = validator_permits.iter().map(|&b| !b).collect();

        // Get max allowed validators.
        let max_allowed_validators: u16 = Self::get_max_allowed_validators( netuid );
        log::debug!( "max_allowed_validators: {:?}", max_allowed_validators );

        // Get new validator permits.
        let new_validator_permits: Vec<bool> = is_topk( &stake, max_allowed_validators as usize );
        log::debug!( "new_validator_permits: {:?}", new_validator_permits );

        // =============
        // == Weights ==
        // =============

        // Access network weights row normalized.
        let mut weights: Vec<Vec<I32F32>> = Self::get_weights( netuid );
        log::debug!( "W:\n{:?}\n", weights.clone() );

        // Mask weights that are not from permitted validators.
        inplace_mask_rows( &validator_forbids, &mut weights );
        // log::debug!( "W (permit): {:?}", weights.clone() );

        // Remove self-weight by masking diagonal.
        inplace_mask_diag( &mut weights );
        // log::debug!( "W (permit+diag):\n{:?}\n", weights.clone() );

        // Mask outdated weights: remove weights referring to deregistered neurons.
        inplace_mask_matrix( &outdated, &mut weights );
        // log::debug!( "W (permit+diag+outdate):\n{:?}\n", weights.clone() );

        // Normalize remaining weights.
        inplace_row_normalize( &mut weights );
        // log::debug!( "W (mask+norm):\n{:?}\n", weights.clone() );

        // ========================================
        // == Ranks, Trust, Consensus, Incentive ==
        // ========================================

        // Compute ranks: r_j = SUM(i) w_ij * s_i
        let mut ranks: Vec<I32F32> = matmul( &weights, &active_stake );
        inplace_normalize( &mut ranks );
        log::debug!( "R:\n{:?}\n", ranks.clone() );

        // Compute thresholded weights.
        let upper: I32F32 = I32F32::from_num( 1.0 );
        let lower: I32F32 = I32F32::from_num( 0.0 );
        let threshold: I32F32 = I32F32::from_num( 0.1 ) / I32F32::from_num( n + 1 );
        let clipped_weights: Vec<Vec<I32F32>> = clip( &weights, threshold, upper, lower );
        // log::debug!( "tW:\n{:?}\n", clipped_weights.clone() );

        // Compute trust scores: t_j = SUM(i) w_ij * s_i
        let trust: Vec<I32F32> = matmul( &clipped_weights, &active_stake );
        log::debug!( "T:\n{:?}\n", trust.clone() );

        // Compute consensus.
        let rho: I32F32 = Self::get_float_rho( netuid );
        let kappa: I32F32 = Self::get_float_kappa( netuid );
        let consensus: Vec<I32F32> = trust.iter().map(|t: &I32F32| sigmoid_safe(*t, rho, kappa)).collect();
        log::debug!( "C:\n{:?}\n", consensus.clone() );

        // Compute incentive.
        let mut incentive: Vec<I32F32> = ranks.iter().zip( consensus.clone() ).map( |(ri, ci)| ri * ci ).collect();
        inplace_normalize( &mut incentive );
        log::debug!( "I:\n{:?}\n", incentive.clone() );

        // =========================
        // == Bonds and Dividends ==
        // =========================

        // Access network bonds column normalized.
        let mut bonds: Vec<Vec<I32F32>> = Self::get_bonds( netuid );
        inplace_mask_matrix( &outdated, &mut bonds );  // mask outdated bonds
        inplace_col_normalize( &mut bonds ); // sum_i b_ij = 1
        // log::debug!( "B:\n{:?}\n", bonds.clone() );        

        // Compute bonds delta column normalized.
        let mut bonds_delta: Vec<Vec<I32F32>> = hadamard( &weights, &active_stake ); // ΔB = W◦S
        inplace_col_normalize( &mut bonds_delta ); // sum_i b_ij = 1
        // log::debug!( "ΔB:\n{:?}\n", bonds_delta.clone() );
    
        // Compute bonds moving average.
        let alpha: I32F32 = I32F32::from_num( 0.1 );
        let mut ema_bonds: Vec<Vec<I32F32>> = mat_ema( &bonds_delta, &bonds, alpha );
        inplace_col_normalize( &mut ema_bonds ); // sum_i b_ij = 1
        // log::debug!( "emaB:\n{:?}\n", ema_bonds.clone() );

        // Compute dividends: d_i = SUM(j) b_ij * inc_j
        let mut dividends: Vec<I32F32> = matmul_transpose( &ema_bonds, &incentive );
        inplace_normalize( &mut dividends );
        log::debug!( "D:\n{:?}\n", dividends.clone() );

        // =================================
        // == Emission and Pruning scores ==
        // =================================

        // Compute emission scores.
        let mut normalized_emission: Vec<I32F32> = incentive.iter().zip( dividends.clone() ).map( |(ii, di)| ii + di ).collect();
        inplace_normalize( &mut normalized_emission );
        
        // If emission is zero, replace emission with normalized stake.
        if is_zero( &normalized_emission ) { // no weights set | outdated weights | self_weights
            if is_zero( &active_stake ) { // no active stake
                normalized_emission = stake.clone(); // do not mask inactive, assumes stake is normalized
            }
            else {
                normalized_emission = active_stake.clone(); // emission proportional to inactive-masked normalized stake
            }
        }

        // Compute rao based emission scores. range: I96F32(0, rao_emission)
        let float_rao_emission: I96F32 = I96F32::from_num( rao_emission );
        let emission: Vec<I96F32> = normalized_emission.iter().map( |e: &I32F32| I96F32::from_num( *e ) * float_rao_emission ).collect();
        let emission: Vec<u64> = emission.iter().map( |e: &I96F32| e.to_num::<u64>() ).collect();
        log::debug!( "E: {:?}", emission.clone() );

        // Set pruning scores.
        let pruning_scores: Vec<I32F32> = normalized_emission.clone();
        log::debug!( "P: {:?}", pruning_scores.clone() );

        // ===================
        // == Value storage ==
        // ===================

        // Sync parameter updates.
        for i in 0..n {
            Self::set_rank( netuid, i, fixed_proportion_to_u16( ranks[i as usize] ) );
            Self::set_trust( netuid, i, fixed_proportion_to_u16( trust[i as usize] ) );
            Self::set_consensus( netuid, i, fixed_proportion_to_u16( consensus[i as usize] ) );
            Self::set_incentive( netuid, i, fixed_proportion_to_u16( incentive[i as usize] ) );
            Self::set_dividend( netuid, i, fixed_proportion_to_u16( dividends[i as usize] ) );
            Self::set_pruning_score( netuid, i, fixed_proportion_to_u16( pruning_scores[i as usize] ) );
            Self::set_emission( netuid, i, emission[i as usize] );
            Self::set_validator_permit( netuid, i, new_validator_permits[i as usize] );
            
            // Set bonds only if uid retains validator permit, otherwise clear bonds.
            if new_validator_permits[i as usize] {
                Self::set_bonds( netuid, i, (0..n).zip( vec_fixed_proportions_to_u16( ema_bonds[i as usize].clone() ) ).collect() );
            }
            else {
                Self::set_bonds( netuid, i, vec![] );
            }
        }  

        // Returning the tao emission here which will be distributed at a higher level.
        emission
    }

    /// Calculates reward consensus values, then updates rank, trust, consensus, incentive, dividend, pruning_score, emission and bonds, and 
    /// returns the emissions for uids/hotkeys in a given `netuid`.
    ///
    /// # Args:
    /// 	* 'netuid': ( u16 ):
    ///         - The network to distribute the emission onto.
    /// 		
    /// 	* 'rao_emission': ( u64 ):
    ///         - The total emission for the epoch.
    ///
    /// 	* 'debug' ( bool ):
    /// 		- Print debugging outputs.
    ///    
    pub fn epoch( netuid: u16, rao_emission: u64 ) -> Vec<u64> {
        // Get subnetwork size.
        let n: u16 = Self::get_subnetwork_n( netuid );
        log::debug!( "n: {:?}", n );

        // ======================
        // == Active & updated ==
        // ======================

        // Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        log::debug!( "current_block: {:?}", current_block );

        // Get activity cutoff.
        let activity_cutoff: u64 = Self::get_activity_cutoff( netuid ) as u64;
        log::debug!( "activity_cutoff: {:?}", activity_cutoff );

        // Last update vector.
        let last_update: Vec<u64> = Self::get_last_update( netuid );
        log::debug!( "Last update: {:?}", last_update.clone() );

        // Inactive mask.
        let inactive: Vec<bool> = last_update.iter().map(| updated | *updated + activity_cutoff < current_block ).collect();
        log::debug!( "Inactive: {:?}", inactive.clone() );

        // Block at registration vector (block when each neuron was most recently registered).
        let block_at_registration: Vec<u64> = Self::get_block_at_registration( netuid );
        log::debug!( "Block at registration: {:?}", block_at_registration.clone() );

        // ===========
        // == Stake ==
        // ===========

        // Access network stake as normalized vector.
        let stake: Vec<I32F32> = Self::get_normalized_stake( netuid );
        // range: I32F32(0, 1)
        log::debug!( "S: {:?}", stake.clone() );

        // Remove inactive stake.
        let mut active_stake: Vec<I32F32> = stake.clone();
        inplace_mask_vector( &inactive, &mut active_stake );
        log::debug!( "S (mask): {:?}", active_stake.clone() );

        // Normalize active stake.
        inplace_normalize( &mut active_stake );
        log::debug!( "S (mask+norm): {:?}", active_stake.clone() );

        // =======================
        // == Validator permits ==
        // =======================

        // Get current validator permits.
        let validator_permits: Vec<bool> = Self::get_validator_permits( netuid );
        log::debug!( "validator_permits: {:?}", validator_permits );

        // Logical negation of validator_permits.
        let validator_forbids: Vec<bool> = validator_permits.iter().map(|&b| !b).collect();

        // Get max allowed validators.
        let max_allowed_validators: u16 = Self::get_max_allowed_validators( netuid );
        log::debug!( "max_allowed_validators: {:?}", max_allowed_validators );

        // Get new validator permits.
        let new_validator_permits: Vec<bool> = is_topk( &stake, max_allowed_validators as usize );
        log::debug!( "new_validator_permits: {:?}", new_validator_permits );

        // =============
        // == Weights ==
        // =============

        // Access network weights row normalized.
        let mut weights: Vec<Vec<(u16, I32F32)>> = Self::get_weights_sparse( netuid );
        // log::debug!( "W: {:?}", weights.clone() );

        // Mask weights that are not from permitted validators.
        weights = mask_rows_sparse( &validator_forbids, &weights );
        // log::debug!( "W (permit): {:?}", weights.clone() );

        // Remove self-weight by masking diagonal.
        weights = mask_diag_sparse( &weights );
        // log::debug!( "W (permit+diag): {:?}", weights.clone() );

        // Remove weights referring to deregistered neurons.
        weights = vec_mask_sparse_matrix( &weights, &last_update, &block_at_registration, &| updated, registered | updated <= registered );
        // log::debug!( "W (permit+diag+outdate): {:?}", weights.clone() );

        // Normalize remaining weights.
        inplace_row_normalize_sparse( &mut weights );
        // log::debug!( "W (mask+norm): {:?}", weights.clone() );

        // ========================================
        // == Ranks, Trust, Consensus, Incentive ==
        // ========================================

        // Compute ranks: r_j = SUM(i) w_ij * s_i.
        // range: I32F32(0, 1)
        let mut ranks: Vec<I32F32> = sparse_matmul( &weights, &active_stake, n );
        inplace_normalize( &mut ranks );
        log::debug!( "R: {:?}", ranks.clone() );

        // Compute thresholded weights.
        // range: I32F32(0, 1)
        let upper: I32F32 = I32F32::from_num( 1.0 );
        let lower: I32F32 = I32F32::from_num( 0.0 );
        let threshold: I32F32 = I32F32::from_num( 0.1 ) / I32F32::from_num( n + 1 );
        let clipped_weights: Vec<Vec<(u16, I32F32)>> = sparse_clip( &weights, threshold, upper, lower );
        // log::debug!( "W (threshold): {:?}", clipped_weights.clone() );

        // Compute trust scores: t_j = SUM(i) w_ij * s_i
        // range: I32F32(0, 1)
        let trust: Vec<I32F32> = sparse_matmul( &clipped_weights, &active_stake, n );
        log::debug!( "T: {:?}", trust.clone() );

        // Compute consensus.
        // range: I32F32(0, 1)
        let rho: I32F32 = Self::get_float_rho( netuid );
        let kappa: I32F32 = Self::get_float_kappa( netuid );
        let consensus: Vec<I32F32> = trust.iter().map(|t: &I32F32| sigmoid_safe(*t, rho, kappa)).collect();
        log::debug!( "C: {:?}", consensus.clone() );

        // Compute incentive.
        // range: I32F32(0, 1)
        let mut incentive: Vec<I32F32> = ranks.iter().zip( consensus.clone() ).map( |(ri, ci)| ri * ci ).collect();
        inplace_normalize( &mut incentive );
        log::debug!( "I: {:?}", incentive.clone() );

        // =========================
        // == Bonds and Dividends ==
        // =========================

        // Access network bonds column normalized.
        let mut bonds: Vec<Vec<(u16, I32F32)>> = Self::get_bonds_sparse( netuid );
        // log::debug!( "B: {:?}", bonds.clone() );
        
        // Remove bonds referring to deregistered neurons.
        bonds = vec_mask_sparse_matrix( &bonds, &last_update, &block_at_registration, &| updated, registered | updated <= registered );
        // log::debug!( "B (outdatedmask): {:?}", bonds.clone() );

        // Normalize remaining bonds: sum_i b_ij = 1.
        inplace_col_normalize_sparse( &mut bonds );
        // log::debug!( "B (mask+norm): {:?}", bonds.clone() );        

        // Compute bonds delta column normalized.
        let mut bonds_delta: Vec<Vec<(u16, I32F32)>> = sparse_hadamard( &weights, &active_stake ); // ΔB = W◦S (outdated W masked)
        // log::debug!( "ΔB: {:?}", bonds_delta.clone() );

        // Normalize bonds delta.
        inplace_col_normalize_sparse( &mut bonds_delta ); // sum_i b_ij = 1
        // log::debug!( "ΔB (norm): {:?}", bonds_delta.clone() );
    
        // Compute bonds moving average.
        let alpha: I32F32 = I32F32::from_num( 0.1 );
        let mut ema_bonds: Vec<Vec<(u16, I32F32)>> = sparse_mat_ema( &bonds_delta, &bonds, alpha );

        // Normalize EMA bonds.
        inplace_col_normalize_sparse( &mut ema_bonds ); // sum_i b_ij = 1
        // log::debug!( "emaB: {:?}", ema_bonds.clone() );

        // Compute dividends: d_i = SUM(j) b_ij * inc_j.
        // range: I32F32(0, 1)
        let mut dividends: Vec<I32F32> = sparse_matmul_transpose( &ema_bonds, &incentive );
        inplace_normalize( &mut dividends );
        log::debug!( "D: {:?}", dividends.clone() );

        // =================================
        // == Emission and Pruning scores ==
        // =================================

        // Compute normalized emission scores. range: I32F32(0, 1)
        let mut normalized_emission: Vec<I32F32> = incentive.iter().zip( dividends.clone() ).map( |(ii, di)| ii + di ).collect();
        inplace_normalize( &mut normalized_emission );

        // If emission is zero, replace emission with normalized stake.
        if is_zero( &normalized_emission ) { // no weights set | outdated weights | self_weights
            if is_zero( &active_stake ) { // no active stake
                normalized_emission = stake.clone(); // do not mask inactive, assumes stake is normalized
            }
            else {
                normalized_emission = active_stake.clone(); // emission proportional to inactive-masked normalized stake
            }
        }
        
        // Compute rao based emission scores. range: I96F32(0, rao_emission)
        let float_rao_emission: I96F32 = I96F32::from_num( rao_emission );
        let emission: Vec<I96F32> = normalized_emission.iter().map( |e: &I32F32| I96F32::from_num( *e ) * float_rao_emission ).collect();
        let emission: Vec<u64> = emission.iter().map( |e: &I96F32| e.to_num::<u64>() ).collect();
        log::debug!( "nE: {:?}", normalized_emission.clone() );
        log::debug!( "E: {:?}", emission.clone() );

        // Set pruning scores.
        let pruning_scores: Vec<I32F32> = normalized_emission.clone();
        log::debug!( "P: {:?}", pruning_scores.clone() );

        // ===================
        // == Value storage ==
        // ===================
        
        // Sync parameter updates.
        for i in 0..n {
            // TODO(taco): set is active.
            Self::set_rank( netuid, i, fixed_proportion_to_u16( ranks[i as usize] ) );
            Self::set_trust( netuid, i, fixed_proportion_to_u16( trust[i as usize] ) );
            Self::set_consensus( netuid, i, fixed_proportion_to_u16( consensus[i as usize] ) );
            Self::set_incentive( netuid, i, fixed_proportion_to_u16( incentive[i as usize] ) );
            Self::set_dividend( netuid, i, fixed_proportion_to_u16( dividends[i as usize] ) );
            Self::set_pruning_score( netuid, i, fixed_proportion_to_u16( pruning_scores[i as usize] ) );
            Self::set_emission( netuid, i, emission[i as usize] );
            Self::set_validator_permit( netuid, i, new_validator_permits[i as usize] );

            // Set bonds only if uid retains validator permit, otherwise clear bonds.
            if new_validator_permits[i as usize] {
                Self::set_bonds( netuid, i, ema_bonds[i as usize].iter().map( |(j, value)| (*j, fixed_proportion_to_u16(*value))).collect())
            }
            else {
                Self::set_bonds( netuid, i, vec![] );
            }
        }

        // Returning the tao emission here which will be distributed at a higher level.
        emission
    }

    pub fn set_rank( netuid:u16, neuron_uid: u16, rank:u16 ) { Rank::<T>::insert( netuid, neuron_uid, rank) }
    pub fn set_trust( netuid:u16, neuron_uid:u16, trust:u16) { Trust::<T>::insert( netuid, neuron_uid, trust ) }
    pub fn set_consensus( netuid:u16, neuron_uid:u16, consensus:u16) { Consensus::<T>::insert( netuid, neuron_uid, consensus ) }
    pub fn set_incentive( netuid:u16, neuron_uid:u16, incentive:u16) { Incentive::<T>::insert( netuid, neuron_uid, incentive ) }
    pub fn set_dividend( netuid:u16, neuron_uid:u16, dividend:u16) { Dividends::<T>::insert( netuid, neuron_uid, dividend ) }
    pub fn set_pruning_score( netuid:u16, neuron_uid: u16, pruning_score: u16 ) { PruningScores::<T>::insert(netuid, neuron_uid, pruning_score); }
    pub fn set_emission( netuid:u16, neuron_uid:u16, emission:u64) { Emission::<T>::insert( netuid, neuron_uid, emission ) }
    pub fn set_bonds( netuid:u16, neuron_uid:u16, bonds:Vec<(u16,u16)>) { Bonds::<T>::insert( netuid, neuron_uid, bonds ) }
    pub fn set_validator_permit( netuid:u16, neuron_uid:u16, validator_permit:bool) { ValidatorPermit::<T>::insert( netuid, neuron_uid, validator_permit ) }

    pub fn get_float_rho( netuid:u16 ) -> I32F32 { I32F32::from_num( Self::get_rho( netuid ) )  }
    pub fn get_float_kappa( netuid:u16 ) -> I32F32 { I32F32::from_num( Self::get_kappa( netuid )  ) / I32F32::from_num( u16::MAX ) }
    pub fn get_rank( netuid:u16, neuron_uid: u16) -> u16 {  Rank::<T>::get( netuid,  neuron_uid) }
    pub fn get_trust( netuid:u16, neuron_uid: u16 ) -> u16 { Trust::<T>::get( netuid, neuron_uid )  }
    pub fn get_consensus( netuid:u16, neuron_uid: u16 ) -> u16 { Consensus::<T>::get( netuid, neuron_uid )  }
    pub fn get_incentive( netuid:u16, neuron_uid: u16 ) -> u16 { Incentive::<T>::get( netuid, neuron_uid )   }
    pub fn get_dividend( netuid:u16, neuron_uid: u16 ) -> u16 { Dividends::<T>::get( netuid, neuron_uid )  }
    pub fn get_emission( netuid:u16, neuron_uid: u16 ) -> u64 { Emission::<T>::get( netuid, neuron_uid )  }
    pub fn get_last_update_for_neuron( netuid:u16, neuron_uid: u16 ) -> u64 { LastUpdate::<T>::get( netuid, neuron_uid ) }
    pub fn get_validator_permit( netuid: u16, neuron_uid: u16 ) -> bool { ValidatorPermit::<T>::get( netuid, neuron_uid ) }

    pub fn get_normalized_stake( netuid:u16 ) -> Vec<I32F32> {
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut stake_64: Vec<I64F64> = vec![ I64F64::from_num(0.0); n ]; 
        for neuron_uid in 0..n {
            stake_64[neuron_uid] = I64F64::from_num( Self::get_stake_for_uid_and_subnetwork( netuid, neuron_uid as u16 ) );
        }
        inplace_normalize_64( &mut stake_64 );
        let stake: Vec<I32F32> = vec_fixed64_to_fixed32( stake_64 );
        stake
    }

    pub fn get_block_at_registration( netuid:u16 ) -> Vec<u64> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize;
        let mut block_at_registration: Vec<u64> = vec![ 0; n ];
        for neuron_uid in 0..n {
            if Keys::<T>::contains_key( netuid, neuron_uid as u16 ){
                block_at_registration[ neuron_uid ] = Self::get_neuron_block_at_registration( netuid, neuron_uid as u16 );
            }
        }
        block_at_registration
    }

    pub fn get_last_update( netuid:u16 ) -> Vec<u64> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize;
        let mut last_update: Vec<u64> = vec![ 0; n ];
        for neuron_uid in 0..n {
            if Keys::<T>::contains_key( netuid, neuron_uid as u16 ){
                last_update[ neuron_uid ] = LastUpdate::<T>::get( netuid, neuron_uid as u16 );
            }
        }
        last_update
    }

    pub fn get_weights_sparse( netuid:u16 ) -> Vec<Vec<(u16, I32F32)>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut weights: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n ]; 
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, weight_ij) in weights_i.iter() { 
                weights [ uid_i as usize ].push( ( *uid_j, u16_proportion_to_fixed( *weight_ij ) ));
            }
        }
        weights
    } 

    pub fn get_weights( netuid:u16 ) -> Vec<Vec<I32F32>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut weights: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(0.0); n ]; n ]; 
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, weight_ij) in weights_i.iter() { 
                weights [ uid_i as usize ] [ *uid_j as usize ] = u16_proportion_to_fixed(  *weight_ij );
            }
        }
        weights
    }

    pub fn get_bonds_sparse( netuid:u16 ) -> Vec<Vec<(u16, I32F32)>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut bonds: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n ]; 
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, bonds_ij) in bonds_i.iter() { 
                bonds [ uid_i as usize ].push( ( *uid_j, u16_proportion_to_fixed( *bonds_ij ) ));
            }
        }
        bonds
    } 

    pub fn get_bonds( netuid:u16 ) -> Vec<Vec<I32F32>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut bonds: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(0.0); n ]; n ]; 
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, bonds_ij) in bonds_i.iter() { 
                bonds [ uid_i as usize ] [ *uid_j as usize ] = u16_proportion_to_fixed( *bonds_ij );
            }
        }
        bonds
    }

    pub fn get_validator_permits( netuid:u16 ) -> Vec<bool> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize;
        let mut validator_permits: Vec<bool> = vec![ false; n ];
        for neuron_uid in 0..n {
            validator_permits[ neuron_uid ] = Self::get_validator_permit(netuid, neuron_uid as u16);
        }
        validator_permits
    }
}

#[allow(dead_code)]
pub fn fixed_to_u16( x: I32F32 ) -> u16 { x.to_num::<u16>() }

#[allow(dead_code)]
pub fn fixed_to_u64( x: I32F32 ) -> u64 { x.to_num::<u64>() }

#[allow(dead_code)]
pub fn fixed64_to_u64( x: I64F64 ) -> u64 { x.to_num::<u64>() }

#[allow(dead_code)]
pub fn fixed64_to_fixed32( x: I64F64 ) -> I32F32 { I32F32::from_num( x ) }

#[allow(dead_code)]
pub fn u16_to_fixed( x: u16 ) -> I32F32 { I32F32::from_num( x ) }

#[allow(dead_code)]
pub fn u16_proportion_to_fixed( x: u16 ) -> I32F32 { I32F32::from_num( x ) / I32F32::from_num( u16::MAX ) }

#[allow(dead_code)]
pub fn fixed_proportion_to_u16( x: I32F32 ) -> u16 { fixed_to_u16( x * I32F32::from_num( u16::MAX )) }

#[allow(dead_code)]
pub fn vec_fixed64_to_fixed32( vec: Vec<I64F64> ) -> Vec<I32F32> { vec.into_iter().map(|e| fixed64_to_fixed32(e) ).collect() }

#[allow(dead_code)]
pub fn vec_u16_proportions_to_fixed( vec: Vec<u16> ) -> Vec<I32F32> { vec.into_iter().map(|e| u16_proportion_to_fixed(e) ).collect() }

#[allow(dead_code)]
pub fn vec_fixed_proportions_to_u16( vec: Vec<I32F32> ) -> Vec<u16> { vec.into_iter().map(|e| fixed_proportion_to_u16(e) ).collect() }

#[allow(dead_code)]
pub fn sum( x: &Vec<I32F32> ) -> I32F32 { x.iter().sum() }

/// Return true when vector sum is zero.
#[allow(dead_code)]
pub fn is_zero( vector: &Vec<I32F32> ) -> bool {
    let vector_sum: I32F32 = sum( &vector );
    vector_sum == I32F32::from_num( 0 )
}

/// Exp safe function with I32F32 output of I32F32 input.
#[allow(dead_code)]
pub fn exp_safe(input: I32F32) -> I32F32 {
    let min_input: I32F32 = I32F32::from_num(-20); // <= 1/exp(-20) = 485 165 195,4097903
    let max_input: I32F32 = I32F32::from_num(20); // <= exp(20) = 485 165 195,4097903
    let mut safe_input: I32F32 = input;
    if input < min_input {
        safe_input = min_input;
    }
    else if max_input < input {
        safe_input = max_input;
    }
    let output: I32F32;
    match exp(safe_input) {
        Ok(val) => {
            output = val;
        },
        Err(_err) => {
            if safe_input <= 0 {
                output = I32F32::from_num(0);
            }
            else {
                output = I32F32::max_value();
            }
        }
    }
    output
}

/// Sigmoid safe function with I32F32 output of I32F32 input with offset kappa and (recommended) scaling 0 < rho <= 40.
#[allow(dead_code)]
pub fn sigmoid_safe(input: I32F32, rho: I32F32, kappa: I32F32) -> I32F32 {
    let one: I32F32 = I32F32::from_num(1);
    let offset: I32F32 = input.saturating_sub(kappa); // (input - kappa)
    let neg_rho: I32F32 = rho.saturating_mul(-one); // -rho
    let exp_input: I32F32 = neg_rho.saturating_mul(offset); // -rho*(input-kappa)
    let exp_output: I32F32 = exp_safe(exp_input); // exp(-rho*(input-kappa))
    let denominator: I32F32 = exp_output.saturating_add(one); // 1 + exp(-rho*(input-kappa))
    let sigmoid_output: I32F32 = one.saturating_div(denominator); // 1 / (1 + exp(-rho*(input-kappa)))
    sigmoid_output
}

/// Returns a bool vector where an item is true if the vector item is in topk values.
#[allow(dead_code)]
pub fn is_topk( vector: &Vec<I32F32>, k: usize ) -> Vec<bool> {
    let n: usize = vector.len();
    let mut result: Vec<bool> = vec![ true; n ];
    if n < k { return result; }
    let mut idxs: Vec<usize> = (0..n).collect();
    idxs.sort_by_key( | &idx | &vector[ idx ] ); // ascending stable sort
    for &idx in &idxs[0..(n-k)] {
        result[ idx ] = false;
    }
    result
}

/// Returns a normalized (sum to 1 except 0) copy of the input vector.
#[allow(dead_code)]
pub fn normalize( x: &Vec<I32F32> ) -> Vec<I32F32> {
    let x_sum: I32F32 = sum( x );
    if x_sum != I32F32::from_num( 0.0 as f32 ) {
        return x.iter().map( |xi| xi / x_sum ).collect();
    } else {
        return x.clone();
    }
}

/// Normalizes (sum to 1 except 0) the input vector directly in-place.
#[allow(dead_code)]
pub fn inplace_normalize( x: &mut Vec<I32F32> ) {
    let x_sum: I32F32 = x.iter().sum();
    if x_sum == I32F32::from_num( 0.0 as f32 ){ return }
    for i in 0..x.len() {
        x[i] = x[i]/x_sum;
    }
}

/// Normalizes (sum to 1 except 0) the I64F64 input vector directly in-place.
#[allow(dead_code)]
pub fn inplace_normalize_64( x: &mut Vec<I64F64> ) {
    let x_sum: I64F64 = x.iter().sum();
    if x_sum == I64F64::from_num( 0 ){ return }
    for i in 0..x.len() {
        x[i] = x[i]/x_sum;
    }
}

/// Normalizes (sum to 1 except 0) each row (dim=0) of a matrix in-place.
#[allow(dead_code)]
pub fn inplace_row_normalize( x: &mut Vec<Vec<I32F32>> ) {
    for i in 0..x.len() {
        let row_sum: I32F32 = x[i].iter().sum();
        if row_sum > I32F32::from_num( 0.0 as f32 ) {
            x[i].iter_mut().for_each(|x_ij| *x_ij /= row_sum);
        }
    }
}

/// Normalizes (sum to 1 except 0) each row (dim=0) of a sparse matrix in-place.
#[allow(dead_code)]
pub fn inplace_row_normalize_sparse( sparse_matrix: &mut Vec<Vec<(u16, I32F32)>> ) {
    for sparse_row in sparse_matrix.iter_mut() {
        let row_sum: I32F32 = sparse_row.iter().map( | (_j, value) | *value ).sum();
        if row_sum > I32F32::from_num( 0.0 ) {
            sparse_row.iter_mut().for_each( | (_j, value) | *value /= row_sum );
        }
    }
}

/// Normalizes (sum to 1 except 0) each column (dim=1) of a sparse matrix in-place.
#[allow(dead_code)]
pub fn inplace_col_normalize_sparse( sparse_matrix: &mut Vec<Vec<(u16, I32F32)>> ) {
    let n = sparse_matrix.len();
    let mut col_sum: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); n ]; // assume square matrix, rows=cols
    for sparse_row in sparse_matrix.iter() {
        for (j, value) in sparse_row.iter() {
            col_sum[*j as usize] += value;
        }
    }
    for sparse_row in sparse_matrix.iter_mut() {
        for (j, value) in sparse_row.iter_mut() {
            if col_sum[*j as usize] == I32F32::from_num( 0.0 as f32 ) { continue }
            *value /= col_sum[*j as usize];
        }
    }
}

/// Normalizes (sum to 1 except 0) each column (dim=1) of a matrix in-place.
#[allow(dead_code)]
pub fn inplace_col_normalize( x: &mut Vec<Vec<I32F32>> ) {
    if x.len() == 0 { return }
    if x[0].len() == 0 { return }
    let cols = x[0].len();
    let mut col_sum: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); cols ];
    for i in 0..x.len() {
        assert_eq!( x[i].len(), cols );
        for j in 0..cols {
            col_sum[j] += x[i][j];
        }
    }
    for j in 0..cols {
        if col_sum[j] == I32F32::from_num( 0.0 as f32 ) { continue }
        for i in 0..x.len() {
            x[i][j] /= col_sum[j];
        }
    }
}

/// Apply mask to vector, mask=true will mask out, i.e. set to 0.
#[allow(dead_code)]
pub fn inplace_mask_vector( mask: &Vec<bool>, vector: &mut Vec<I32F32> ) {
    if mask.len() == 0 { return }
    assert_eq!( mask.len(), vector.len() );
    let zero: I32F32 = I32F32::from_num( 0.0 );
    for i in 0..mask.len() {
        if mask[i] {
            vector[i] = zero;
        }
    }
}

/// Apply mask to matrix, mask=true will mask out, i.e. set to 0.
#[allow(dead_code)]
pub fn inplace_mask_matrix( mask: &Vec<Vec<bool>>, matrix: &mut Vec<Vec<I32F32>> ) {
    if mask.len() == 0 { return }
    if mask[0].len() == 0 { return }
    assert_eq!( mask.len(), matrix.len() );
    let zero: I32F32 = I32F32::from_num( 0.0 );
    for i in 0..mask.len() {
        for j in 0..mask[i].len() {
            if mask[i][j] {
                matrix[i][j] = zero;
            }
        }
    }
}

/// Apply row mask to matrix, mask=true will mask out, i.e. set to 0.
#[allow(dead_code)]
pub fn inplace_mask_rows( mask: &Vec<bool>, matrix: &mut Vec<Vec<I32F32>> ) {
    let rows = matrix.len();
    if rows == 0 { return }
    let cols = matrix[0].len();
    assert_eq!( mask.len(), rows );
    let zero: I32F32 = I32F32::from_num( 0 );
    for i in 0..rows {
        if mask[i] {
            matrix[i] = vec![zero; cols];
        }
    }
}

/// Mask out the diagonal of the input matrix in-place.
#[allow(dead_code)]
pub fn inplace_mask_diag( matrix: &mut Vec<Vec<I32F32>> ) {
    if matrix.len() == 0 { return }
    if matrix[0].len() == 0 { return }
    assert_eq!( matrix.len(), matrix[0].len() );
    let zero: I32F32 = I32F32::from_num( 0.0 );
    for i in 0..matrix.len() {
        matrix[i][i] = zero;
    }
}

/// Return a new sparse matrix that replaces masked rows with an empty vector placeholder.
#[allow(dead_code)]
pub fn mask_rows_sparse( mask: &Vec<bool>, sparse_matrix: &Vec<Vec<(u16, I32F32)>> ) -> Vec<Vec<(u16, I32F32)>> {
    let n: usize = sparse_matrix.len();
    assert_eq!( n, mask.len() );
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        if !mask[i] {
            result[i] = sparse_row.clone();
        }
    }
    result
}

/// Return a new sparse matrix with a masked out diagonal of input sparse matrix.
#[allow(dead_code)]
pub fn mask_diag_sparse( sparse_matrix: &Vec<Vec<(u16, I32F32)>> ) -> Vec<Vec<(u16, I32F32)>> {
    let n: usize = sparse_matrix.len();
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            if i != (*j as usize) {
                result[i].push( (*j, *value) );
            }
        }
    }
    result
}

/// Remove cells from sparse matrix where the mask function of two vectors is true.
#[allow(dead_code)]
pub fn vec_mask_sparse_matrix( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, first_vector: &Vec<u64>, second_vector: &Vec<u64>, mask_fn: &dyn Fn(u64, u64) -> bool) -> Vec<Vec<(u16, I32F32)>> {
    let n: usize = sparse_matrix.len();
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            if !mask_fn(first_vector[i], second_vector[*j as usize]) {
                result[i].push( (*j, *value) );
            }
        }
    }
    result
}

/// Row-wise matrix-vector hadamard product.
#[allow(dead_code)]
pub fn hadamard( matrix: &Vec<Vec<I32F32>>, vector: &Vec<I32F32> ) -> Vec<Vec<I32F32>> {
    if matrix.len() == 0 { return vec![ vec![] ] }
    if matrix[0].len() == 0 { return vec![ vec![] ] }
    let mut result: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num( 0.0 ); matrix[0].len() ]; matrix.len() ];
    for i in 0..matrix.len() {
        for j in 0..matrix[i].len() {
            result[i][j] = vector[i] * matrix[i][j];
        }
    }
    result
}

/// Row-wise sparse matrix-vector hadamard product.
#[allow(dead_code)]
pub fn sparse_hadamard( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, vector: &Vec<I32F32> ) -> Vec<Vec<(u16, I32F32)>> {
    let mut result: Vec<Vec<(u16, I32F32)>> = sparse_matrix.clone();
    for (i, sparse_row) in result.iter_mut().enumerate() {
        for (_j, value) in sparse_row.iter_mut() {
            *value *= vector[i];
        }
    }
    result
}

/// Row-wise matrix-vector product, column-wise sum: result_j = SUM(i) vector_i * matrix_ij.
#[allow(dead_code)]
pub fn matmul( matrix: &Vec<Vec<I32F32>>, vector: &Vec<I32F32> ) -> Vec<I32F32> {
    if matrix.len() == 0 { return vec![] }
    if matrix[0].len() == 0 { return vec![] }
    assert!( matrix.len() == vector.len() );
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); matrix[0].len() ];
    for i in 0..matrix.len() {
        for j in 0..matrix[i].len() {
            // Compute ranks: r_j = SUM(i) w_ij * s_i
            // Compute trust scores: t_j = SUM(i) w_ij * s_i
            // result_j = SUM(i) vector_i * matrix_ij
            result[j] += vector[i] * matrix[i][j];
        }
    }
    result
}

/// Column-wise matrix-vector product, row-wise sum: result_i = SUM(j) vector_j * matrix_ij.
#[allow(dead_code)]
pub fn matmul_transpose( matrix: &Vec<Vec<I32F32>>, vector: &Vec<I32F32> ) -> Vec<I32F32> {
    if matrix.len() == 0 { return vec![] }
    if matrix[0].len() == 0 { return vec![] }
    assert!( matrix[0].len() == vector.len() );
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); matrix.len() ];
    for i in 0..matrix.len() {
        for j in 0..matrix[i].len() {
            // Compute dividends: d_j = SUM(i) b_ji * inc_i
            // result_j = SUM(i) vector_i * matrix_ji
            // result_i = SUM(j) vector_j * matrix_ij
            result[i] += vector[j] * matrix[i][j];
        }
    }
    result
}

/// Row-wise sparse_matrix-vector product, column-wise sum: result_j = SUM(i) vector_i * matrix_ij.
#[allow(dead_code)]
pub fn sparse_matmul( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, vector: &Vec<I32F32>, columns: u16 ) -> Vec<I32F32> {
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); columns as usize ];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            // Compute ranks: r_j = SUM(i) w_ij * s_i
            // Compute trust scores: t_j = SUM(i) w_ij * s_i
            // result_j = SUM(i) vector_i * matrix_ij
            result[*j as usize] += vector[i] * value;
        }
    }
    result
}

/// Column-wise sparse_matrix-vector product, row-wise sum: result_i = SUM(j) vector_j * matrix_ij.
#[allow(dead_code)]
pub fn sparse_matmul_transpose( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, vector: &Vec<I32F32> ) -> Vec<I32F32> {
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); sparse_matrix.len() ];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            // Compute dividends: d_j = SUM(i) b_ji * inc_i
            // result_j = SUM(i) vector_i * matrix_ji
            // result_i = SUM(j) vector_j * matrix_ij
            result[i] += vector[*j as usize] * value;
        }
    }
    result
}

/// Set sparse matrix values below threshold to lower, and equal-above to upper.
/// Does not add missing elements (0 value assumed) when lower!=0.
#[allow(dead_code)]
pub fn sparse_clip( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, threshold: I32F32, upper: I32F32, lower: I32F32) -> Vec<Vec<(u16, I32F32)>> {
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; sparse_matrix.len() ];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            if *value < threshold {
                result[i].push( (*j, lower) );
            }
            else {
                result[i].push( (*j, upper) );
            }
        }
    }
    result
}

/// Set matrix values below threshold to lower, and equal-above to upper.
#[allow(dead_code)]
pub fn clip( x: &Vec<Vec<I32F32>>, threshold: I32F32, upper: I32F32, lower: I32F32) -> Vec<Vec<I32F32>> {
    // Check Nill length. 
    if x.len() == 0 {
        return vec![ vec![ ] ];
    }
    let mut result: Vec<Vec<I32F32>> = vec![ vec![ lower; x[0].len() ]; x.len() ]; 
    for i in 0..x.len() {
        for j in 0..x[i].len() {
            if x [ i ][ j ] >= threshold {
                result[ i ][ j ] = upper;
            }
        }
    }
    result
}

/// Set inplace matrix values below threshold to lower, and equal-above to upper.
#[allow(dead_code)]
pub fn inplace_clip( x: &mut Vec<Vec<I32F32>>, threshold: I32F32, upper: I32F32, lower: I32F32 ) {
    for i in 0..x.len() {
        for j in 0..x[i].len() {
            if x [ i ][ j ] >= threshold {
                x[ i ][ j ] = upper;
            } else {
                x[ i ][ j ] = lower;
            }
        }
    }
}

/// Return matrix exponential moving average: `alpha * a_ij + one_minus_alpha * b_ij`.
/// `alpha` is the EMA coefficient, how much to add of the new observation, typically small, 
/// higher alpha discounts older observations faster.
#[allow(dead_code)]
pub fn mat_ema( new: &Vec<Vec<I32F32>>, old: &Vec<Vec<I32F32>>, alpha: I32F32 ) -> Vec<Vec<I32F32>> {
    if new.len() == 0 { return vec![vec![];1] }
    if new[0].len() == 0 { return vec![vec![];1] }
    let one_minus_alpha:I32F32 = I32F32::from_num( 1.0 ) - alpha;
    let mut result: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num( 0.0 ); new[0].len() ]; new.len() ]; 
    assert!(new.len() == old.len());
    for i in 0..new.len() {
        assert!(new[i].len() == old[i].len());
        for j in 0..new[i].len() {
            result[i][j] = alpha * new[i][j] + one_minus_alpha * old[i][j] 
        }
    }
    result
}

/// Return sparse matrix exponential moving average: `alpha * a_ij + one_minus_alpha * b_ij`.
/// `alpha` is the EMA coefficient, how much to add of the new observation, typically small, 
/// higher alpha discounts older observations faster.
#[allow(dead_code)]
pub fn sparse_mat_ema( new: &Vec<Vec<(u16, I32F32)>>, old: &Vec<Vec<(u16, I32F32)>>, alpha: I32F32 ) -> Vec<Vec<(u16, I32F32)>> {
    assert!(new.len() == old.len());
    let n = new.len(); // assume square matrix, rows=cols
    let zero: I32F32 = I32F32::from_num( 0.0 );
    let one_minus_alpha:I32F32 = I32F32::from_num( 1.0 ) - alpha;
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n ];
    for i in 0..new.len() {
        let mut row: Vec<I32F32> = vec![ zero; n];
        for (j, value) in new[i].iter() {
            row[*j as usize] += alpha * value;
        }
        for (j, value) in old[i].iter() {
            row[*j as usize] += one_minus_alpha * value;
        }
        for (j, value) in row.iter().enumerate() {
            if value > &zero {
                result[i].push( (j as u16, *value) )
            }
        }
    }
    result
}

/// Return sparse matrix only with elements >= threshold of an input sparse matrix.
#[allow(dead_code)]
pub fn sparse_threshold( w: &Vec<Vec<(u16, I32F32)>>, threshold: I32F32 ) -> Vec<Vec<(u16, I32F32)>> {
    let mut sparse_threshold_result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; w.len() ]; 
    for ( uid_i, weights_i ) in w.iter().enumerate() {
        for (uid_j, weight_ij) in weights_i.iter() { 
            if *weight_ij >= threshold {
                sparse_threshold_result [ uid_i as usize ].push( ( *uid_j, *weight_ij ));
            }
        }
    }
    sparse_threshold_result
}

#[cfg(test)]
mod tests {
    use substrate_fixed::transcendental::exp;
    use substrate_fixed::types::{I32F32, I64F64, I96F32, I110F18};
    use crate::epoch;

    fn assert_float_compare(a: I32F32, b: I32F32, epsilon: I32F32 ) {
        assert!( I32F32::abs( a - b ) <= epsilon, "a({:?}) != b({:?})", a, b);
    }

    fn assert_float_compare_64(a: I64F64, b: I64F64, epsilon: I64F64 ) {
        assert!( I64F64::abs( a - b ) <= epsilon, "a({:?}) != b({:?})", a, b);
    }
    
    fn assert_vec_compare(va: &Vec<I32F32>, vb: &Vec<I32F32>, epsilon: I32F32) {
        assert!(va.len() == vb.len());
        for i in 0..va.len(){
            assert_float_compare(va[i], vb[i], epsilon);
        }  
    }
    
    fn assert_vec_compare_64(va: &Vec<I64F64>, vb: &Vec<I64F64>, epsilon: I64F64) {
        assert!(va.len() == vb.len());
        for i in 0..va.len(){
            assert_float_compare_64(va[i], vb[i], epsilon);
        }  
    }
    
    fn assert_mat_compare(ma: &Vec<Vec<I32F32>>, mb: &Vec<Vec<I32F32>>, epsilon: I32F32) {
        assert!(ma.len() == mb.len());
        for row in 0..ma.len() {
            assert!(ma[row].len() == mb[row].len());
            for col in 0..ma[row].len() {
                assert_float_compare(ma[row][col], mb[row][col], epsilon)
            }
        }
    }
    
    fn assert_sparse_mat_compare(ma: &Vec<Vec<(u16, I32F32)>>, mb: &Vec<Vec<(u16, I32F32)>>, epsilon: I32F32) {
        assert!(ma.len() == mb.len());
        for row in 0..ma.len() {
            assert!(ma[row].len() == mb[row].len());
            for j in 0..ma[row].len() {
                assert!(ma[row][j].0 == mb[row][j].0); // u16
                assert_float_compare(ma[row][j].1, mb[row][j].1, epsilon) // I32F32
            }
        }
    }

    fn vec_to_fixed(vector: &Vec<f32>) -> Vec<I32F32> {
        vector.iter().map( | x | I32F32::from_num( *x ) ).collect()
    }

    #[test]
    fn test_fixed_overflow() {
        let max_32: I32F32 = I32F32::max_value();
        let max_u64: u64 = u64::MAX;
        let _prod_96: I96F32 = I96F32::from_num(max_32) * I96F32::from_num(max_u64);
        // let one: I96F32 = I96F32::from_num(1);
        // let prod_96: I96F32 = (I96F32::from_num(max_32) + one) * I96F32::from_num(max_u64); // overflows
        let _prod_110: I110F18 = I110F18::from_num(max_32) * I110F18::from_num(max_u64);
    }

    #[test]
    fn test_u64_normalization() {
        let min: u64 = 1;
        let min32: u64 = 4_889_444; // 21_000_000_000_000_000 / 4_294_967_296
        let mid: u64 = 10_500_000_000_000_000;
        let max: u64 = 21_000_000_000_000_000;
        let min_64: I64F64 = I64F64::from_num(min);
        let min32_64: I64F64 = I64F64::from_num(min32);
        let mid_64: I64F64 = I64F64::from_num(mid);
        let max_64: I64F64 = I64F64::from_num(max);
        let max_sum: I64F64 = I64F64::from_num(max);
        let min_frac: I64F64 = min_64 / max_sum;
        assert_eq!(min_frac, I64F64::from_num(0.0000000000000000476));
        let min_frac_32: I32F32 = I32F32::from_num(min_frac);
        assert_eq!(min_frac_32, I32F32::from_num(0));
        let min32_frac: I64F64 = min32_64 / max_sum;
        assert_eq!(min32_frac,    I64F64::from_num(0.00000000023283066664));
        let min32_frac_32: I32F32 = I32F32::from_num(min32_frac);
        assert_eq!(min32_frac_32, I32F32::from_num(0.0000000002));
        let half: I64F64 = mid_64 / max_sum;
        assert_eq!(half, I64F64::from_num(0.5));
        let half_32: I32F32 = I32F32::from_num(half);
        assert_eq!(half_32, I32F32::from_num(0.5));
        let one: I64F64 = max_64 / max_sum;
        assert_eq!(one, I64F64::from_num(1));
        let one_32: I32F32 = I32F32::from_num(one);
        assert_eq!(one_32, I32F32::from_num(1));
    }

    #[test]
    fn test_to_num() {
        let val: I32F32 = I32F32::from_num(u16::MAX);
        let res: u16 = val.to_num::<u16>();
        assert_eq!(res, u16::MAX);
        let vector: Vec<I32F32> = vec![val; 1000];
        let target: Vec<u16> = vec![u16::MAX; 1000];
        let output: Vec<u16> = vector.iter().map( |e: &I32F32| e.to_num::<u16>() ).collect();
        assert_eq!(output, target);
        let output: Vec<u16> = vector.iter().map( |e: &I32F32| (*e).to_num::<u16>() ).collect();
        assert_eq!(output, target);
        let val: I32F32 = I32F32::max_value();
        let res: u64 = val.to_num::<u64>();
        let vector: Vec<I32F32> = vec![val; 1000];
        let target: Vec<u64> = vec![res; 1000];
        let output: Vec<u64> = vector.iter().map( |e: &I32F32| e.to_num::<u64>() ).collect();
        assert_eq!(output, target);
        let output: Vec<u64> = vector.iter().map( |e: &I32F32| (*e).to_num::<u64>() ).collect();
        assert_eq!(output, target);
        let val: I96F32 = I96F32::from_num(u64::MAX);
        let res: u64 = val.to_num::<u64>();
        assert_eq!(res, u64::MAX);
        let vector: Vec<I96F32> = vec![val; 1000];
        let target: Vec<u64> = vec![u64::MAX; 1000];
        let output: Vec<u64> = vector.iter().map( |e: &I96F32| e.to_num::<u64>() ).collect();
        assert_eq!(output, target);
        let output: Vec<u64> = vector.iter().map( |e: &I96F32| (*e).to_num::<u64>() ).collect();
        assert_eq!(output, target);
    }
    
    #[test]
    fn test_vec_to_fixed() {
        let vector: Vec<f32> = vec![ 0., 1., 2., 3.];
        let target: Vec<I32F32> = vec![I32F32::from_num(0.), I32F32::from_num(1.), I32F32::from_num(2.), I32F32::from_num(3.)];
        let result = vec_to_fixed(&vector);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
    }

    /// Reshape vector to matrix with specified number of rows, cast to I32F32.
    fn vec_to_mat_fixed(vector: &Vec<f32>, rows: usize, transpose: bool) -> Vec<Vec<I32F32>> {
        assert!( vector.len() % rows == 0, "Vector of len {:?} cannot reshape to {rows} rows.", vector.len());
        let cols: usize = vector.len() / rows;
        let mut mat: Vec<Vec<I32F32>> = vec![];
        if transpose {
            for col in 0..cols as usize {
                let mut vals: Vec<I32F32> = vec![];
                for row in 0..rows as usize {
                    vals.push( I32F32::from_num( vector[row * cols + col]) );
                }
                mat.push(vals);
            }
        }
        else {
            for row in 0..rows as usize {
                mat.push( vector[row * cols .. (row + 1) * cols].iter().map( | v | I32F32::from_num( *v ) ).collect() );
            }
        }
        mat
    }

    #[test]
    fn test_vec_to_mat_fixed() {
        let vector: Vec<f32> = vec![ 0., 1., 2.,
                                    0., 10., 100.];
        let target: Vec<Vec<I32F32>> = vec![vec![I32F32::from_num(0.), I32F32::from_num(1.), I32F32::from_num(2.)], 
                                            vec![I32F32::from_num(0.), I32F32::from_num(10.), I32F32::from_num(100.)] ];
        let mat = vec_to_mat_fixed(&vector, 2, false);
        assert_mat_compare(&mat, &target, I32F32::from_num(0));
    }

    /// Reshape vector to sparse matrix with specified number of input rows, cast f32 to I32F32.
    fn vec_to_sparse_mat_fixed(vector: &Vec<f32>, rows: usize, transpose: bool) -> Vec<Vec<(u16, I32F32)>> {
        assert!( vector.len() % rows == 0, "Vector of len {:?} cannot reshape to {rows} rows.", vector.len());
        let cols: usize = vector.len() / rows;
        let mut mat: Vec<Vec<(u16, I32F32)>> = vec![];
        if transpose {
            for col in 0..cols as usize {
                let mut row_vec: Vec<(u16, I32F32)> = vec![];
                for row in 0..rows as usize {
                    if vector[row * cols + col] > 0. {
                        row_vec.push( (row as u16, I32F32::from_num(vector[row * cols + col])) );
                    }
                }
                mat.push(row_vec);
            }
        }
        else {
            for row in 0..rows as usize {
                let mut row_vec: Vec<(u16, I32F32)> = vec![];
                for col in 0..cols as usize {
                    if vector[row * cols + col] > 0. {
                        row_vec.push( (col as u16, I32F32::from_num(vector[row * cols + col])) );
                    }
                }
                mat.push(row_vec);
            }
        }
        mat
    }

    #[test]
    fn test_vec_to_sparse_mat_fixed() {
        let vector: Vec<f32> = vec![ 0., 1., 2.,
                                    0., 10., 100.];
        let target: Vec<Vec<(u16, I32F32)>> = vec![ vec![(1 as u16, I32F32::from_num(1.)), (2 as u16, I32F32::from_num(2.))], 
                                                    vec![(1 as u16, I32F32::from_num(10.)), (2 as u16, I32F32::from_num(100.))] ];
        let mat = vec_to_sparse_mat_fixed(&vector, 2, false);
        assert_sparse_mat_compare(&mat, &target, I32F32::from_num(0));
        let vector: Vec<f32> = vec![ 0.,
                                     0.];
        let target: Vec<Vec<(u16, I32F32)>> = vec![ vec![], 
                                                    vec![] ];
        let mat = vec_to_sparse_mat_fixed(&vector, 2, false);
        assert_sparse_mat_compare(&mat, &target, I32F32::from_num(0));
        let vector: Vec<f32> = vec![ 0., 1., 2.,
                                    0., 10., 100.];
        let target: Vec<Vec<(u16, I32F32)>> = vec![ vec![],
                                                    vec![(0 as u16, I32F32::from_num(1.)), (1 as u16, I32F32::from_num(10.))], 
                                                    vec![(0 as u16, I32F32::from_num(2.)), (1 as u16, I32F32::from_num(100.))] ];
        let mat = vec_to_sparse_mat_fixed(&vector, 2, true);
        assert_sparse_mat_compare(&mat, &target, I32F32::from_num(0));
        let vector: Vec<f32> = vec![ 0.,
                                     0.];
        let target: Vec<Vec<(u16, I32F32)>> = vec![ vec![] ];
        let mat = vec_to_sparse_mat_fixed(&vector, 2, true);
        assert_sparse_mat_compare(&mat, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_exp_safe() {
        let zero: I32F32 = I32F32::from_num(0);
        let one: I32F32 = I32F32::from_num(1);
        let target: I32F32 = exp(zero).unwrap();
        assert_eq!(epoch::exp_safe(zero), target);
        let target: I32F32 = exp(one).unwrap();
        assert_eq!(epoch::exp_safe(one), target);
        let min_input: I32F32 = I32F32::from_num(-20); // <= 1/exp(-20) = 485 165 195,4097903
        let max_input: I32F32 = I32F32::from_num(20); // <= exp(20) = 485 165 195,4097903
        let target: I32F32 = exp(min_input).unwrap();
        assert_eq!(epoch::exp_safe(min_input), target);
        assert_eq!(epoch::exp_safe(min_input - one), target);
        assert_eq!(epoch::exp_safe(I32F32::min_value()), target);
        let target: I32F32 = exp(max_input).unwrap();
        assert_eq!(epoch::exp_safe(max_input), target);
        assert_eq!(epoch::exp_safe(max_input + one), target);
        assert_eq!(epoch::exp_safe(I32F32::max_value()), target);
    }

    #[test]
    fn test_sigmoid_safe() {
        let trust: Vec<I32F32> = vec![I32F32::min_value(), I32F32::from_num(0), I32F32::from_num(0.4), I32F32::from_num(0.5), I32F32::from_num(0.6), I32F32::from_num(1), I32F32::max_value()];
        let consensus: Vec<I32F32> = trust.iter().map(|t: &I32F32| epoch::sigmoid_safe(*t, I32F32::max_value(), I32F32::max_value())).collect();
        let target: Vec<I32F32> = vec_to_fixed(&vec![0.0000000019, 0.0000000019, 0.0000000019, 0.0000000019, 0.0000000019, 0.0000000019, 0.5]);
        assert_eq!(&consensus, &target);
        let consensus: Vec<I32F32> = trust.iter().map(|t: &I32F32| epoch::sigmoid_safe(*t, I32F32::min_value(), I32F32::min_value())).collect();
        let target: Vec<I32F32> = vec_to_fixed(&vec![0.5, 0.0000000019, 0.0000000019, 0.0000000019, 0.0000000019, 0.0000000019, 0.0000000019]);
        assert_eq!(&consensus, &target);
        let consensus: Vec<I32F32> = trust.iter().map(|t: &I32F32| epoch::sigmoid_safe(*t, I32F32::from_num(30), I32F32::from_num(0.5))).collect();
        let target: Vec<f64> = vec![0.0000000019, 0.0000003057, 0.0474258729, 0.5, 0.952574127, 0.9999996943, 0.9999999981];
        let target: Vec<I32F32> = target.iter().map(|c: &f64| I32F32::from_num(*c)).collect();
        assert_eq!(&consensus, &target);
        let trust: Vec<I32F32> = vec_to_fixed(&vec![0., 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.]);
        let consensus: Vec<I32F32> = trust.iter().map(|t: &I32F32| epoch::sigmoid_safe(*t, I32F32::from_num(40), I32F32::from_num(0.5))).collect();
        let target: Vec<f64> = vec![0.0000000019, 0.0000001125, 0.0000061442, 0.0003353502, 0.017986214, 0.5, 0.9820138067, 0.9996646498, 0.9999938558, 0.9999998875, 0.9999999981];
        let target: Vec<I32F32> = target.iter().map(|c: &f64| I32F32::from_num(*c)).collect();
        assert_eq!(&consensus, &target);
    }

    #[test]
    fn test_is_topk() {
        let vector: Vec<I32F32> = vec_to_fixed(&vec![]);
        let result = epoch::is_topk(&vector, 5);
        let target: Vec<bool> = vec![];
        assert_eq!(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0., 1., 2., 3., 4., 5., 6., 7., 8., 9.]);
        let result = epoch::is_topk(&vector, 0);
        let target: Vec<bool> = vec![false, false, false, false, false, false, false, false, false, false];
        assert_eq!(&result, &target);
        let result = epoch::is_topk(&vector, 5);
        let target: Vec<bool> = vec![false, false, false, false, false, true, true, true, true, true];
        assert_eq!(&result, &target);
        let result = epoch::is_topk(&vector, 10);
        let target: Vec<bool> = vec![true, true, true, true, true, true, true, true, true, true];
        assert_eq!(&result, &target);
        let result = epoch::is_topk(&vector, 100);
        assert_eq!(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![9., 8., 7., 6., 5., 4., 3., 2., 1., 0.]);
        let result = epoch::is_topk(&vector, 5);
        let target: Vec<bool> = vec![true, true, true, true, true, false, false, false, false, false];
        assert_eq!(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![9., 0., 8., 1., 7., 2., 6., 3., 5., 4.]);
        let result = epoch::is_topk(&vector, 5);
        let target: Vec<bool> = vec![true, false, true, false, true, false, true, false, true, false];
        assert_eq!(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0.9, 0., 0.8, 0.1, 0.7, 0.2, 0.6, 0.3, 0.5, 0.4]);
        let result = epoch::is_topk(&vector, 5);
        let target: Vec<bool> = vec![true, false, true, false, true, false, true, false, true, false];
        assert_eq!(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0., 1., 2., 3., 4., 5., 5., 5., 5., 6.]);
        let result = epoch::is_topk(&vector, 5);
        let target: Vec<bool> = vec![false, false, false, false, false, true, true, true, true, true];
        assert_eq!(&result, &target);
    }

    #[test]
    fn test_math_sum() {
        assert!( epoch::sum(&vec![]) == I32F32::from_num(0));
        assert!( epoch::sum(&vec![ I32F32::from_num(1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]) == I32F32::from_num(41));
        assert!( epoch::sum(&vec![ I32F32::from_num(-1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]) == I32F32::from_num(39));
    }

    #[test]
    fn test_math_normalize() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let x: Vec<I32F32> = vec![]; 
        let y: Vec<I32F32> = epoch::normalize(&x);
        assert_vec_compare( &x, &y, epsilon);
        let x: Vec<I32F32> = vec![ I32F32::from_num(1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]; 
        let y: Vec<I32F32> = epoch::normalize(&x);
        assert_vec_compare( &y, &vec![ I32F32::from_num(0.0243902437),  I32F32::from_num(0.243902439),  I32F32::from_num(0.7317073171)], epsilon );
        assert_float_compare( epoch::sum( &y ), I32F32::from_num(1.0), epsilon);
        let x: Vec<I32F32> = vec![ I32F32::from_num(-1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]; 
        let y: Vec<I32F32> = epoch::normalize(&x);
        assert_vec_compare( &y, &vec![ I32F32::from_num(-0.0256410255),  I32F32::from_num(0.2564102563),  I32F32::from_num(0.769230769)], epsilon );
        assert_float_compare( epoch::sum( &y ), I32F32::from_num(1.0), epsilon );
    }

    #[test]
    fn test_math_inplace_normalize() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let mut x1: Vec<I32F32> = vec![ I32F32::from_num(1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]; 
        epoch::inplace_normalize(&mut x1);
        assert_vec_compare( &x1, &vec![ I32F32::from_num(0.0243902437),  I32F32::from_num(0.243902439),  I32F32::from_num(0.7317073171)], epsilon );
        let mut x2: Vec<I32F32> = vec![ I32F32::from_num(-1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]; 
        epoch::inplace_normalize(&mut x2);
        assert_vec_compare( &x2, &vec![ I32F32::from_num(-0.0256410255),  I32F32::from_num(0.2564102563),  I32F32::from_num(0.769230769)], epsilon );
    }

    #[test]
    fn test_math_inplace_normalize_64() {
        let epsilon: I64F64 = I64F64::from_num(0.0001);
        let mut x1: Vec<I64F64> = vec![ I64F64::from_num(1.0),  I64F64::from_num(10.0),  I64F64::from_num(30.0)]; 
        epoch::inplace_normalize_64(&mut x1);
        assert_vec_compare_64( &x1, &vec![ I64F64::from_num(0.0243902437),  I64F64::from_num(0.243902439),  I64F64::from_num(0.7317073171)], epsilon );
        let mut x2: Vec<I64F64> = vec![ I64F64::from_num(-1.0),  I64F64::from_num(10.0),  I64F64::from_num(30.0)]; 
        epoch::inplace_normalize_64(&mut x2);
        assert_vec_compare_64( &x2, &vec![ I64F64::from_num(-0.0256410255),  I64F64::from_num(0.2564102563),  I64F64::from_num(0.769230769)], epsilon );
    }

    #[test]
    fn test_math_inplace_row_normalize() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let vector:Vec<f32> = vec![ 0., 1., 2., 3., 4., 
                                    0., 10., 100., 1000., 10000., 
                                    0., 0., 0., 0., 0., 
                                    1., 1., 1., 1., 1.];
        let mut mat = vec_to_mat_fixed(&vector, 4, false);
        epoch::inplace_row_normalize(&mut mat);
        let target:Vec<f32> = vec![ 0., 0.1, 0.2, 0.3, 0.4, 
                                    0., 0.0009, 0.009, 0.09, 0.9, 
                                    0., 0., 0., 0., 0., 
                                    0.2, 0.2, 0.2, 0.2, 0.2 ];
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 4, false), epsilon);
    }

    #[test]
    fn test_math_inplace_row_normalize_sparse() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let vector:Vec<f32> = vec![ 0., 1., 0., 2., 0., 3., 4., 
                                    0., 1., 0., 2., 0., 3., 0., 
                                    1., 0., 0., 2., 0., 3., 4., 
                                    0., 10., 0., 100., 1000., 0., 10000., 
                                    0., 0., 0., 0., 0., 0., 0., 
                                    1., 1., 1., 1., 1., 1., 1.];
        let mut mat = vec_to_sparse_mat_fixed(&vector, 6, false);
        epoch::inplace_row_normalize_sparse(&mut mat);
        let target:Vec<f32> = vec![ 0., 0.1, 0., 0.2, 0., 0.3, 0.4, 
                                    0., 0.166666, 0., 0.333333, 0., 0.5, 0., 
                                    0.1, 0., 0., 0.2, 0., 0.3, 0.4, 
                                    0., 0.0009, 0., 0.009, 0.09, 0., 0.9, 
                                    0., 0., 0., 0., 0., 0., 0., 
                                    0.142857, 0.142857, 0.142857, 0.142857, 0.142857, 0.142857, 0.142857];
        assert_sparse_mat_compare(&mat, &vec_to_sparse_mat_fixed(&target, 6, false), epsilon);
        let vector:Vec<f32> = vec![ 0., 0., 0., 0.,
                                    0., 0., 0., 0., 
                                    0., 0., 0., 0.];
        let target:Vec<f32> = vec![ 0., 0., 0., 0.,
                                    0., 0., 0., 0.,
                                    0., 0., 0., 0.];
        let mut mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        epoch::inplace_row_normalize_sparse(&mut mat);
        assert_sparse_mat_compare(&mat, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_inplace_col_normalize() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let vector:Vec<f32> = vec![ 0., 1., 2., 3., 4., 
                                    0., 10., 100., 1000., 10000., 
                                    0., 0., 0., 0., 0., 
                                    1., 1., 1., 1., 1.];
        let mut mat = vec_to_mat_fixed(&vector, 4, true);
        epoch::inplace_col_normalize(&mut mat);
        let target:Vec<f32> = vec![ 0., 0.1, 0.2, 0.3, 0.4, 
                                    0., 0.0009, 0.009, 0.09, 0.9, 
                                    0., 0., 0., 0., 0., 
                                    0.2, 0.2, 0.2, 0.2, 0.2 ];
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 4, true), epsilon);
    }

    #[test]
    fn test_math_inplace_col_normalize_sparse() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let vector:Vec<f32> = vec![ 0., 1., 0., 2., 0., 3., 4., 
                                    0., 1., 0., 2., 0., 3., 0., 
                                    1., 0., 0., 2., 0., 3., 4., 
                                    0., 10., 0., 100., 1000., 0., 10000., 
                                    0., 0., 0., 0., 0., 0., 0., 
                                    1., 1., 1., 1., 1., 1., 1.];
        let mut mat = vec_to_sparse_mat_fixed(&vector, 6, true);
        epoch::inplace_col_normalize_sparse(&mut mat);
        let target:Vec<f32> = vec![ 0., 0.1, 0., 0.2, 0., 0.3, 0.4, 
                                    0., 0.166666, 0., 0.333333, 0., 0.5, 0., 
                                    0.1, 0., 0., 0.2, 0., 0.3, 0.4, 
                                    0., 0.0009, 0., 0.009, 0.09, 0., 0.9, 
                                    0., 0., 0., 0., 0., 0., 0., 
                                    0.142857, 0.142857, 0.142857, 0.142857, 0.142857, 0.142857, 0.142857];
        assert_sparse_mat_compare(&mat, &vec_to_sparse_mat_fixed(&target, 6, true), epsilon);
        let vector:Vec<f32> = vec![ 0., 0., 0., 0.,
                                    0., 0., 0., 0., 
                                    0., 0., 0., 0.];
        let target:Vec<f32> = vec![ 0., 0., 0., 0.,
                                    0., 0., 0., 0.,
                                    0., 0., 0., 0.];
        let mut mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        epoch::inplace_col_normalize_sparse(&mut mat);
        assert_sparse_mat_compare(&mat, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_inplace_mask_vector() {
        let mask:Vec<bool> = vec![ false, false, false ];
        let mut vector:Vec<I32F32> = vec_to_fixed(&vec![ 0., 1., 2.]);
        let target:Vec<I32F32> = vec_to_fixed(&vec![ 0., 1., 2.]);
        epoch::inplace_mask_vector(&mask, &mut vector);
        assert_vec_compare(&vector, &target, I32F32::from_num( 0 ));
        let mask:Vec<bool> = vec![ false, true, false ];
        let mut vector:Vec<I32F32> = vec_to_fixed(&vec![ 0., 1., 2.]);
        let target:Vec<I32F32> = vec_to_fixed(&vec![ 0., 0., 2.]);
        epoch::inplace_mask_vector(&mask, &mut vector);
        assert_vec_compare(&vector, &target, I32F32::from_num( 0 ));
        let mask:Vec<bool> = vec![ true, true, true ];
        let mut vector:Vec<I32F32> = vec_to_fixed(&vec![ 0., 1., 2.]);
        let target:Vec<I32F32> = vec_to_fixed(&vec![ 0., 0., 0.]);
        epoch::inplace_mask_vector(&mask, &mut vector);
        assert_vec_compare(&vector, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_inplace_mask_matrix() {
        let mask:Vec<Vec<bool>> = vec![ vec![false, false, false], 
                                        vec![false, false, false], 
                                        vec![false, false, false]];
        let vector:Vec<f32> = vec![ 0., 1., 2., 
                                    3., 4., 5., 
                                    6., 7., 8.];
        let mut mat = vec_to_mat_fixed(&vector, 3, false);
        epoch::inplace_mask_matrix(&mask, &mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&vector, 3, false), I32F32::from_num( 0 ));
        let mask:Vec<Vec<bool>> = vec![ vec![true, false, false], 
                                        vec![false, true, false], 
                                        vec![false, false, true]];
        let target:Vec<f32> = vec![ 0., 1., 2., 
                                    3., 0., 5., 
                                    6., 7., 0.];
        let mut mat = vec_to_mat_fixed(&vector, 3, false);
        epoch::inplace_mask_matrix(&mask, &mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let mask:Vec<Vec<bool>> = vec![ vec![true, true, true], 
                                        vec![true, true, true], 
                                        vec![true, true, true]];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let mut mat = vec_to_mat_fixed(&vector, 3, false);
        epoch::inplace_mask_matrix(&mask, &mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_inplace_mask_rows() {
        let input:Vec<f32> = vec![  1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.];
        let mask:Vec<bool> = vec![false, false, false];
        let target:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.];
        let mut mat = vec_to_mat_fixed(&input, 3, false);
        epoch::inplace_mask_rows(&mask, &mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let mask:Vec<bool> = vec![true, true, true];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let mut mat = vec_to_mat_fixed(&input, 3, false);
        epoch::inplace_mask_rows(&mask, &mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let mask:Vec<bool> = vec![true, false, true];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    4., 5., 6., 
                                    0., 0., 0.];
        let mut mat = vec_to_mat_fixed(&input, 3, false);
        epoch::inplace_mask_rows(&mask, &mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let input:Vec<f32> = vec![  0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let mut mat = vec_to_mat_fixed(&input, 3, false);
        let mask:Vec<bool> = vec![false, false, false];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        epoch::inplace_mask_rows(&mask, &mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_inplace_mask_diag() {
        let vector:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.];
        let target:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.];
        let mut mat = vec_to_mat_fixed(&vector, 3, false);
        epoch::inplace_mask_diag(&mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_mask_rows_sparse() {
        let input:Vec<f32> = vec![  1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.];
        let mat = vec_to_sparse_mat_fixed(&input, 3, false);
        let mask:Vec<bool> = vec![false, false, false];
        let target:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.];
        let result = epoch::mask_rows_sparse(&mask, &mat);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let mask:Vec<bool> = vec![true, true, true];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let result = epoch::mask_rows_sparse(&mask, &mat);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let mask:Vec<bool> = vec![true, false, true];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    4., 5., 6., 
                                    0., 0., 0.];
        let result = epoch::mask_rows_sparse(&mask, &mat);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let input:Vec<f32> = vec![  0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let mat = vec_to_sparse_mat_fixed(&input, 3, false);
        let mask:Vec<bool> = vec![false, false, false];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let result = epoch::mask_rows_sparse(&mask, &mat);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_mask_diag_sparse() {
        let vector:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.];
        let target:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let result = epoch::mask_diag_sparse(&mat);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let vector:Vec<f32> = vec![ 1., 0., 0., 
                                    0., 5., 0., 
                                    0., 0., 9.];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let result = epoch::mask_diag_sparse(&mat);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let vector:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let result = epoch::mask_diag_sparse(&mat);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_vec_mask_sparse_matrix() {
        let vector:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.];
        let target:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let first_vector: Vec<u64> = vec![ 1, 2, 3 ];
        let second_vector: Vec<u64> = vec![ 1, 2, 3 ];
        let result = epoch::vec_mask_sparse_matrix(&mat, &first_vector, &second_vector, &|a, b| a == b);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let target:Vec<f32> = vec![ 1., 0., 0., 
                                    4., 5., 0., 
                                    7., 8., 9.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let first_vector: Vec<u64> = vec![ 1, 2, 3 ];
        let second_vector: Vec<u64> = vec![ 1, 2, 3 ];
        let result = epoch::vec_mask_sparse_matrix(&mat, &first_vector, &second_vector, &|a, b| a < b);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let vector:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let first_vector: Vec<u64> = vec![ 1, 2, 3 ];
        let second_vector: Vec<u64> = vec![ 1, 2, 3 ];
        let result = epoch::vec_mask_sparse_matrix(&mat, &first_vector, &second_vector, &|a, b| a == b);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_hadamard() {
        let vector:Vec<I32F32> = vec_to_fixed( &vec![ 1., 2., 3., 4.] );
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let result = epoch::hadamard(&matrix, &vector);
        let target:Vec<f32> = vec![ 1., 2., 3., 
                                    8., 10., 12., 
                                    21., 24., 27.,
                                    40., 44., 48.];
        let target = vec_to_mat_fixed(&target, 4, false);
        assert_mat_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_sparse_hadamard() {
        let vector:Vec<I32F32> = vec_to_fixed( &vec![ 1., 2., 3., 4.] );
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = epoch::sparse_hadamard(&matrix, &vector);
        let target:Vec<f32> = vec![ 1., 2., 3., 
                                    8., 10., 12., 
                                    21., 24., 27.,
                                    40., 44., 48.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = epoch::sparse_hadamard(&matrix, &vector);
        let target:Vec<f32> = vec![ 0., 2., 3., 
                                    8., 0., 12., 
                                    21., 24., 0.,
                                    40., 44., 48.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = epoch::sparse_hadamard(&matrix, &vector);
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_matmul() {
        let vector:Vec<I32F32> = vec_to_fixed( &vec![ 1., 2., 3., 4.] );
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let result = epoch::matmul(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 70., 80., 90. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_matmul_transpose() {
        let vector:Vec<I32F32> = vec_to_fixed( &vec![ 1., 2., 3.] );
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let result = epoch::matmul_transpose(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 14., 32., 50., 68. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_sparse_matmul() {
        let vector:Vec<I32F32> = vec_to_fixed( &vec![ 1., 2., 3., 4.] );
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = epoch::sparse_matmul(&matrix, &vector, 3);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 70., 80., 90. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = epoch::sparse_matmul(&matrix, &vector, 3);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 69., 70., 63. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = epoch::sparse_matmul(&matrix, &vector, 3);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 0., 0., 0. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_sparse_matmul_transpose() {
        let vector:Vec<I32F32> = vec_to_fixed( &vec![ 1., 2., 3.] );
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = epoch::sparse_matmul_transpose(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 14., 32., 50., 68. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = epoch::sparse_matmul_transpose(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 13., 22., 23., 68. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = epoch::sparse_matmul_transpose(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 0., 0., 0., 0. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_sparse_clip() {
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let target:Vec<f32> = vec![ 0., 1., 1., 
                                    1., 1., 1., 
                                    1., 100., 100.,
                                    100., 100., 100.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = epoch::sparse_clip(&matrix, I32F32::from_num(8), I32F32::from_num(100), I32F32::from_num(1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_clip() {
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let target:Vec<f32> = vec![ 1., 1., 1., 
                                    1., 1., 1., 
                                    1., 100., 100.,
                                    100., 100., 100.];
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = epoch::clip(&matrix, I32F32::from_num(8), I32F32::from_num(100), I32F32::from_num(1));
        assert_mat_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_inplace_clip() {
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let mut matrix = vec_to_mat_fixed(&matrix, 4, false);
        let target:Vec<f32> = vec![ 1., 1., 1., 
                                    1., 1., 1., 
                                    1., 100., 100.,
                                    100., 100., 100.];
        let target = vec_to_mat_fixed(&target, 4, false);
        epoch::inplace_clip(&mut matrix, I32F32::from_num(8), I32F32::from_num(100), I32F32::from_num(1));
        assert_mat_compare(&matrix, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_mat_ema() {
        let old: Vec<f32> = vec![   1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let new: Vec<f32> = vec![   10., 20., 30., 
                                    40., 50., 60., 
                                    70., 80., 90.,
                                    100., 110., 120.];
        let target: Vec<f32> = vec![1.9, 3.8, 5.7, 
                                    7.6, 9.5, 11.4, 
                                    13.3, 15.2, 17.1,
                                    19., 20.9, 22.8];
        let old = vec_to_mat_fixed(&old, 4, false);
        let new = vec_to_mat_fixed(&new, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = epoch::mat_ema(&new, &old, I32F32::from_num(0.1));
        assert_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let old: Vec<f32> = vec![   1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let new: Vec<f32> = vec![   10., 20., 30., 
                                    40., 50., 60., 
                                    70., 80., 90.,
                                    100., 110., 120.];
        let target: Vec<f32> = vec![1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let old = vec_to_mat_fixed(&old, 4, false);
        let new = vec_to_mat_fixed(&new, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = epoch::mat_ema(&new, &old, I32F32::from_num(0));
        assert_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let old: Vec<f32> = vec![   1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let new: Vec<f32> = vec![   10., 20., 30., 
                                    40., 50., 60., 
                                    70., 80., 90.,
                                    100., 110., 120.];
        let target: Vec<f32> = vec![10., 20., 30., 
                                    40., 50., 60., 
                                    70., 80., 90.,
                                    100., 110., 120.];
        let old = vec_to_mat_fixed(&old, 4, false);
        let new = vec_to_mat_fixed(&new, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = epoch::mat_ema(&new, &old, I32F32::from_num(1));
        assert_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
    }

    #[test]
    fn test_math_sparse_mat_ema() {
        let old: Vec<f32> = vec![   1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let new: Vec<f32> = vec![   10., 20., 30., 
                                    40., 50., 60., 
                                    70., 80., 90.,
                                    100., 110., 120.];
        let target: Vec<f32> = vec![1.9, 3.8, 5.7, 
                                    7.6, 9.5, 11.4, 
                                    13.3, 15.2, 17.1,
                                    19., 20.9, 22.8];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = epoch::sparse_mat_ema(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let old: Vec<f32> = vec![   0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.,
                                    10., 11., 12.];
        let new: Vec<f32> = vec![   10., 20., 0., 
                                    40., 0., 60., 
                                    0., 80., 90.,
                                    100., 110., 120.];
        let target: Vec<f32> = vec![1., 3.8, 2.7, 
                                    7.6, 0., 11.4, 
                                    6.3, 15.2, 9.,
                                    19., 20.9, 22.8];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = epoch::sparse_mat_ema(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let old: Vec<f32> = vec![   0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let new: Vec<f32> = vec![   10., 20., 0., 
                                    40., 0., 60., 
                                    0., 80., 90.,
                                    100., 110., 120.];
        let target: Vec<f32> = vec![1., 2., 0., 
                                    4., 0., 6., 
                                    0., 8., 9.,
                                    10., 11., 12.];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = epoch::sparse_mat_ema(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let old: Vec<f32> = vec![   0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let new: Vec<f32> = vec![   0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = epoch::sparse_mat_ema(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let old: Vec<f32> = vec![   1., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let new: Vec<f32> = vec![   0., 0., 0., 
                                    0., 2., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let target: Vec<f32> = vec![0.9, 0., 0., 
                                    0., 0.2, 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = epoch::sparse_mat_ema(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
    }

    #[test]
    fn test_math_matmul2() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let w: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(1.0);3 ]; 3 ]; 
        assert_vec_compare( &epoch::matmul( &w, &vec![ I32F32::from_num(1.0); 3] ), &vec![ I32F32::from_num(3),  I32F32::from_num(3),  I32F32::from_num(3)], epsilon );
        assert_vec_compare( &epoch::matmul( &w, &vec![ I32F32::from_num(2.0); 3] ), &vec![ I32F32::from_num(6),  I32F32::from_num(6),  I32F32::from_num(6)], epsilon );
        assert_vec_compare( &epoch::matmul( &w, &vec![ I32F32::from_num(3.0); 3] ), &vec![ I32F32::from_num(9),  I32F32::from_num(9),  I32F32::from_num(9)], epsilon );
        assert_vec_compare( &epoch::matmul( &w, &vec![ I32F32::from_num(-1.0); 3] ), &vec![ I32F32::from_num(-3),  I32F32::from_num(-3),  I32F32::from_num(-3)], epsilon );
        let w: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(-1.0);3 ]; 3 ]; 
        assert_vec_compare( &epoch::matmul( &w, &vec![ I32F32::from_num(1.0); 3] ), &vec![ I32F32::from_num(-3),  I32F32::from_num(-3),  I32F32::from_num(-3)], epsilon );
        assert_vec_compare( &epoch::matmul( &w, &vec![ I32F32::from_num(2.0); 3] ), &vec![ I32F32::from_num(-6),  I32F32::from_num(-6),  I32F32::from_num(-6)], epsilon );
        assert_vec_compare( &epoch::matmul( &w, &vec![ I32F32::from_num(3.0); 3] ), &vec![ I32F32::from_num(-9),  I32F32::from_num(-9),  I32F32::from_num(-9)], epsilon );
        assert_vec_compare( &epoch::matmul( &w, &vec![ I32F32::from_num(-1.0); 3] ), &vec![ I32F32::from_num(3),  I32F32::from_num(3),  I32F32::from_num(3)], epsilon );
        let w: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(1.0);3 ], vec![ I32F32::from_num(2.0); 3], vec![ I32F32::from_num(3.0);3 ] ]; 
        assert_vec_compare( &epoch::matmul( &w, &vec![ I32F32::from_num(0.0); 3] ), &vec![ I32F32::from_num(0.0),  I32F32::from_num(0.0),  I32F32::from_num(0.0)], epsilon );
        assert_vec_compare( &epoch::matmul( &w, &vec![ I32F32::from_num(2.0); 3] ), &vec![ I32F32::from_num(12),  I32F32::from_num(12),  I32F32::from_num(12)], epsilon );
        let w: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(1), I32F32::from_num(2), I32F32::from_num(3) ]; 3 ]; 
        assert_vec_compare( &epoch::matmul( &w, &vec![ I32F32::from_num(0.0); 3] ), &vec![ I32F32::from_num(0.0),  I32F32::from_num(0.0),  I32F32::from_num(0.0)], epsilon );
        assert_vec_compare( &epoch::matmul( &w, &vec![ I32F32::from_num(2.0); 3] ), &vec![ I32F32::from_num(6),  I32F32::from_num(12),  I32F32::from_num(18)], epsilon );
    }

}