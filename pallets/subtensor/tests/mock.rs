use frame_support::{assert_ok, parameter_types, traits::{StorageMapShim, Contains, Hooks}, weights::{GetDispatchInfo,DispatchInfo}};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{
		self,
		BlakeTwo256, 
		IdentityLookup,
		Checkable,
		Applyable,
		Dispatchable,
		SignedExtension,
		ValidateUnsigned,
		PostDispatchInfoOf,
		DispatchInfoOf
	},
	ApplyExtrinsicResultWithInfo,
	transaction_validity::{TransactionValidity, TransactionSource, TransactionValidityError},
	codec::{Codec, Encode, Decode}
};

use pallet_subtensor::{NeuronMetadata};
use std::net::{Ipv6Addr, Ipv4Addr};
use std::{fmt::{self, Debug}};
use serde::{Serialize, Serializer};

use frame_system::{limits};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
		Subtensor: pallet_subtensor::{Pallet, Call, Storage, Event<T>},
	}
);

#[allow(dead_code)]
pub type SubtensorCall = pallet_subtensor::Call<Test>;

#[allow(dead_code)]
pub type BalanceCall = pallet_balances::Call<Test>;

thread_local!{
	pub static RUNTIME_VERSION: std::cell::RefCell<sp_version::RuntimeVersion> =
		Default::default();
}

/// Balance of an account.
#[allow(dead_code)]
pub type Balance = u128;

/// An index to a block.
#[allow(dead_code)]
pub type BlockNumber = u64;

pub struct TestBaseCallFilter;
impl Contains<Call> for TestBaseCallFilter {
	fn contains(c: &Call) -> bool {
		match *c {
			// Transfer works. Use `transfer_keep_alive` for a call that doesn't pass the filter.
			Call::Balances(pallet_balances::Call::transfer { .. }) => true,
			// For benchmarking, this acts as a noop call
			Call::System(frame_system::Call::remark { .. }) => true,
			_ => false,
		}
	}
}
impl frame_system::Config for Test {
	type BaseCallFilter = TestBaseCallFilter;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

#[allow(dead_code)]
pub type AccountId = u64;

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ();
	type AccountStore = StorageMapShim<
		pallet_balances::Account<Test>,
		frame_system::Provider<Test>,
		AccountId,
		pallet_balances::AccountData<Balance>,
	>;
	type MaxLocks = ();
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 640;
	pub BlockWeights: limits::BlockWeights = limits::BlockWeights::simple_max(1024);
	pub const ExistentialDeposit: Balance = 1;
	pub const TransactionByteFee: Balance = 100;
	pub const SDebug:u64 = 1;
	pub const InitialRho: u64 = 10;
	pub const InitialKappa: u64 = 2;
	pub const SelfOwnership: u64 = 2;
	pub const InitialImmunityPeriod: u64 = 2;
	pub const InitialMaxAllowedUids: u64 = 100;
	pub const InitialBondsMovingAverage: u64 = 500_000;
	pub const InitialIncentivePruningDenominator: u64 = 1;
	pub const InitialStakePruningDenominator: u64 = 1;
	pub const InitialFoundationDistribution: u64 = 0;

	pub const InitialValidatorBatchSize: u64 = 10;
	pub const InitialValidatorSequenceLen: u64 = 10;
	pub const InitialValidatorEpochLen: u64 = 10;
	pub const InitialValidatorEpochsPerReset: u64 = 10;

	pub const InitialMinAllowedWeights: u64 = 0;
	pub const InitialMaxAllowedMaxMinRatio: u64 = 0;
	pub const InitialBlocksPerStep: u64 = 1;
	pub const InitialIssuance: u64 = 548833985028256;
	pub const InitialDifficulty: u64 = 10000;
	pub const MinimumDifficulty: u64 = 10000;
	pub const InitialActivityCutoff: u64 = 5000;
	pub const MaximumDifficulty: u64 = u64::MAX/4;
	pub const InitialAdjustmentInterval: u64 = 100;
	pub const InitialMaxRegistrationsPerBlock: u64 = 2;
	pub const InitialTargetRegistrationsPerInterval: u64 = 2;

}

impl pallet_subtensor::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type TransactionByteFee = TransactionByteFee;
	type SDebug = SDebug;
	type InitialRho = InitialRho;
	type InitialKappa = InitialKappa;
	type SelfOwnership = SelfOwnership;
	
	type InitialValidatorBatchSize = InitialValidatorBatchSize;
	type InitialValidatorSequenceLen = InitialValidatorSequenceLen;
	type InitialValidatorEpochLen = InitialValidatorEpochLen;
	type InitialValidatorEpochsPerReset = InitialValidatorEpochsPerReset;

	type InitialImmunityPeriod = InitialImmunityPeriod;
	type InitialMaxAllowedUids = InitialMaxAllowedUids;
	type InitialMinAllowedWeights = InitialMinAllowedWeights;
	type InitialBondsMovingAverage = InitialBondsMovingAverage;
	type InitialMaxAllowedMaxMinRatio = InitialMaxAllowedMaxMinRatio;
	type InitialStakePruningDenominator = InitialStakePruningDenominator;
	type InitialIncentivePruningDenominator = InitialIncentivePruningDenominator;
	type InitialFoundationDistribution = InitialFoundationDistribution;
	type InitialIssuance = InitialIssuance;
	type InitialDifficulty = InitialDifficulty;
	type MinimumDifficulty = MinimumDifficulty;
	type MaximumDifficulty = MaximumDifficulty;
	type InitialBlocksPerStep = InitialBlocksPerStep;
	type InitialActivityCutoff = InitialActivityCutoff;
	type InitialAdjustmentInterval = InitialAdjustmentInterval;
	type InitialMaxRegistrationsPerBlock = InitialMaxRegistrationsPerBlock;
	type InitialTargetRegistrationsPerInterval = InitialTargetRegistrationsPerInterval;
}

// Build genesis storage according to the mock runtime.
#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	sp_tracing::try_init_simple();
	frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

#[allow(dead_code)]
pub fn register_ok_neuron( hotkey_account_id: u64, coldkey_account_id: u64) -> NeuronMetadata<u64> {
	let block_number: u64 = Subtensor::get_current_block_as_u64();
	let (nonce, work): (u64, Vec<u8>) = Subtensor::create_work_for_block_number( block_number, (hotkey_account_id + coldkey_account_id) * 1000000 );
	let result = Subtensor::register( <<Test as frame_system::Config>::Origin>::signed(hotkey_account_id), block_number, nonce, work, hotkey_account_id, coldkey_account_id );
	assert_ok!(result);
	let neuron = Subtensor::get_neuron_for_hotkey(&hotkey_account_id);
	neuron
}

#[allow(dead_code)]
pub fn register_ok_neuron_with_nonce( hotkey_account_id: u64, coldkey_account_id: u64, nonce: u64 ) -> NeuronMetadata<u64> {
	let block_number: u64 = Subtensor::get_current_block_as_u64();
	let (nonce2, work): (u64, Vec<u8>) = Subtensor::create_work_for_block_number( block_number, nonce );
	let result = Subtensor::register( <<Test as frame_system::Config>::Origin>::signed(hotkey_account_id), block_number, nonce2, work, hotkey_account_id, coldkey_account_id );
	assert_ok!(result);
	let neuron = Subtensor::get_neuron_for_hotkey(&hotkey_account_id);
	neuron
}


#[allow(dead_code)]
pub fn serve_axon( hotkey_account_id : u64, version: u32, ip: u128, port: u16, ip_type : u8, modality: u8 ) -> NeuronMetadata<u64> {
	let result = Subtensor::serve_axon(<<Test as frame_system::Config>::Origin>::signed(hotkey_account_id), version, ip, port, ip_type, modality );
	assert_ok!(result);
	let neuron = Subtensor::get_neuron_for_hotkey(&hotkey_account_id);
	neuron
}

#[allow(dead_code)]
pub(crate) fn step_block(n: u64) {
	for _ in 0..n {
		Subtensor::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Subtensor::on_initialize(System::block_number());
	}
}

// Generates an ipv6 address based on 8 ipv6 words and returns it as u128
#[allow(dead_code)]
pub fn ipv6(a: u16, b : u16, c : u16, d : u16, e : u16 ,f: u16, g: u16,h :u16) -> u128 {
	return Ipv6Addr::new(a,b,c,d,e,f,g,h).into();
}

// Generate an ipv4 address based on 4 bytes and returns the corresponding u128, so it can be fed
// to the module::subscribe() function
#[allow(dead_code)]
pub fn ipv4(a: u8 ,b: u8,c : u8,d : u8) -> u128 {
	let ipv4 : Ipv4Addr =  Ipv4Addr::new(a, b, c, d);
	let integer : u32 = ipv4.into();
	return u128::from(integer);
}



/// Test transaction, tuple of (sender, call, signed_extra)
/// with index only used if sender is some.
///
/// If sender is some then the transaction is signed otherwise it is unsigned.
#[derive(PartialEq, Eq, Clone, Encode, Decode)]
pub struct TestXt<Call, Extra> {
	/// Signature of the extrinsic.
	pub signature: Option<(u64, Extra)>,
	/// Call of the extrinsic.
	pub call: Call,
}

#[allow(dead_code)]
impl<Call, Extra> TestXt<Call, Extra> {
	/// Create a new `TextXt`.
	pub fn new(call: Call, signature: Option<(u64, Extra)>) -> Self {
		Self { call, signature }
	}
}

// Non-opaque extrinsics always 0.
parity_util_mem::malloc_size_of_is_0!(any: TestXt<Call, Extra>);

impl<Call, Extra> Serialize for TestXt<Call, Extra> where TestXt<Call, Extra>: Encode {
	fn serialize<S>(&self, seq: S) -> Result<S::Ok, S::Error> where S: Serializer {
		self.using_encoded(|bytes| seq.serialize_bytes(bytes))
	}
}

impl<Call, Extra> Debug for TestXt<Call, Extra> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "TestXt({:?}, ...)", self.signature.as_ref().map(|x| &x.0))
	}
}

impl<Call: Codec + Sync + Send, Context, Extra> Checkable<Context> for TestXt<Call, Extra> {
	type Checked = Self;
	fn check(self, _: &Context) -> Result<Self::Checked, TransactionValidityError> { Ok(self) }
}

impl<Call: Codec + Sync + Send, Extra> traits::Extrinsic for TestXt<Call, Extra> {
	type Call = Call;
	type SignaturePayload = (u64, Extra);

	fn is_signed(&self) -> Option<bool> {
		Some(self.signature.is_some())
	}

	fn new(c: Call, sig: Option<Self::SignaturePayload>) -> Option<Self> {
		Some(TestXt { signature: sig, call: c })
	}

}

impl<Origin, Call, Extra> Applyable for TestXt<Call, Extra> where
	Call: 'static + Sized + Send + Sync + Clone + Eq + Codec + Debug + Dispatchable<Origin=Origin>,
	Extra: SignedExtension<AccountId=u64, Call=Call>,
	Origin: From<Option<u64>>,
{
	type Call = Call;

	/// Checks to see if this is a valid *transaction*. It returns information on it if so.
	fn validate<U: ValidateUnsigned<Call=Self::Call>>(
		&self,
		_source: TransactionSource,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		Ok(Default::default())
	}

	/// Executes all necessary logic needed prior to dispatch and deconstructs into function call,
	/// index and sender.
	fn apply<U: ValidateUnsigned<Call=Self::Call>>(
		self,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> ApplyExtrinsicResultWithInfo<PostDispatchInfoOf<Self::Call>> {
        let maybe_who = if let Some((who, extra)) = self.signature {
			Extra::pre_dispatch(extra, &who, &self.call, info, len)?;
			Some(who)
		} else {
			Extra::pre_dispatch_unsigned(&self.call, info, len)?;
			U::pre_dispatch(&self.call)?;
			None
		};
        
		let res = self.call.dispatch(Origin::from(maybe_who));
		let post_info = match res {
			Ok(info) => info,
			Err(err) => err.post_info,
		};
		// Extra::post_dispatch(info, &post_info, len, &res.map(|_| ()).map_err(|e| e.error))?;
		Ok(res)
	}
}

/// Implementation for unchecked extrinsic.
impl<Call, Extra> GetDispatchInfo
	for TestXt<Call, Extra>
where
	Call: GetDispatchInfo,
	Extra: SignedExtension,
{
	fn get_dispatch_info(&self) -> DispatchInfo {
		self.call.get_dispatch_info()
	}
}






