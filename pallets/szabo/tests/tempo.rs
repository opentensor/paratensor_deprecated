use crate::{mock::*};
mod mock;
mod helpers;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use frame_support::assert_ok;


#[cfg(feature = "no_std")]
use ndarray::{ndarray::Array1, ndarray::Array2, ndarray::arr1 };

#[allow(dead_code)]
pub fn print_network_state( netuid: u16 ) {
    println!( "S: {}", SzaboModule::get_stake_as_array( netuid ) );
    println!( "Sn: {}", SzaboModule::get_stake_as_float_array( netuid ) );
    println!( "W: {}", SzaboModule::get_weights_as_float_matrix( netuid ) );
    println!( "B: {}", SzaboModule::get_bonds_as_float_matrix( netuid ) );
    println!( "R: {}", SzaboModule::get_rank_as_float_array( netuid ) );
    println!( "T: {}", SzaboModule::get_trust_as_float_array( netuid ) );
    println!( "C: {}", SzaboModule::get_consensus_as_float_array( netuid ) );
    println!( "I: {}", SzaboModule::get_incentive_as_float_array( netuid ) );
    println!( "D: {}", SzaboModule::get_dividends_as_float_array( netuid ) );
    println!( "E: {}", SzaboModule::get_emission_as_array( netuid ) );
}
#[allow(dead_code)]
pub fn create_random_subgraph( netuid: u16, n: u16, tempo: u64 ) {
    let seed: &[u8; 32] = &[1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4];
    let mut rng: StdRng  = SeedableRng::from_seed(*seed);
    for i in 0..n {
        SzaboModule::add_global_account( &(i as u64), &(i as u64) );
        let random_stake = rng.gen_range(0..100);
        SzaboModule::add_balance_to_coldkey_account( &(i as u64), random_stake );
        assert_ok!(SzaboModule::add_stake(Origin::signed(i as u64), i as u64, random_stake));
        SzaboModule::add_subnetwork_account( netuid, i, &(i as u64) );
    }
    let nu: u64 = n as u64;
    let mut weights_vec: Vec<f32> = vec![0.0; (nu * nu) as usize];
    for i in 0..nu*nu {
        let rw: u16 = rng.gen_range(0..100);
        weights_vec[i as usize] = rw as f32 / 100.0;
    }
    let mut warr: ndarray::Array2<f32> = ndarray::Array2::from_shape_vec( (n as usize, n as usize), weights_vec ).unwrap();
    for row in warr.rows_mut() {
        let sum: f32 = row.sum(); if sum > 0.0 { row.mapv_into(|x| x/sum); }
    }
    SzaboModule::set_weights_from_float_matrix( netuid, warr );

    let mut bonds_vec: Vec<f32> = vec![0.0; (nu * nu) as usize];
    for i in 0..nu*nu {
        let rw: u16 = rng.gen_range(0..100);
        bonds_vec[i as usize] = rw as f32 / 100.0;
    }
    let mut barr: ndarray::Array2<f32> = ndarray::Array2::from_shape_vec( (n as usize, n as usize), bonds_vec ).unwrap();
    for row in barr.rows_mut() {
        let sum: f32 = row.sum(); if sum > 0.0 {  row.mapv_into(|x| x/sum); }
    }
    SzaboModule::set_bonds_from_float_matrix( netuid, barr );
    SzaboModule::set_tempo_for_network( netuid, tempo );
    SzaboModule::set_emission_distribution_for_netuid( netuid, rand::thread_rng().gen_range(0..1000) as u16 );
    SzaboModule::increment_num_subnetworks();
}

#[test]
fn test_random_network() { new_test_ext().execute_with(|| {
        create_random_subgraph( 0, 2, 1);
        print_network_state( 0 );
        SzaboModule::epoch(0,1000, true);
        print_network_state( 0 );
        helpers::assert_u64_array_eq( &SzaboModule::get_stake_as_array(0), &ndarray::arr1( &[57, 90] ), 0 );
        helpers::assert_f32_array_eq( &SzaboModule::get_rank_as_float_array(0), &ndarray::arr1::<f32>( &[0.792401, 0.20756848] ), 0.01 );
        helpers::assert_f32_array_eq( &SzaboModule::get_trust_as_float_array(0), &ndarray::arr1( &[1.0, 0.38774702] ), 0.01 );
        helpers::assert_f32_array_eq( &SzaboModule::get_consensus_as_float_array(0), &ndarray::arr1( &[0.993286, 0.24528877] ), 0.01 );
        helpers::assert_f32_array_eq( &SzaboModule::get_incentive_as_float_array(0), &ndarray::arr1( &[0.78709084, 0.0509041] ), 0.01 );
        helpers::assert_f32_array_eq( &SzaboModule::get_dividends_as_float_array(0), &ndarray::arr1( &[0.384329, 0.45037004] ), 0.01 );
        helpers::assert_u64_array_eq( &SzaboModule::get_emission_as_array(0), &ndarray::arr1( &[384, 450] ), 0 );
});
}
/*
#[test]
fn test_params_from_empty_network() {
	new_test_ext().execute_with(|| {
        assert!( SzaboModule::get_num_subnetworks() == 0 );
        helpers::assert_u64_array_eq( &SzaboModule::get_pending_emission_as_array(), &ndarray::Array1::<u64>::zeros( 0 ), 0 );
        helpers::assert_u64_array_eq( &SzaboModule::get_tempo_as_array(), &ndarray::Array1::<u64>::zeros( 0 ), 0 );
	});
}
#[test]
fn test_params_with_cardinal_network() {
	new_test_ext().execute_with(|| {
        create_random_subgraph(0, 4, 1);
        assert!( SzaboModule::get_num_subnetworks() == 1 );
        helpers::assert_u64_array_eq( &SzaboModule::get_pending_emission_as_array(), &ndarray::Array1::<u64>::zeros( 1 ), 0 );
        helpers::assert_u64_array_eq( &SzaboModule::get_tempo_as_array(), &ndarray::Array1::<u64>::ones( 1 ), 0 );
	});
}

#[test]
fn test_params_with_cardinal_and_secondary_network() {
	new_test_ext().execute_with(|| {
        create_random_subgraph(0, 4, 1);
        create_random_subgraph(1, 4, 2);
        assert!( SzaboModule::get_num_subnetworks() == 2 );
        helpers::assert_u64_array_eq( &SzaboModule::get_pending_emission_as_array(), &ndarray::Array1::<u64>::zeros( 2 ), 0 );
        helpers::assert_u64_array_eq( &SzaboModule::get_tempo_as_array(), &ndarray::arr1( &[1,2] ), 0 );
	});
}
#[test]
fn test_params_with_cardinal_and_secondary_network_run_distribution() {
	new_test_ext().execute_with(|| {
        create_random_subgraph(0, 4, 1);
        create_random_subgraph(1, 4, 2);
        assert!( SzaboModule::get_num_subnetworks() == 2 );
        helpers::assert_u64_array_eq( &SzaboModule::get_pending_emission_as_array(), &ndarray::Array1::<u64>::zeros( 2 ), 0 );
        helpers::assert_u64_array_eq( &SzaboModule::get_tempo_as_array(), &ndarray::arr1( &[1,2] ), 0 );
        SzaboModule::update_pending_emission( true );
        SzaboModule::run_epochs( true );
        });
}
#[test]
fn test_params_with_cardinal_and_multiple_networks() {
	new_test_ext().execute_with(|| {
        create_random_subgraph(0, 4, 1);
        create_random_subgraph(1, 4, 1);
        create_random_subgraph(2, 4, 2);
        create_random_subgraph(3, 4, 2);
        create_random_subgraph(4, 4, 3);
        create_random_subgraph(5, 4, 3);
        create_random_subgraph(6, 4, 4);
        create_random_subgraph(7, 4, 4);
        assert!( SzaboModule::get_num_subnetworks() == 8 );
        helpers::assert_u64_array_eq( &SzaboModule::get_pending_emission_as_array(), &ndarray::Array1::<u64>::zeros( 8 ), 0 );
        helpers::assert_u64_array_eq( &SzaboModule::get_tempo_as_array(), &ndarray::arr1( &[1,1,2,2,3,3,4,4] ), 0 );
        SzaboModule::global_step( true );
        helpers::assert_u64_array_eq( &SzaboModule::get_pending_emission_as_array(), &ndarray::arr1( &[0, 0, 9689479, 9689479, 2899214, 9567407, 8285649, 10040437] ), 0 );
        SzaboModule::global_step( true );
});
}


#[test]
fn test_tempo_no_graph() {
	new_test_ext().execute_with(|| {
		SzaboModule::update_pending_emission( true );
	});
}

#[test]
fn test_tempo_with_cardinal_graph() {
	new_test_ext().execute_with(|| {
                create_random_subgraph(0, 4, 1);
		SzaboModule::update_pending_emission( true );
	});
}
*/






