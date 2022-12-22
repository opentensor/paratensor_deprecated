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

        let value  =  ( ParatensorModule::get_float_kappa(netuid)  *  I32F32::from_num( u16::MAX )).to_num::<u16>() + 1;
        assert_eq!(value , kappa); 
    });
}

#[test]
fn test_sudo_set_max_allowed_uid() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 10;
        let max_allowed_uids: u16 = 10;
		assert_ok!(ParatensorModule::sudo_set_max_allowed_uids(<<Test as Config>::Origin>::root(), netuid, max_allowed_uids));
        
        match ParatensorModule::get_max_allowed_uids(netuid) {
                Ok(k) => assert_eq!(k, max_allowed_uids),
                Err(_e) => panic!(),
            } 
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

/// -------- tests for PendingEmissionValues --------
#[test]
fn test_sudo_test_tempo_pending_emissions_ok() {
	new_test_ext().execute_with(|| {
        let netuid0: u16 = 1;
        let netuid1: u16 = 2;
        let netuid2: u16 = 3;
        let netuid3: u16 = 5;
        let tempo0: u16 = 1;
        let tempo1: u16 = 2;
        let tempo2: u16 = 3;
        let tempo3: u16 = 5;
        add_network(netuid0, tempo0, 0);
		add_network(netuid1, tempo1, 0);
        add_network(netuid2, tempo2, 0);
        add_network(netuid3, tempo3, 0);
        assert_eq!(ParatensorModule::get_tempo(netuid0), tempo0);
        assert_eq!(ParatensorModule::get_tempo(netuid1), tempo1);
        assert_eq!(ParatensorModule::get_tempo(netuid2), tempo2);
        assert_eq!(ParatensorModule::get_tempo(netuid3), tempo3);
        assert_eq!(ParatensorModule::get_emission_value(netuid0), 0);
        assert_eq!(ParatensorModule::get_emission_value(netuid1), 0);
        assert_eq!(ParatensorModule::get_emission_value(netuid2), 0);
        assert_eq!(ParatensorModule::get_emission_value(netuid3), 0);
        let emission_values: Vec<(u16, u64)> = vec![(1, 100000000),(2, 400000000), (3, 200000000), (5, 300000000)]; 
        assert_ok!(ParatensorModule::sudo_set_emission_values(<<Test as Config>::Origin>::root(), emission_values));
        assert_eq!(ParatensorModule::get_emission_value(netuid0), 100000000);
        assert_eq!(ParatensorModule::get_emission_value(netuid1), 400000000);
        assert_eq!(ParatensorModule::get_emission_value(netuid2), 200000000);
        assert_eq!(ParatensorModule::get_emission_value(netuid3), 300000000);
        assert_eq!(ParatensorModule::get_pending_emission(netuid0), 0);
        assert_eq!(ParatensorModule::get_pending_emission(netuid1), 0);
        assert_eq!(ParatensorModule::get_pending_emission(netuid2), 0);
        assert_eq!(ParatensorModule::get_pending_emission(netuid3), 0);
        
    });
}

#[test]
pub fn test_sudo_test_pending_emission_ok() {
    new_test_ext().execute_with(|| {
        let netuid1: u16 = 1;
        let tempo1: u16 = 5;

        let netuid2: u16 = 2;
        let tempo2: u16 = 7;

        let emission_values: Vec<(u16, u64)> = vec![(1, 250000000),(2, 750000000)]; 

        add_network(netuid1, tempo1, 0);
        add_network(netuid2, tempo2, 0);

        assert_ok!(ParatensorModule::sudo_set_emission_values(<<Test as Config>::Origin>::root(), emission_values));
        assert_eq!(ParatensorModule::get_emission_value(netuid1), 250000000);

        step_block(3);

        assert_eq!(ParatensorModule::get_pending_emission(netuid1), 750000000); // 250000000 + 250000000 + 250000000
        assert_eq!(ParatensorModule::get_pending_emission(netuid2), 2250000000); // 750000000 + 750000000 + 750000000
    });
}

