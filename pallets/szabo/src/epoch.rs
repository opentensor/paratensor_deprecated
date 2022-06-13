#[cfg(feature = "no_std")]
extern crate nalgebra as na;

impl<T: Config> Pallet<T> {
    pub fn epoch() {
        let v = na::Vector::from_vec( vec![1,2,3] ); 
    }
}

