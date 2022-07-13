use frame_support::assert_ok;

#[cfg(feature = "no_std")]
use ndarray::{ndarray::Array1, ndarray::Array2, ndarray::arr1};


#[allow(dead_code)]
pub fn assert_f32_array_eq( a_array: &ndarray::Array1<f32>, b_array: &ndarray::Array1<f32>, eps: f32 ) {
    for (a, b) in a_array.iter().zip( b_array.iter() ) { 
        if a > b { assert!( a - b <= eps ); }
        if b > a { assert!( b - a <= eps ); }
    }
    return assert!( true );
}
#[allow(dead_code)]
pub fn assert_u64_array_eq( a_array: &ndarray::Array1<u64>, b_array: &ndarray::Array1<u64>, eps: u64 ) {
    for (a, b) in a_array.iter().zip( b_array.iter() ) { 
        if a > b { assert!( a - b <= eps ); }
        if b > a { assert!( b - a <= eps ); }
    }
    return assert!( true );
}
#[allow(dead_code)]
pub fn assert_u16_approx_equals( a:u16, b: u16 ) {
    let eps:u16 = 100;
    if a > b { assert!( a - b <= eps ); }
    if b > a { assert!( b - a <= eps ); }
}
#[allow(dead_code)]
pub fn assert_u64_approx_equals( a:u64, b: u64 ) {
    let eps:u64 = 100;
    if a > b { assert!( a - b <= eps ); }
    if b > a { assert!( b - a <= eps ); }
}
#[allow(dead_code)]
pub fn assert_u16_vec_eq( a_vec: &Vec<u16>, b_vec: &Vec<u16> ) {
    for (a, b) in a_vec.iter().zip(b_vec.iter()) { assert_u16_approx_equals( *a, *b ); }
    return assert!( true );
}
#[allow(dead_code)]
pub fn assert_u64_vec_eq( a_vec: &Vec<u64>, b_vec: &Vec<u64> ) {
    for (a, b) in a_vec.iter().zip(b_vec.iter()) { assert_u64_approx_equals( *a, *b ); }
    return assert!( true );
}
#[allow(dead_code)]
pub fn mat_approx_equals( a_vec: &Vec<Vec<u16>>, b_vec: &Vec<Vec<u16>> ) {
    for (a, b) in a_vec.iter().zip(b_vec.iter()) { assert_u16_vec_eq( a, b ); }
    return assert!( true );
}