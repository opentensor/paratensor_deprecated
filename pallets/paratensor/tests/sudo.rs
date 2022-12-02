use frame_support::{assert_ok};
use frame_system::Config;
mod mock;
use mock::*;
use frame_support::sp_runtime::DispatchError;
use substrate_fixed::types::I32F32;

#[test]
fn test_sudo_set_rho() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 10;
        let rho: u16 = 11;
		assert_ok!(ParatensorModule::sudo_set_rho(<<Test as Config>::Origin>::root(), netuid, rho));
        assert_eq!(ParatensorModule::get_rho(netuid), rho);
    });
}

#[test]
fn test_sudo_set_blocks_per_step() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let blocks_per_step: u16 = 10;
		assert_ok!(ParatensorModule::sudo_set_blocks_per_step(<<Test as Config>::Origin>::root(), netuid, blocks_per_step));
        assert_eq!(ParatensorModule::get_blocks_per_step(netuid), blocks_per_step);
    });
}

#[test]
fn test_sudo_set_emission_ratio() {
	new_test_ext().execute_with(|| {
        /*TO DO */
    });
}

#[test]
fn test_sudo_set_bonds_moving_average () {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 10;
        let bonds_moving_average: u64 = 10;
		assert_ok!(ParatensorModule::sudo_set_bonds_moving_average(<<Test as Config>::Origin>::root(), netuid, bonds_moving_average));
        assert_eq!(ParatensorModule::get_bonds_moving_average(netuid), bonds_moving_average);
    });
}

#[test]
fn test_sudo_set_difficulty () {
	new_test_ext().execute_with(|| {
        let difficulty: u64 = 10;
        let netuid: u16 = 10;
		assert_ok!(ParatensorModule::sudo_set_difficulty(<<Test as Config>::Origin>::root(), netuid, difficulty));
        assert_eq!(ParatensorModule::get_difficulty_as_u64(netuid), difficulty);
    });
}

#[test]
fn test_sudo_set_adjustment_interval() {
	new_test_ext().execute_with(|| {
        let adjustment_interval: u16 = 10;
        let netuid: u16 = 10;
		assert_ok!(ParatensorModule::sudo_set_adjustment_interval(<<Test as Config>::Origin>::root(), netuid, adjustment_interval));
        assert_eq!(ParatensorModule::get_adjustment_interval(netuid), adjustment_interval);
    });
}

#[test]
fn test_sudo_set_target_registrations_per_interval() {
	new_test_ext().execute_with(|| {
        let target_registrations_per_interval: u16 = 10;
        let netuid: u16 = 10;
		assert_ok!(ParatensorModule::sudo_set_target_registrations_per_interval(<<Test as Config>::Origin>::root(), netuid, target_registrations_per_interval));
        assert_eq!(ParatensorModule::get_target_registrations_per_interval(netuid), target_registrations_per_interval);
    });
}

#[test]
fn test_sudo_set_activity_cutoff() {
	new_test_ext().execute_with(|| {
        let activity_cutoff: u16 = 10;
        let netuid: u16 = 10;
        let init_activity_cutoff: u16 = ParatensorModule::get_activity_cutoff(netuid);
		assert_eq!(ParatensorModule::sudo_set_activity_cutoff(<<Test as Config>::Origin>::signed(0), netuid, activity_cutoff),  Err(DispatchError::BadOrigin.into()));
        assert_eq!(ParatensorModule::get_activity_cutoff(netuid), init_activity_cutoff);
    });
}

#[test]
fn test_sudo_set_kappa() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let kappa: u16 = 5;
        
		assert_ok!(ParatensorModule::sudo_set_kappa(<<Test as Config>::Origin>::root(), netuid, kappa));

        // let value  =  ( ParatensorModule::get_kappa(netuid)  *  I32F32::from_num( u16::MAX )).to_num::<u16>() + 1;
        // assert_eq!(value , kappa); 
    });
}

#[test]
fn test_sudo_set_max_allowed_uid() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 10;
        let max_allowed_uids: u16 = 10;
		assert_ok!(ParatensorModule::sudo_set_max_allowed_uids(<<Test as Config>::Origin>::root(), netuid, max_allowed_uids));
        assert_eq!(ParatensorModule::get_max_allowed_uids(netuid), max_allowed_uids);
    });
}

#[test]
fn test_sudo_set_min_allowed_weights() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 10;
        let min_allowed_weights: u16 = 1;
		assert_ok!(ParatensorModule::sudo_set_min_allowed_weights(<<Test as Config>::Origin>::root(), netuid, min_allowed_weights));
        assert_eq!(ParatensorModule::get_min_allowed_weights(netuid), min_allowed_weights);
    });
}

#[test]
fn test_sudo_set_max_allowed_max_min_ratio() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let max_min_ratio: u16 = 10;
        assert_ok!(ParatensorModule::sudo_set_max_allowed_max_min_ratio(<<Test as Config>::Origin>::root(), netuid, max_min_ratio));
        assert_eq!(ParatensorModule::get_max_allowed_max_min_ratio(netuid), max_min_ratio);
    });
}

#[test]
fn test_sudo_set_validator_batch_size() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let validator_batch_size: u16 = 10;
		assert_ok!(ParatensorModule::sudo_set_validator_batch_size(<<Test as Config>::Origin>::root(), netuid, validator_batch_size));
        assert_eq!(ParatensorModule::get_validator_batch_size(netuid), validator_batch_size);
    });
}

#[test]
fn test_sudo_set_validator_sequence_length() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let validator_sequence_length: u16 = 10;
		assert_ok!(ParatensorModule::sudo_set_validator_sequence_length(<<Test as Config>::Origin>::root(), netuid, validator_sequence_length));
        assert_eq!(ParatensorModule::get_validator_sequence_length(netuid), validator_sequence_length);
    });
}

#[test]
fn test_sudo_set_validator_epochs_per_reset() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let validator_epochs_per_reset: u16= 10;
        let init_validator_epochs_per_reset: u16 = ParatensorModule::get_validator_epochs_per_reset(netuid);
		assert_eq!(ParatensorModule::sudo_set_validator_epochs_per_reset(<<Test as Config>::Origin>::signed(0), netuid, validator_epochs_per_reset),  Err(DispatchError::BadOrigin.into()));
        assert_eq!(ParatensorModule::get_validator_epochs_per_reset(netuid), init_validator_epochs_per_reset);
    });
}

#[test]
fn test_sudo_set_incentive_pruning_denominator() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let incentive_pruning_denominator: u16 = 10;
		assert_ok!(ParatensorModule::sudo_set_incentive_pruning_denominator(<<Test as Config>::Origin>::root(), netuid, incentive_pruning_denominator));
        assert_eq!(ParatensorModule::get_incentive_pruning_denominator(netuid), incentive_pruning_denominator);
    });
}

#[test]
fn test_sudo_set_stake_pruning_denominator() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let stake_pruning_denominator: u16 = 10;
		assert_ok!(ParatensorModule::sudo_set_stake_pruning_denominator(<<Test as Config>::Origin>::root(), netuid, stake_pruning_denominator));
        assert_eq!(ParatensorModule::get_stake_pruning_denominator(netuid), stake_pruning_denominator);
    });
}

#[test]
fn test_sudo_set_immunity_period() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let immunity_period: u16 = 10;
		assert_ok!(ParatensorModule::sudo_set_immunity_period(<<Test as Config>::Origin>::root(), netuid, immunity_period));
        assert_eq!(ParatensorModule::get_immunity_period(netuid), immunity_period);
    });
}

#[test]
fn test_sudo_set_max_weight_limit() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let max_weight_limit: u16 = 10;
        let init_max_weight_limit: u16 = ParatensorModule::get_max_weight_limit(netuid);
		assert_eq!(ParatensorModule::sudo_set_max_weight_limit(<<Test as Config>::Origin>::signed(0), netuid, max_weight_limit),  Err(DispatchError::BadOrigin.into()));
        assert_eq!(ParatensorModule::get_max_weight_limit(netuid), init_max_weight_limit);
    });
}

#[test]
fn test_sudo_validator_exclude_quantile() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let validator_exclude_quantile: u16 = 10;
		assert_ok!(ParatensorModule::sudo_set_validator_exclude_quantile(<<Test as Config>::Origin>::root(), netuid, validator_exclude_quantile));
        assert_eq!(ParatensorModule::get_validator_exclude_quantile(netuid), validator_exclude_quantile);
    });
}

