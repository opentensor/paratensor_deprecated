use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
	generic::BlockId,
	traits::Block as BlockT,
};
use std::sync::Arc;

pub use paratensor_custom_rpc_runtime_api::DelegateInfoRuntimeApi;
use pallet_paratensor::delegate_info::DelegateInfo as DelegateInfoStruct;

pub use paratensor_custom_rpc_runtime_api::NeuronInfoRuntimeApi;
use pallet_paratensor::neuron_info::NeuronInfo as NeuronInfoStruct;

pub use paratensor_custom_rpc_runtime_api::SubnetInfoRuntimeApi;
use pallet_paratensor::subnet_info::SubnetInfo as SubnetInfoStruct;

#[rpc]
pub trait ParatensorCustomApi<BlockHash> {
	#[rpc(name = "delegateInfo_getDelegates")]
	fn get_delegates(&self, at: Option<BlockHash>) -> Result<Vec<DelegateInfoStruct>>;
	#[rpc(name = "delegateInfo_getDelegate")]
	fn get_delegate(&self, delegate_account_vec: Vec<u8>, at: Option<BlockHash>) -> Result<Option<DelegateInfoStruct>>;

	#[rpc(name = "neuronInfo_getNeurons")]
	fn get_neurons(&self, netuid: u16, at: Option<BlockHash>) -> Result<Vec<NeuronInfoStruct>>;
	#[rpc(name = "neuronInfo_getNeuron")]
	fn get_neuron(&self, netuid: u16, uid: u16, at: Option<BlockHash>) -> Result<Option<NeuronInfoStruct>>;

	#[rpc(name = "subnetInfo_getSubnetInfo")]
	fn get_subnet_info(&self, netuid: u16, at: Option<BlockHash>) -> Result<Option<SubnetInfoStruct>>;
	#[rpc(name = "subnetInfo_getSubnetsInfo")]
	fn get_subnets_info(&self, at: Option<BlockHash>) -> Result<Vec<Option<SubnetInfoStruct>>>;
}

pub struct ParatensorCustom<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> ParatensorCustom<C, M> {
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

/// Error type of this RPC api.
pub enum Error {
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
		}
	}
}

impl<C, Block> ParatensorCustomApi<<Block as BlockT>::Hash> for ParatensorCustom<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: DelegateInfoRuntimeApi<Block>,
	C::Api: NeuronInfoRuntimeApi<Block>,
	C::Api: SubnetInfoRuntimeApi<Block>,
	{ 
	fn get_delegates(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<DelegateInfoStruct>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.get_delegates(&at).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get delegates info.".into(),
			data: Some(e.to_string().into()),
		})
	}

	fn get_delegate(&self, delegate_account_vec: Vec<u8>, at: Option<<Block as BlockT>::Hash>) -> Result<Option<DelegateInfoStruct>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.get_delegate(&at, delegate_account_vec).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get delegate info.".into(),
			data: Some(e.to_string().into()),
		})
	}

	fn get_neurons(&self, netuid: u16, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<NeuronInfoStruct>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.get_neurons(&at, netuid).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get neurons info.".into(),
			data: Some(e.to_string().into()),
		})
	}

	fn get_neuron(&self, netuid: u16, uid: u16, at: Option<<Block as BlockT>::Hash>) -> Result<Option<NeuronInfoStruct>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.get_neuron(&at, netuid, uid).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get neuron info.".into(),
			data: Some(e.to_string().into()),
		})
	}
	
	fn get_subnet_info(&self, netuid: u16, at: Option<<Block as BlockT>::Hash>) -> Result<Option<SubnetInfoStruct>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.get_subnet_info(&at, netuid).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get subnet info.".into(),
			data: Some(e.to_string().into()),
		})
	}

	fn get_subnets_info(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<Option<SubnetInfoStruct>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.get_subnets_info(&at).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get subnets info.".into(),
			data: Some(e.to_string().into()),
		})
	}
}