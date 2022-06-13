use frame_support::inherent::Vec;
use frame_support::sp_std::vec;

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
