use crate::{mock::*};
#[cfg(feature = "no_std")]
use ndarray::{ndarray::Array1, ndarray::Array2, ndarray::arr1};
use substrate_fixed::types::I32F32;
use frame_system::Config;
use frame_support::{assert_ok};
use std::time::{Instant};
mod mock;

#[test]
// Test an epoch on an empty graph.
fn test_nill_epoch_paratensor() {
	new_test_ext().execute_with(|| {
        println!( "test_nill_epoch:" );
		ParatensorModule::epoch( 0, 0, true );
	});
}

#[test]
// Test an epoch on a graph with a single item.
fn test_1_graph() {
	new_test_ext().execute_with(|| {
        println!( "test_1_graph:" );
		let netuid: u16 = 0;
		let coldkey: u64 = 0;
		let hotkey: u64 = 0;
		let uid: u16 = 0;
		let stake_amount: u64 = 1;
		ParatensorModule::add_balance_to_coldkey_account( &coldkey, stake_amount as u128 );
 		ParatensorModule::set_stake_for_testing( &hotkey, stake_amount );
		ParatensorModule::add_subnetwork_account( netuid, uid, &hotkey );
		assert_eq!( ParatensorModule::get_subnetwork_n(netuid), 1 );
		run_to_block( 1 ); // run to next block to ensure weights are set on nodes after their registration block
		assert_ok!(ParatensorModule::set_weights(Origin::signed(uid as u64), netuid, vec![ uid as u16 ], vec![ u16::MAX ]));
		// ParatensorModule::set_weights_for_testing( netuid, i as u16, vec![ ( 0, u16::MAX )]); // doesn't set update status
		// ParatensorModule::set_bonds_for_testing( netuid, uid, vec![ ( 0, u16::MAX )]); // rather, bonds are calculated in epoch
		ParatensorModule::epoch( 0, 1_000_000_000, true );
		assert_eq!( ParatensorModule::get_stake_for_hotkey( &hotkey ), stake_amount );
		assert_eq!( ParatensorModule::get_rank( netuid, uid ), 0 );
		assert_eq!( ParatensorModule::get_trust( netuid, uid ), 0 );
		assert_eq!( ParatensorModule::get_consensus( netuid, uid ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
		assert_eq!( ParatensorModule::get_incentive( netuid, uid ), 0 );
		assert_eq!( ParatensorModule::get_dividend( netuid, uid ), 0 );
		assert_eq!( ParatensorModule::get_emission( netuid, uid ), 0 );  // all self-weight masked out (TODO: decide emission for this case)
	});
}

#[test]
// Test an epoch on a graph with two items.
fn test_10_graph() {
	new_test_ext().execute_with(|| {
        println!( "test_10_graph" );
		// Function for adding a nodes to the graph.
		pub fn add_node( 
				netuid: u16,
				coldkey: u64, 
				hotkey:u64, 
				uid: u16, 
				stake_amount: u64
			){
			println!(
				"+Add net:{:?} coldkey:{:?} hotkey:{:?} uid:{:?} stake_amount: {:?} subn: {:?}", 
				netuid,
				coldkey,
				hotkey,
				uid,
				stake_amount,
				ParatensorModule::get_subnetwork_n(netuid),
			);
			ParatensorModule::add_balance_to_coldkey_account( &coldkey, stake_amount as u128 );
			ParatensorModule::set_stake_for_testing( &hotkey, stake_amount );
			ParatensorModule::add_subnetwork_account( netuid, uid, &hotkey );
			assert_eq!( ParatensorModule::get_subnetwork_n(netuid) - 1 , uid );
		}
		// Build the graph with 10 items 
		// each with 1 stake and self weights.
		let n: usize = 10;
		let netuid: u16 = 0;
		ParatensorModule::set_max_allowed_uids( netuid, n as u16 ); 
		for i in 0..10 {
			add_node(
				netuid,
				i as u64,
				i as u64,
				i as u16,
				1
			)
		}
		assert_eq!( ParatensorModule::get_subnetwork_n(netuid), 10 );
		run_to_block( 1 ); // run to next block to ensure weights are set on nodes after their registration block
		for i in 0..10 {
			assert_ok!(ParatensorModule::set_weights(Origin::signed(i), netuid, vec![ i as u16 ], vec![ u16::MAX ]));
			// ParatensorModule::set_weights_for_testing( netuid, i as u16, vec![ ( i as u16, u16::MAX )]); // doesn't set update status
			// ParatensorModule::set_bonds_for_testing( netuid, uid, vec![ ( i as u16, u16::MAX )]); // rather, bonds are calculated in epoch
		}
		// Run the epoch.
		ParatensorModule::epoch( 0, 1_000_000_000, true );
		// Check return values.
		for i in 0..n {
			assert_eq!( ParatensorModule::get_stake_for_hotkey( &(i as u64) ), 1 );
			assert_eq!( ParatensorModule::get_rank( netuid, i as u16 ), 0 );
			assert_eq!( ParatensorModule::get_trust( netuid, i as u16 ), 0 );
			assert_eq!( ParatensorModule::get_consensus( netuid, i as u16 ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
			assert_eq!( ParatensorModule::get_incentive( netuid, i as u16 ), 0 );
			assert_eq!( ParatensorModule::get_dividend( netuid, i as u16 ), 0 );
			assert_eq!( ParatensorModule::get_emission( netuid, i as u16 ), 0 );
		}
	});
}

fn uid_stats(netuid: u16, uid: u16) {
	println!( "stake: {:?}", ParatensorModule::get_stake_for_hotkey( &(uid as u64) ));
	println!( "rank: {:?}", ParatensorModule::get_rank( netuid, uid ));
	println!( "trust: {:?}", ParatensorModule::get_trust( netuid, uid ));
	println!( "consensus: {:?}", ParatensorModule::get_consensus( netuid, uid ));
	println!( "incentive: {:?}", ParatensorModule::get_incentive( netuid, uid ));
	println!( "dividend: {:?}", ParatensorModule::get_dividend( netuid, uid ));
	println!( "emission: {:?}", ParatensorModule::get_emission( netuid, uid ));
}

#[test]
/// Test an epoch on a graph with 512 nodes, of which the first 64 are validators setting non-self weights, and the rest servers setting only self-weights.
fn test_512_graph() {
	new_test_ext().execute_with(|| {
		let netuid: u16 = 0;
		let n: u16 = 512;
		let validators: u16 = 64;
		let servers = n - validators;
		let epochs: u16 = 100;
		println!( "test_{n:?}_graph ({validators:?} validators)" );
		init_run_epochs(netuid, n, validators, servers, epochs, false);
		let bonds = ParatensorModule::get_bonds( netuid );
		for uid in 0..validators { // validators
			assert_eq!( ParatensorModule::get_stake_for_hotkey( &(uid as u64) ), 1 );
			assert_eq!( ParatensorModule::get_rank( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_trust( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_consensus( netuid, uid ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
			assert_eq!( ParatensorModule::get_incentive( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_dividend( netuid, uid ), 1023 ); // Note D = floor(1 / 64 * 65_535) = 1023
			assert_eq!( ParatensorModule::get_emission( netuid, uid ), 7812485 ); // Note E = 0.5 / 200 * 1_000_000_000 = 7_812_500 (discrepancy)
			assert_eq!( bonds[uid as usize][0], 0.0 );
			assert_eq!( bonds[uid as usize][validators as usize], I32F32::from_num(1023) / I32F32::from_num(65_535) ); // Note B_ij = floor(1 / 64 * 65_535) / 65_535 = 1023 / 65_535
		}
		for uid in validators..n { // servers
			assert_eq!( ParatensorModule::get_stake_for_hotkey( &(uid as u64) ), 0 );
			assert_eq!( ParatensorModule::get_rank( netuid, uid ), 146 ); // Note R = floor(1 / (512 - 64) * 65_535) = 146
			assert_eq!( ParatensorModule::get_trust( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_consensus( netuid, uid ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
			assert_eq!( ParatensorModule::get_incentive( netuid, uid ), 146 ); // Note I = floor(1 / (512 - 64) * 65_535) = 146
			assert_eq!( ParatensorModule::get_dividend( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_emission( netuid, uid ), 1116073 ); // Note E = floor(0.5 / (512 - 64) * 1_000_000_000) = 1_116_071 (discrepancy)
			assert_eq!( bonds[uid as usize][0], 0.0 );
			assert_eq!( bonds[uid as usize][validators as usize], 0.0 );
		}
	});
}

#[test]
/// Test an epoch on a graph with 4096 nodes, of which the first 200 are validators setting non-self weights, and the rest servers setting only self-weights.
fn test_4096_graph() {
	new_test_ext().execute_with(|| {
		let netuid: u16 = 0;
		let n: u16 = 4096;
		let validators: u16 = 200;
		let servers = n - validators;
		let epochs: u16 = 1;
		println!( "test_{n:?}_graph ({validators:?} validators)" );
		init_run_epochs(netuid, n, validators, servers, epochs, false);
		let bonds = ParatensorModule::get_bonds( netuid );
		for uid in 0..validators { // validators
			assert_eq!( ParatensorModule::get_stake_for_hotkey( &(uid as u64) ), 1 );
			assert_eq!( ParatensorModule::get_rank( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_trust( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_consensus( netuid, uid ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
			assert_eq!( ParatensorModule::get_incentive( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_dividend( netuid, uid ), 327 ); // Note D = floor(1 / 200 * 65_535) = 327
			assert_eq!( ParatensorModule::get_emission( netuid, uid ), 2499995 ); // Note E = 0.5 / 200 * 1_000_000_000 = 2_500_000 (discrepancy)
			assert_eq!( bonds[uid as usize][0], 0.0 );
			assert_eq!( bonds[uid as usize][validators as usize], I32F32::from_num(327) / I32F32::from_num(65_535) ); // Note B_ij = floor(1 / 200 * 65_535) / 65_535 = 327 / 65_535
		}
		for uid in validators..n { // servers
			assert_eq!( ParatensorModule::get_stake_for_hotkey( &(uid as u64) ), 0 );
			assert_eq!( ParatensorModule::get_rank( netuid, uid ), 16 ); // Note R = floor(1 / (4096 - 200) * 65_535) = 16
			assert_eq!( ParatensorModule::get_trust( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_consensus( netuid, uid ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
			assert_eq!( ParatensorModule::get_incentive( netuid, uid ), 16 ); // Note I = floor(1 / (4096 - 200) * 65_535) = 16
			assert_eq!( ParatensorModule::get_dividend( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_emission( netuid, uid ), 128336 ); // Note E = floor(0.5 / (4096 - 200) * 1_000_000_000) = 128336
			assert_eq!( bonds[uid as usize][0], 0.0 );
			assert_eq!( bonds[uid as usize][validators as usize], 0.0 );
		}
	});
}

#[test]
/// Test an epoch_sparse on a graph with 4096 nodes, of which the first 200 are validators setting non-self weights, and the rest servers setting only self-weights.
fn test_4096_graph_sparse() {
	new_test_ext().execute_with(|| {
		let netuid: u16 = 0;
		let n: u16 = 4096;
		let validators: u16 = 200;
		let servers = n - validators;
		let epochs: u16 = 1;
		println!( "test_{n:?}_graph ({validators:?} validators)" );
		init_run_epochs(netuid, n, validators, servers, epochs, true);
		let bonds = ParatensorModule::get_bonds( netuid );
		for uid in 0..validators { // validators
			assert_eq!( ParatensorModule::get_stake_for_hotkey( &(uid as u64) ), 1 );
			assert_eq!( ParatensorModule::get_rank( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_trust( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_consensus( netuid, uid ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
			assert_eq!( ParatensorModule::get_incentive( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_dividend( netuid, uid ), 327 ); // Note D = floor(1 / 200 * 65_535) = 327
			assert_eq!( ParatensorModule::get_emission( netuid, uid ), 2499995 ); // Note E = 0.5 / 200 * 1_000_000_000 = 2_500_000 (discrepancy)
			assert_eq!( bonds[uid as usize][0], 0.0 );
			assert_eq!( bonds[uid as usize][validators as usize], I32F32::from_num(327) / I32F32::from_num(65_535) ); // Note B_ij = floor(1 / 200 * 65_535) / 65_535 = 327 / 65_535
		}
		for uid in validators..n { // servers
			assert_eq!( ParatensorModule::get_stake_for_hotkey( &(uid as u64) ), 0 );
			assert_eq!( ParatensorModule::get_rank( netuid, uid ), 16 ); // Note R = floor(1 / (4096 - 200) * 65_535) = 16
			assert_eq!( ParatensorModule::get_trust( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_consensus( netuid, uid ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
			assert_eq!( ParatensorModule::get_incentive( netuid, uid ), 16 ); // Note I = floor(1 / (4096 - 200) * 65_535) = 16
			assert_eq!( ParatensorModule::get_dividend( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_emission( netuid, uid ), 128336 ); // Note E = floor(0.5 / (4096 - 200) * 1_000_000_000) = 128336
			assert_eq!( bonds[uid as usize][0], 0.0 );
			assert_eq!( bonds[uid as usize][validators as usize], 0.0 );
		}
	});
}

#[test]
/// Test an epoch_sparse on a graph with 16384 nodes, of which the first 512 are validators setting non-self weights, and the rest servers setting only self-weights.
fn test_16384_graph_sparse() {
	new_test_ext().execute_with(|| {
		let netuid: u16 = 0;
		let n: u16 = 16384;
		let validators: u16 = 512;
		let servers = n - validators;
		let epochs: u16 = 1;
		println!( "test_{n:?}_graph ({validators:?} validators)" );
		init_run_epochs(netuid, n, validators, servers, epochs, true);
		let bonds = ParatensorModule::get_bonds( netuid );
		for uid in 0..validators { // validators
			assert_eq!( ParatensorModule::get_stake_for_hotkey( &(uid as u64) ), 1 );
			assert_eq!( ParatensorModule::get_rank( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_trust( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_consensus( netuid, uid ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
			assert_eq!( ParatensorModule::get_incentive( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_dividend( netuid, uid ), 127 ); // Note D = floor(1 / 512 * 65_535) = 127
			assert_eq!( ParatensorModule::get_emission( netuid, uid ), 976085 ); // Note E = 0.5 / 512 * 1_000_000_000 = 976_562 (discrepancy)
			assert_eq!( bonds[uid as usize][0], 0.0 );
			assert_eq!( bonds[uid as usize][validators as usize], I32F32::from_num(127) / I32F32::from_num(65_535) ); // Note B_ij = floor(1 / 512 * 65_535) / 65_535 = 127 / 65_535
		}
		for uid in validators..n { // servers
			assert_eq!( ParatensorModule::get_stake_for_hotkey( &(uid as u64) ), 0 );
			assert_eq!( ParatensorModule::get_rank( netuid, uid ), 4 ); // Note R = floor(1 / (16384 - 512) * 65_535) = 4
			assert_eq!( ParatensorModule::get_trust( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_consensus( netuid, uid ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
			assert_eq!( ParatensorModule::get_incentive( netuid, uid ), 4 ); // Note I = floor(1 / (16384 - 512) * 65_535) = 4
			assert_eq!( ParatensorModule::get_dividend( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_emission( netuid, uid ), 31517 ); // Note E = floor(0.5 / (16384 - 512) * 1_000_000_000) = 31502 (discrepancy)
			assert_eq!( bonds[uid as usize][0], 0.0 );
			assert_eq!( bonds[uid as usize][validators as usize], 0.0 );
		}
	});
}

fn init_run_epochs(netuid: u16, n: u16, validators: u16, servers: u16, epochs: u16, sparse: bool) {
	ParatensorModule::set_max_allowed_uids( netuid, n );
	for key in 0..n {
		let stake: u128 = if key < validators { 1 } else { 0 }; // only validators receive stake
		// let stake: u128 = 1; // alternative test: all nodes receive stake, should be same outcome, except stake
		ParatensorModule::add_balance_to_coldkey_account( &(key as u64), stake );
		ParatensorModule::set_stake_for_testing( &(key as u64), stake as u64 );
		ParatensorModule::add_subnetwork_account( netuid, key, &(key as u64) );
	}
	assert_eq!( ParatensorModule::get_subnetwork_n(netuid), n );
	run_to_block( 1 ); // run to next block to ensure weights are set on nodes after their registration block
	for uid in 0..validators { // validators
		assert_ok!(ParatensorModule::set_weights(Origin::signed(uid as u64), netuid, (validators..n).collect(), vec![ u16::MAX / n; servers as usize ]));
	}
	for uid in validators..n { // servers
		assert_ok!(ParatensorModule::set_weights(Origin::signed(uid as u64), netuid, vec![ uid as u16 ], vec![ u16::MAX ])); // server self-weight
	}
	// Run the epochs.
	println!("Start {epochs} epoch(s)");
	let start = Instant::now();
	for _ in 0..epochs {
		if sparse {
			ParatensorModule::epoch_sparse( netuid, 1_000_000_000, false );
		}
		else {
			ParatensorModule::epoch( netuid, 1_000_000_000, false );
		}
	}
	let duration = start.elapsed();
	println!("Time elapsed in epoch() is: {:?}", duration);
	let bonds = ParatensorModule::get_bonds( netuid );
	for (uid, node) in vec![ (0, "validator"), (validators, "server") ] {
		println!( "\n{node}" );
		uid_stats(netuid, uid);
		println!( "bonds: {:?} (on validator), {:?} (on server)", bonds[uid as usize][0], bonds[uid as usize][validators as usize]);
	}
}

#[test]
/// Test that epoch masks out inactive stake of validators with outdated weights beyond activity cutoff.
fn test_active_stake() {
	new_test_ext().execute_with(|| {
		let sparse: bool = true;
		let debug: bool = false;
		let n: u16 = 4;
		let netuid: u16 = 0;
		let tempo: u16 = u16::MAX - 1;  // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
		let block_number: u64 = 0;
		add_network(netuid, tempo, 0);
		ParatensorModule::set_max_allowed_uids( netuid, n );
		ParatensorModule::set_max_registratations_per_block( n );
		// === Register [validator1, validator2, server1, server2]
		for key in 0..n as u64 {
			let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, key * 1_000_000);
			assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(key), netuid, block_number, nonce, work, key, key));
			ParatensorModule::add_balance_to_coldkey_account( &key, 1 );
			ParatensorModule::set_stake_for_testing( &key, 1 );
		}
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid), n);
		run_to_block( 1 ); // run to next block to ensure weights are set on nodes after their registration block
		// === Set weights [val1->srv1: 0.5, val1->srv2: 0.5, val2->srv1: 0.5, val2->srv2: 0.5]
		for uid in 0..(n/2) as u64 {
			assert_ok!(ParatensorModule::set_weights(Origin::signed(uid), netuid, ((n/2)..n).collect(), vec![ u16::MAX / (n/2); (n/2) as usize ]));
		}
		if sparse { ParatensorModule::epoch_sparse( netuid, 1_000_000_000, debug ); }
		else { ParatensorModule::epoch( netuid, 1_000_000_000, debug ); }
		let bonds = ParatensorModule::get_bonds( netuid );
		for uid in 0..n as u16 {
			// println!( "\n{uid}" );
			// uid_stats(netuid, uid);
			// println!( "bonds: {:?}", bonds[uid as usize]);
			if uid < n/2 {
				assert_eq!( ParatensorModule::get_dividend( netuid, uid ), 32767 ); // Note D = floor(0.5 * 65_535)
			}
			assert_eq!( ParatensorModule::get_emission( netuid, uid ), 250000000 ); // Note E = 0.5 / (n/2) * 1_000_000_000 = 250_000_000
		}
		for validator in 0..(n/2) as usize {
			for on_validator in 0..(n/2) as usize {
				assert_eq!( bonds[validator][on_validator], 0 );
			}
			for server in ((n/2) as usize)..n as usize {
				assert_eq!( bonds[validator][server], I32F32::from_num(32767) / I32F32::from_num(65_535) ); // floor(0.5*(2^16-1))/(2^16-1)
			}
		}
        let activity_cutoff: u64 = ParatensorModule::get_activity_cutoff( netuid ) as u64;
		run_to_block( activity_cutoff + 2 ); // run to block where validator (uid 0, 1) weights become outdated
		// === Update uid 0 weights
		assert_ok!(ParatensorModule::set_weights(Origin::signed(0), netuid, ((n/2)..n).collect(), vec![ u16::MAX / (n/2); (n/2) as usize ]));
		if sparse { ParatensorModule::epoch_sparse( netuid, 1_000_000_000, debug ); }
		else { ParatensorModule::epoch( netuid, 1_000_000_000, debug ); }
		/*	current_block: 5002; activity_cutoff: 5000
			Last update: [5002, 1, 0, 0]; Inactive: [false, true, true, true]
			S: [1, 1, 1, 1]; S (mask): [1, 0, 0, 0]; S (mask+norm): [1, 0, 0, 0]
			Block at registration: [0, 0, 0, 0]
			W: [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			W (diagmask): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			W (diag+outdatemask): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			W (mask+norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
			R: [0, 0, 0.5, 0.5]
			W (threshold): [[(2, 1), (3, 1)], [(2, 1), (3, 1)], [], []]
			T: [0, 0, 1, 1]
			C: [0.006693358, 0.006693358, 0.9933076561, 0.9933076561]
			I: [0, 0, 0.5, 0.5]
			B: [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			B (outdatedmask): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			B (mask+norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
			ΔB: [[(2, 0.5), (3, 0.5)], [(2, 0), (3, 0)], [], []]
			ΔB (norm): [[(2, 1), (3, 1)], [(2, 0), (3, 0)], [], []]
			emaB: [[(2, 0.55), (3, 0.55)], [(2, 0.45), (3, 0.45)], [], []]
			D: [0.5499999998, 0.4499999997, 0, 0]
			E: [274999999.9068677425, 224999999.8603016138, 250000000, 250000000]
			P: [0.275, 0.2249999999, 0.25, 0.25] */
		let bonds = ParatensorModule::get_bonds( netuid );
		assert_eq!( ParatensorModule::get_dividend( netuid, 0 ), 36044 ); // Note D = floor((0.5 * 0.9 + 0.1) * 65_535)
		assert_eq!( ParatensorModule::get_emission( netuid, 0 ), 274999999 ); // Note E = 0.5 * 0.55 * 1_000_000_000 = 275_000_000 (discrepancy)
		for server in ((n/2) as usize)..n as usize {
			assert_eq!( bonds[0][server], I32F32::from_num(36044) / I32F32::from_num(65_535) ); // floor(0.55*(2^16-1))/(2^16-1)
		}
		for validator in 1..(n/2) as u16 {
			assert_eq!( ParatensorModule::get_dividend( netuid, validator ), 29490 ); // Note D = floor((0.5 * 0.9) * 65_535)
			assert_eq!( ParatensorModule::get_emission( netuid, validator  ), 224999999 ); // Note E = 0.5 * 0.45 * 1_000_000_000 = 225_000_000 (discrepancy)
			for server in ((n/2) as usize)..n as usize {
				assert_eq!( bonds[validator as usize][server], I32F32::from_num(29490) / I32F32::from_num(65_535) ); // floor(0.45*(2^16-1))/(2^16-1)
			}
		}
		// === Update uid 1 weights as well
		assert_ok!(ParatensorModule::set_weights(Origin::signed(1), netuid, ((n/2)..n).collect(), vec![ u16::MAX / (n/2); (n/2) as usize ]));
		run_to_block( activity_cutoff + 3 ); // run to block where validator (uid 0, 1) weights become outdated
		if sparse { ParatensorModule::epoch_sparse( netuid, 1_000_000_000, debug ); }
		else { ParatensorModule::epoch( netuid, 1_000_000_000, debug ); }
		/*	current_block: 5003; activity_cutoff: 5000
			Last update: [5002, 5002, 0, 0]; Inactive: [false, false, true, true]
			S: [1, 1, 1, 1]; S (mask): [1, 1, 0, 0]; S (mask+norm): [0.5, 0.5, 0, 0]
			Block at registration: [0, 0, 0, 0]
			W: [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			W (diagmask): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			W (diag+outdatemask): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			W (mask+norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
			R: [0, 0, 0.5, 0.5]
			W (threshold): [[(2, 1), (3, 1)], [(2, 1), (3, 1)], [], []]
			T: [0, 0, 1, 1]
			C: [0.006693358, 0.006693358, 0.9933076561, 0.9933076561]
			I: [0, 0, 0.5, 0.5]
			B: [[(2, 0.5499961851), (3, 0.5499961851)], [(2, 0.4499885556), (3, 0.4499885556)], [], []]
			B (outdatedmask): [[(2, 0.5499961851), (3, 0.5499961851)], [(2, 0.4499885556), (3, 0.4499885556)], [], []]
			B (mask+norm): [[(2, 0.5500045777), (3, 0.5500045777)], [(2, 0.449995422), (3, 0.449995422)], [], []]
			ΔB: [[(2, 0.25), (3, 0.25)], [(2, 0.25), (3, 0.25)], [], []]
			ΔB (norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
			emaB: [[(2, 0.54500412), (3, 0.54500412)], [(2, 0.4549958797), (3, 0.4549958797)], [], []]
			D: [0.54500412, 0.4549958794, 0, 0]
			E: [272502060.0482821465, 227497939.71888721, 250000000, 250000000]
			P: [0.27250206, 0.2274979397, 0.25, 0.25] */
		let bonds = ParatensorModule::get_bonds( netuid );
		assert_eq!( ParatensorModule::get_dividend( netuid, 0 ), 35716 ); // Note D = floor((0.55 * 0.9 + 0.5 * 0.1) * 65_535)
		assert_eq!( ParatensorModule::get_emission( netuid, 0 ), 272502060 ); // Note E = 0.5 * (0.55 * 0.9 + 0.5 * 0.1) * 1_000_000_000 = 272_500_000 (discrepancy)
		for server in ((n/2) as usize)..n as usize {
			assert_eq!( bonds[0][server], I32F32::from_num(35716) / I32F32::from_num(65_535) ); // floor((0.55 * 0.9 + 0.5 * 0.1)*(2^16-1))/(2^16-1)
		}
		assert_eq!( ParatensorModule::get_dividend( netuid, 1 ), 29818 ); // Note D = floor((0.45 * 0.9 + 0.5 * 0.1) * 65_535)
		assert_eq!( ParatensorModule::get_emission( netuid, 1 ), 227497939 ); // Note E = 0.5 * (0.45 * 0.9 + 0.5 * 0.1) * 1_000_000_000 = 227_500_000 (discrepancy)
		for server in ((n/2) as usize)..n as usize {
			assert_eq!( bonds[1][server], I32F32::from_num(29818) / I32F32::from_num(65_535) ); // floor((0.45 * 0.9 + 0.5 * 0.1)*(2^16-1))/(2^16-1)
		}
	});
}

#[test]
/// Test that epoch masks out outdated weights and bonds of validators on deregistered servers.
fn test_outdated_weights() {
	new_test_ext().execute_with(|| {
		let sparse: bool = true;
		let debug: bool = false;
		let n: u16 = 4;
		let netuid: u16 = 0;
		let tempo: u16 = u16::MAX - 1;  // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
		let mut block_number: u64 = 0;
		add_network(netuid, tempo, 0);
		ParatensorModule::set_max_allowed_uids( netuid, n );
		ParatensorModule::set_max_registratations_per_block( n+1 ); // should be n, but RegistrationsThisBlock is not reset (TODO: Saeideh)
		// === Register [validator1, validator2, server1, server2]
		for key in 0..n as u64 {
			let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, key * 1_000_000);
			assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(key), netuid, block_number, nonce, work, key, key));
			ParatensorModule::add_balance_to_coldkey_account( &key, 1 );
			ParatensorModule::set_stake_for_testing( &key, 1 );
		}
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid), n);
		run_to_block( 1 ); block_number += 1; // run to next block to ensure weights are set on nodes after their registration block
		// === Set weights [val1->srv1: 2/3, val1->srv2: 1/3, val2->srv1: 2/3, val2->srv2: 1/3, srv1->srv1: 1, srv2->srv2: 1]
		for uid in 0..(n/2) as u64 {
			assert_ok!(ParatensorModule::set_weights(Origin::signed(uid), netuid, ((n/2)..n).collect(), vec![ 2 * (u16::MAX / 3), u16::MAX / 3 ]));
		}
		for uid in ((n/2) as u64)..n as u64 {
			assert_ok!(ParatensorModule::set_weights(Origin::signed(uid), netuid, vec![ uid as u16 ], vec![ u16::MAX ])); // server self-weight
		}
		if sparse { ParatensorModule::epoch_sparse( netuid, 1_000_000_000, debug ); }
		else { ParatensorModule::epoch( netuid, 1_000_000_000, debug ); }
		/*	current_block: 1; activity_cutoff: 5000
			Last update: [1, 1, 1, 1]; Inactive: [false, false, false, false]
			S: [1, 1, 1, 1]; S (mask): [1, 1, 1, 1]; S (mask+norm): [0.25, 0.25, 0.25, 0.25]
			Block at registration: [0, 0, 0, 0]
			W: [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665), (3, 0.3333333333)], [(2, 1)], [(3, 1)]]
			W (diagmask): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665), (3, 0.3333333333)], [], []]
			W (diag+outdatemask): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665), (3, 0.3333333333)], [], []]
			W (mask+norm): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665), (3, 0.3333333333)], [], []]
			R: [0, 0, 0.6666666665, 0.3333333333]
			W (threshold): [[(2, 1), (3, 1)], [(2, 1), (3, 1)], [], []]
			T: [0, 0, 0.5, 0.5]
			C: [0.006693358, 0.006693358, 0.500019074, 0.500019074]
			I: [0, 0, 0.6666666667, 0.333333333]
			B: [[], [], [], []]
			B (outdatedmask): [[], [], [], []]
			B (mask+norm): [[], [], [], []]
			ΔB: [[(2, 0.1666666665), (3, 0.0833333333)], [(2, 0.1666666665), (3, 0.0833333333)], [], []]
			ΔB (norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
			emaB: [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
			D: [0.4999999998, 0.4999999998, 0, 0]
			E: [249999999.7671693563, 249999999.7671693563, 333333333.4885537624, 166666666.5114462376]
			P: [0.2499999998, 0.2499999998, 0.3333333335, 0.1666666665] */
		// === Dereg server2 at uid3 (least emission) + register new key over uid3
		let new_key: u64 = n as u64; // register a new key while at max capacity, which means the least incentive uid will be deregistered
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 0);
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(new_key), netuid, block_number, nonce, work, new_key, new_key));
		let deregistered_uid: u16 = n-1; // since uid=n-1 only recieved 1/3 of weight, it will get pruned first
		assert_eq!(new_key, ParatensorModule::get_hotkey_for_net_and_neuron(netuid, deregistered_uid));
		run_to_block( 2 ); // run to next block to outdate weights and bonds set on deregistered uid
		// === Update weights from only uid=0
		assert_ok!(ParatensorModule::set_weights(Origin::signed(0), netuid, ((n/2)..n).collect(), vec![ 2 * (u16::MAX / 3), u16::MAX / 3 ]));
		if sparse { ParatensorModule::epoch_sparse( netuid, 1_000_000_000, debug ); }
		else { ParatensorModule::epoch( netuid, 1_000_000_000, debug ); }
		/* 	current_block: 2; activity_cutoff: 5000
			Last update: [2, 1, 1, 1]; Inactive: [false, false, false, false]
			Block at registration: [0, 0, 0, 1]
			S: [1, 1, 1, 0]; S (mask): [1, 1, 1, 0]
			S (mask+norm): [0.3333333333, 0.3333333333, 0.3333333333, 0]
			W: [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665), (3, 0.3333333333)], [(2, 1)], []]
			W (diagmask): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665), (3, 0.3333333333)], [], []]
			W (diag+outdatemask): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665)], [], []]
			W (mask+norm): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 1)], [], []]
			R: [0, 0, 0.8333333333, 0.1666666665]
			W (threshold): [[(2, 1), (3, 1)], [(2, 1)], [], []]
			T: [0, 0, 0.6666666665, 0.3333333333]
			C: [0.006693358, 0.006693358, 0.84114109, 0.1588793003]
			I: [0, 0, 0.9635980718, 0.036401928]
			B: [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			B (outdatedmask): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704)], [], []]
			B (mask+norm): [[(2, 0.5), (3, 1)], [(2, 0.5)], [], []]
			ΔB: [[(2, 0.222222222), (3, 0.111111111)], [(2, 0.3333333333)], [], []]
			ΔB (norm): [[(2, 0.3999999997), (3, 1)], [(2, 0.6)], [], []]
			emaB: [[(2, 0.4899999998), (3, 1)], [(2, 0.51)], [], []]
			D: [0.5085649828, 0.4914350165, 0, 0]
			E: [254282491.5144592524, 245717508.252710104, 481799036.031588912, 18200963.968411088]
			P: [0.2542824915, 0.2457175083, 0.481799036, 0.018200964] */
		let bonds = ParatensorModule::get_bonds( netuid );
		assert_eq!( ParatensorModule::get_dividend( netuid, 0 ), 33328 ); // Note D = floor(0.5085649828 * 65_535)
		assert_eq!( ParatensorModule::get_emission( netuid, 0 ), 254282491 ); // Note E = 0.5 * 0.5085649828 * 1_000_000_000 = 272_500_000 (discrepancy)
		assert_eq!( bonds[0][2], I32F32::from_num(32112) / I32F32::from_num(65_535) ); // floor(0.49*(2^16-1))/(2^16-1)
		assert_eq!( bonds[0][3], I32F32::from_num(1) ); // only uid0 has updated weights for new reg
	});
}
