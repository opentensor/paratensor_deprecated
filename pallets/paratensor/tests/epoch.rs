use crate::{mock::*};
use rand::Rng;
use std::time::{Duration, Instant};

#[cfg(feature = "no_std")]
use ndarray::{ndarray::Array1, ndarray::Array2, ndarray::arr1};

mod mock;

#[allow(dead_code)]
pub fn print_network_state( netuid: u16, neuron_uid: u16 ) {
	//println!( "S: {:?}", ParatensorModule::get_stake( netuid, neuron_uid ) );
	//println!( "W: {:?}", ParatensorModule::get_weights( netuid, neuron_uid ) );
	//println!( "B: {:?}", ParatensorModule::get_bonds( netuid, neuron_uid ) );
	//println!( "R: {:?}", ParatensorModule::get_ranks( netuid, neuron_uid ) );
	//println!( "T: {:?}", ParatensorModule::get_trust( netuid, neuron_uid ) );
	//println!( "C: {:?}", ParatensorModule::get_consensus( netuid, neuron_uid ) );
	//println!( "I: {:?}", ParatensorModule::get_incentives( netuid, neuron_uid ) );
	//println!( "D: {:?}", ParatensorModule::get_dividends( netuid, neuron_uid ) );
}

#[allow(dead_code)]
pub fn create_random_subgraph( netuid: u16, n: u16, tempo: u64 ) {
	/*
	for i in 0..n {
		ParatensorModule::add_global_account( &(i as u64), &(i as u64) );
		let random_stake = rand::thread_rng().gen_range(0..100);
		ParatensorModule::add_balance_to_coldkey_account( &(i as u64), random_stake );
		assert_ok!(ParatensorModule::add_stake(Origin::signed(i as u64), i as u64, random_stake));
		ParatensorModule::add_subnetwork_account( netuid, i, &(i as u64) );
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
	ParatensorModule::set_weights_from_float_matrix( netuid, warr );

	let mut bonds_vec: Vec<f32> = vec![0.0; (nu * nu) as usize];
	for i in 0..nu*nu {
		let rw: u16 = rand::thread_rng().gen_range(0..100);
		bonds_vec[i as usize] = rw as f32 / 100.0;
	}
	let mut barr: ndarray::Array2<f32> = ndarray::Array2::from_shape_vec( (n as usize, n as usize), bonds_vec ).unwrap();
	for row in barr.rows_mut() {
		let sum: f32 = row.sum(); if sum > 0.0 {  row.mapv_into(|x| x/sum); }
	}
	//ParatensorModule::set_bonds_from_float_matrix( netuid, barr );
	//ParatensorModule::set_tempo_for_network( netuid, tempo );
	//ParatensorModule::set_emission_distribution_for_netuid( netuid, rand::thread_rng().gen_range(0..1000) as u16 );
	//ParatensorModule::increment_num_subnetworks();
	*/
}

#[test]
fn test_nill_epoch_paratensor() {
/* 	new_test_ext().execute_with(|| {
        println!( "test_nill_epoch:" );
		ParatensorModule::epoch( 0, 0, false );
	}); */
}

#[test]
fn test_many_epochs() {
/* 	new_test_ext().execute_with(|| {
        println!( "test_1000_epochs:" );
		let n = 10;
		let start:Instant = Instant::now();
		for _ in 0..n {
			ParatensorModule::epoch( 0, 0, false );
		}
		let finish: Instant = Instant::now();
		let duration: Duration = finish.duration_since(start);
		let avg_secs: f32 = duration.as_secs() as f32 / n as f32;
		println!("total: {:?}, avg_per:{}", duration.as_secs(), avg_secs );
	});  */
}

