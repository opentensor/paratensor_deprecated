use super::*;
use frame_support::inherent::Vec;
use frame_support::sp_std::vec;


impl<T: Config> Pallet<T> {

    /// ---- The implementation for the extrinsic serve_axon which sets the ip endpoint information for a uid on a network.
    ///
    /// # Args:
    /// 	* 'origin': (<T as frame_system::Config>Origin):
    /// 		- The signature of the caller.
    ///
    /// 	* 'netuid' (u16):
    /// 		- The u16 network identifier.
    ///
    /// 	* 'version' (u64):
    /// 		- The bittensor version identifier.
    ///
    /// 	* 'ip' (u64):
    /// 		- The endpoint ip information as a u128 encoded integer.
    ///
    /// 	* 'port' (u16):
    /// 		- The endpoint port information as a u16 encoded integer.
    /// 
    /// 	* 'ip_type' (u8):
    /// 		- The endpoint ip version as a u8, 4 or 6.
    /// 
    /// 	* 'modality' (u8):
    /// 		- The endpoint modality. TODO: The modality should be known by the network.
    ///
    pub fn do_serve_axon( origin: T::Origin, netuid: u16, version: u32, ip: u128, port: u16, ip_type: u8, modality: u16 ) -> dispatch::DispatchResult {
        // --- 1. We check the callers (hotkey) signature.
        let hotkey_id = ensure_signed(origin)?;

        // --- 2. We check if the network exist.
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);

        // --- 3. We make validy checks on the passed data.
        ensure!( Self::is_hotkey_registered(netuid, &hotkey_id), Error::<T>::NotRegistered );        
        ensure!( Self::if_modality_is_valid(modality), Error::<T>::InvalidModality );
        ensure!( Self::is_valid_ip_type(ip_type), Error::<T>::InvalidIpType );
        ensure!( Self::is_valid_ip_address(ip_type, ip), Error::<T>::InvalidIpAddress );
  
        // --- 4. We get the uid associated with this hotkey account.
        let neuron_uid = Self::get_neuron_for_net_and_hotkey(netuid, &hotkey_id);

        // --- 5. We get the neuron assoicated with this hotkey.
        let mut neuron = Self::get_neuron_metadata(neuron_uid);

        // --- 6. We insert the neuron metadata.
        neuron.version = version;
        neuron.ip = ip;
        neuron.port = port;
        neuron.ip_type = ip_type;
        NeuronsMetaData::<T>::insert(neuron_uid, neuron);

        // --- 7. We set the last update for this neuron.
        Self::set_last_update_for_neuron(netuid, neuron_uid, Self::get_current_block_as_u64());

        // --- 8. We deposit the neuron updated event.
        Self::deposit_event(Event::AxonServed(neuron_uid));
        
        // --- 9. Return is successful dispatch. 
        Ok(())
    }

    /********************************
     --==[[  Helper functions   ]]==--
    *********************************/

    /*fn is_valid_modality(modality: u8) -> bool {
        let allowed_values: Vec<u8> = vec![0];
        return allowed_values.contains(&modality);
    } */

    pub fn is_valid_ip_type(ip_type: u8) -> bool {
        let allowed_values: Vec<u8> = vec![4, 6];
        return allowed_values.contains(&ip_type);
    }

    // @todo (Parallax 2-1-2021) : Implement exclusion of private IP ranges
    pub fn is_valid_ip_address(ip_type: u8, addr: u128) -> bool {
        if !Self::is_valid_ip_type(ip_type) {
            return false;
        }

        if addr == 0 {
            return false;
        }

        if ip_type == 4 {
            if addr == 0 { return false; }
            if addr >= u32::MAX as u128 { return false; }
            if addr == 0x7f000001 { return false; } // Localhost
        }

        if ip_type == 6 {
            if addr == 0x0 { return false; }
            if addr == u128::MAX { return false; }
            if addr == 1 { return false; } // IPv6 localhost
        }

        return true;
    }


}