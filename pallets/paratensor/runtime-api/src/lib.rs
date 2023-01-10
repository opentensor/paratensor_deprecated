#![cfg_attr(not(feature = "std"), no_std)]
use pallet_paratensor::neuron_info::NeuronInfo as NeuronInfoStruct;
use pallet_paratensor::subnet_info::SubnetInfo as SubnetInfoStruct;
extern crate alloc;
use alloc::vec::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// src/neuron_info.rs and src/subnet_info.rs
sp_api::decl_runtime_apis! {
	pub trait NeuronInfoRuntimeApi {
		fn get_neurons(netuid: u16) -> Vec<NeuronInfoStruct>;
		fn get_neuron(netuid: u16, uid: u16) -> Option<NeuronInfoStruct>;
	}

	pub trait SubnetInfoRuntimeApi {
		fn get_subnet_info(netuid: u16) -> Option<SubnetInfoStruct>;
		fn get_subnets_info() -> Vec<Option<SubnetInfoStruct>>;
	}
}