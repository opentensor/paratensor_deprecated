use crate::{mock::*};
use pallet_szabo::{Error};
use frame_system::{Config};
use frame_support::assert_ok;
use rand::Rng;
use frame_support::inherent::Vec;

#[cfg(feature = "no_std")]
extern crate nalgebra;

use nalgebra::DMatrix;

pub fn assert_u16_approx_equals( a:u16, b: u16 ) {
    let eps:u16 = 100;
    if a > b { assert!( a - b <= eps ); }
    if b > a { assert!( b - a <= eps ); }
}

pub fn assert_u64_approx_equals( a:u64, b: u64 ) {
    let eps:u64 = 100;
    if a > b { assert!( a - b <= eps ); }
    if b > a { assert!( b - a <= eps ); }
}

pub fn assert_u16_vec_eq( a_vec: &Vec<u16>, b_vec: &Vec<u16> ) {
    for (a, b) in a_vec.iter().zip(b_vec.iter()) { assert_u16_approx_equals( *a, *b ); }
    return assert!( true );
}

pub fn assert_u64_vec_eq( a_vec: &Vec<u64>, b_vec: &Vec<u64> ) {
    for (a, b) in a_vec.iter().zip(b_vec.iter()) { assert_u64_approx_equals( *a, *b ); }
    return assert!( true );
}

pub fn mat_approx_equals( a_vec: &Vec<Vec<u16>>, b_vec: &Vec<Vec<u16>> ) {
    for (a, b) in a_vec.iter().zip(b_vec.iter()) { assert_u16_vec_eq( a, b ); }
    return assert!( true );
}

pub fn print_network_state( netuid: u16 ) {
    println!( "S: {}", SzaboModule::get_stake_as_vector( netuid ) );
    println!( "Sn: {}", SzaboModule::get_stake_as_float_vector( netuid ) );
    println!( "W: {}", SzaboModule::get_weights_as_float_matrix( netuid ) );
    println!( "B: {}", SzaboModule::get_bonds_as_float_matrix( netuid ) );
    println!( "R: {}", SzaboModule::get_rank_as_float_vector( netuid ) );
    println!( "T: {}", SzaboModule::get_trust_as_float_vector( netuid ) );
    println!( "C: {}", SzaboModule::get_consensus_as_float_vector( netuid ) );
    println!( "I: {}", SzaboModule::get_incentive_as_float_vector( netuid ) );
    println!( "D: {}", SzaboModule::get_dividends_as_float_vector( netuid ) );
    println!( "E: {}", SzaboModule::get_emission_as_vector( netuid ) );
}

pub fn create_random_subgraph( netuid: u16, n: u16 ) {
    for i in 0..n {
        SzaboModule::add_global_account( &(i as u64), &(i as u64) );
        let random_stake = rand::thread_rng().gen_range(0..100);
        SzaboModule::add_balance_to_coldkey_account( &(i as u64), random_stake );
		assert_ok!(SzaboModule::add_stake(Origin::signed(i as u64), i as u64, random_stake));
        SzaboModule::add_subnetwork_account( netuid, i, &(i as u64) );
    }
    let mut weights_vec: Vec<f32> = vec![0.0; (n * n) as usize];
    for i in 0..n*n {
        let rw: u16 = rand::thread_rng().gen_range(0..100);
        weights_vec[i as usize] = rw as f32 / 100.0;
    }
    let mut W: nalgebra::DMatrix<f32> = nalgebra::DMatrix::from_vec(n as usize, n as usize, weights_vec );

    //let W2 = 
    //println!( "W1: {}", W );
    //W.row_mut(0).map(|x: f32| x * 2.0);
    //println!( "W1: {}", W );
    let W1 = W * nalgebra::DVector::from_vec( vec![1.0/2.0; W.ncols() as usize] );
    let W2: nalgebra::DMatrix<f32> = W1.into();
    //let s0 = W.row(0).sum();
    //W.row_mut(0).map( |i: f32| -> f32 { i / s0 } );
    //println!( "W2: {}", W2 );
    SzaboModule::set_weights_from_float_matrix( netuid, W );
}
