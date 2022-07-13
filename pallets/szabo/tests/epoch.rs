use crate::{mock::*};
use frame_support::assert_ok;
use rand::Rng;

#[cfg(feature = "no_std")]
use ndarray::{ndarray::Array1, ndarray::Array2, ndarray::arr1};

mod mock;
mod helpers;

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
	for i in 0..n {
		SzaboModule::add_global_account( &(i as u64), &(i as u64) );
		let random_stake = rand::thread_rng().gen_range(0..100);
		SzaboModule::add_balance_to_coldkey_account( &(i as u64), random_stake );
		assert_ok!(SzaboModule::add_stake(Origin::signed(i as u64), i as u64, random_stake));
		SzaboModule::add_subnetwork_account( netuid, i, &(i as u64) );
	}
	let nu: u64 = n as u64;
	let mut weights_vec: Vec<f32> = vec![0.0; (nu * nu) as usize];
	for i in 0..nu*nu {
		let rw: u16 = rand::thread_rng().gen_range(0..100);
		weights_vec[i as usize] = rw as f32 / 100.0;
	}
	let mut warr: ndarray::Array2<f32> = ndarray::Array2::from_shape_vec( (n as usize, n as usize), weights_vec ).unwrap();
	for row in warr.rows_mut() {
		let sum: f32 = row.sum(); if sum > 0.0 { row.mapv_into(|x| x/sum); }
	}
	SzaboModule::set_weights_from_float_matrix( netuid, warr );

	let mut bonds_vec: Vec<f32> = vec![0.0; (nu * nu) as usize];
	for i in 0..nu*nu {
		let rw: u16 = rand::thread_rng().gen_range(0..100);
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

/*
#[test]
fn test_random_graphs() {
	new_test_ext().execute_with(|| {
		helpers::create_random_subgraph(0, 2, 1);
        helpers::print_network_state( 0 );
		helpers::create_random_subgraph(1, 4, 1);
        helpers::print_network_state( 1 );
		helpers::create_random_subgraph(2, 10, 1);
        helpers::print_network_state( 2 );
	});
}

#[test]
fn test_nill_epoch() {
	new_test_ext().execute_with(|| {
		SzaboModule::epoch(0,0);
        SzaboModule::epoch(1,1000);
	});
}



#[test]
fn test_epoch_n2() {
	new_test_ext().execute_with(|| {
		helpers::create_random_subgraph(0, 2, 1);
		SzaboModule::epoch( 0, 1_000_000_000, false );
	});
}

*/

#[test]
fn test_epoch_n100() {
	new_test_ext().execute_with(|| {
		create_random_subgraph(0, 4096, 1);
		for i in 0..10 {
			SzaboModule::epoch( 0, 1_000_000_000, false );
			println!("{}", i);
		}
	});
}
