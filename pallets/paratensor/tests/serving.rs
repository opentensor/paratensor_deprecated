use crate::{mock::*};
mod mock;
use frame_support::assert_ok;
use frame_support::dispatch::{GetDispatchInfo, DispatchInfo};
use frame_support::weights::{DispatchClass, Pays};
use frame_system::Config;

mod test {
use std::net::{Ipv4Addr, Ipv6Addr};

    // Generates an ipv6 address based on 8 ipv6 words and returns it as u128
    pub fn ipv6(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> u128 {
        return Ipv6Addr::new(a, b, c, d, e, f, g, h).into();
    }

    // Generate an ipv4 address based on 4 bytes and returns the corresponding u128, so it can be fed
    // to the module::subscribe() function
    pub fn ipv4(a: u8, b: u8, c: u8, d: u8) -> u128 {
        let ipv4: Ipv4Addr = Ipv4Addr::new(a, b, c, d);
        let integer: u32 = ipv4.into();
        return u128::from(integer);
    }
}

#[test]
fn test_serving_subscribe_ok_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
		
        let netuid: u16 = 1;
		let version : u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 128;
        let ip_type: u8 = 4;
        let modality: u8 = 0;
        let call = Call::ParatensorModule(ParatensorCall::serve_axon { netuid, version, ip, port, ip_type, modality });
		assert_eq!(call.get_dispatch_info(), DispatchInfo {
			weight: 0,
			class: DispatchClass::Normal,
			pays_fee: Pays::No
		});
	});
}

#[test]
fn test_serving_ok() {
	new_test_ext().execute_with(|| {
        let hotkey_account_id = 1;
        let netuid: u16 = 1;
		let version : u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 128;
        let ip_type: u8 = 4;
        let modality: u8 = 0;
        //
        add_network(netuid, modality);
        register_ok_neuron( netuid, hotkey_account_id, 66, 0);
        //
        assert_ok!(ParatensorModule::serve_axon(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, version, ip, port, ip_type, modality));
	});
}

#[test]
fn test_serving_is_valid_ip_type_ok_ipv4() {
	new_test_ext().execute_with(|| {
        assert_eq!(ParatensorModule::is_valid_ip_type(4), true);
	});
}

#[test]
fn test_serving_is_valid_ip_type_ok_ipv6() {
	new_test_ext().execute_with(|| {
        assert_eq!(ParatensorModule::is_valid_ip_type(6), true);
	});
}

#[test]
fn test_serving_is_valid_ip_type_nok() {
	new_test_ext().execute_with(|| {
        assert_eq!(ParatensorModule::is_valid_ip_type(10), false);
	});
}

#[test]
fn test_serving_is_valid_ip_address_ipv4() {
	new_test_ext().execute_with(|| {
        assert_eq!(ParatensorModule::is_valid_ip_address(4, test::ipv4(8, 8, 8, 8)), true);
	});
}

#[test]
fn test_serving_is_valid_ip_address_ipv6() {
	new_test_ext().execute_with(|| {
        assert_eq!(ParatensorModule::is_valid_ip_address(6, test::ipv6(1, 2, 3, 4, 5, 6, 7, 8)), true);
        assert_eq!(ParatensorModule::is_valid_ip_address(6, test::ipv6(1, 2, 3, 4, 5, 6, 7, 8)), true);
	});
}

#[test]
fn test_serving_is_invalid_ipv4_address() {
	new_test_ext().execute_with(|| {
        assert_eq!(ParatensorModule::is_valid_ip_address(4, test::ipv4(0, 0, 0, 0)), false);
        assert_eq!(ParatensorModule::is_valid_ip_address(4, test::ipv4(255, 255, 255, 255)), false);
        assert_eq!(ParatensorModule::is_valid_ip_address(4, test::ipv4(127, 0, 0, 1)), false);
        assert_eq!(ParatensorModule::is_valid_ip_address(4, test::ipv6(0xffff, 2, 3, 4, 5, 6, 7, 8)), false);
	});
}

#[test]
fn test_serving_is_invalid_ipv6_address() {
	new_test_ext().execute_with(|| {
        assert_eq!(ParatensorModule::is_valid_ip_address(6, test::ipv6(0, 0, 0, 0, 0, 0, 0, 0)), false);
        assert_eq!(ParatensorModule::is_valid_ip_address(4, test::ipv6(0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff)), false);
	});
}

#[test]
fn test_serving_set_metadata() {
	new_test_ext().execute_with(|| {
        let hotkey_account_id = 1;
        let netuid: u16 = 1;
		let version : u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 128;
        let ip_type: u8 = 4;
        let modality: u8 = 0;
        //
        add_network(netuid, modality);
        register_ok_neuron( netuid, hotkey_account_id, 66, 0);
        //
        assert_ok!(ParatensorModule::serve_axon(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, version, ip, port, ip_type, modality));

        let neuron_id = ParatensorModule::get_neuron_for_net_and_hotkey(netuid, &hotkey_account_id);
		let neuron = ParatensorModule::get_neuron_metadata(neuron_id);
		assert_eq!(neuron.ip, 1676056785);
		assert_eq!(neuron.version, 2);
		assert_eq!(neuron.port, 128);
        assert_eq!(neuron.ip_type, 4);
	});
}

