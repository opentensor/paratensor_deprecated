use frame_support::{parameter_types, traits::{StorageMapShim}};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

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

thread_local!{
	pub static RUNTIME_VERSION: std::cell::RefCell<sp_version::RuntimeVersion> =
		Default::default();
}

/// Balance of an account.
#[allow(dead_code)]
pub type Balance = u128;

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
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

#[allow(dead_code)]
pub type AccountId = u64;

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type Event = ();
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
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub const SDebug:u64 = 1;
	pub const InitialRho: u64 = 10;
	pub const InitialKappa: u64 = 2;
	pub const SelfOwnership: u64 = 2;
	pub const InitialDifficulty: u64 = 10000;
	pub const MinimumDifficulty: u64 = 10000;
	pub const MaximumDifficulty: u64 = u64::MAX/4;
	pub const InitialAdjustmentInterval: u64 = 100;
	pub const InitialTargetRegistrationsPerInterval: u64 = 2;
	pub const InitialBondsMovingAverage: u64 = 900_000;
	pub const InitialActivityCutoff: u64 = 5000;
	pub const InitialIssuance: u64 = 548833985028256;

}

impl pallet_subtensor::Config for Test {
	type Event = Event;
	type SDebug = SDebug;
	type Currency = Balances;
	type InitialRho = InitialRho;
	type InitialKappa = InitialKappa;
	type SelfOwnership = SelfOwnership;
	type MinimumDifficulty = MinimumDifficulty;
	type MaximumDifficulty = MaximumDifficulty;
	type InitialDifficulty = InitialDifficulty;
	type InitialAdjustmentInterval = InitialAdjustmentInterval;
	type InitialTargetRegistrationsPerInterval = InitialTargetRegistrationsPerInterval;
	type InitialBondsMovingAverage = InitialBondsMovingAverage;
	type InitialActivityCutoff = InitialActivityCutoff;
	type InitialIssuance = InitialIssuance;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

