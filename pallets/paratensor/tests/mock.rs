use frame_support::{parameter_types, traits::Everything};
use frame_system::{limits};
use frame_support::traits:: StorageMapShim;
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
		ParatensorModule: pallet_paratensor::{Pallet, Call, Storage, Event<T>},
	}
);

#[allow(dead_code)]
pub type ParatensorCall = pallet_paratensor::Call<Test>;

#[allow(dead_code)]
pub type BalanceCall = pallet_balances::Call<Test>;

#[allow(dead_code)]
pub type TestRuntimeCall = frame_system::Call<Test>;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

#[allow(dead_code)]
pub type AccountId = u64;

/// Balance of an account.
#[allow(dead_code)]
pub type Balance = u128;

/// An index to a block.
#[allow(dead_code)]
pub type BlockNumber = u64;

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

impl system::Config for Test {
	type BaseCallFilter = Everything;
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

parameter_types! {
	pub const InitialMinAllowedWeights: u16 = 0;
	pub const InitialMaxAllowedMaxMinRatio: u16 = 0;
	pub BlockWeights: limits::BlockWeights = limits::BlockWeights::simple_max(1024);
	pub const ExistentialDeposit: Balance = 1;
	pub const TransactionByteFee: Balance = 100;
	pub const SDebug:u64 = 1;
	pub const InitialRho: u16 = 10;
	pub const InitialKappa: u16 = 2;
	pub const InitialTempo: u16 = 0;
	pub const SelfOwnership: u64 = 2;
	pub const InitialImmunityPeriod: u16 = 2;
	pub const InitialMaxAllowedUids: u16 = 100;
	pub const InitialBondsMovingAverage: u64 = 500_000;
	pub const InitialIncentivePruningDenominator: u16 = 1;
	pub const InitialStakePruningDenominator: u16 = 1;
	pub const InitialStakePruningMin: u16 = 0;
	pub const InitialFoundationDistribution: u64 = 0;

	pub const InitialValidatorBatchSize: u16 = 10;
	pub const InitialValidatorSequenceLen: u16 = 10;
	pub const InitialValidatorEpochLen: u16 = 10;
	pub const InitialValidatorEpochsPerReset: u16 = 10;

	pub const InitialBlocksPerStep: u64 = 1;
	pub const InitialIssuance: u64 = 548833985028256;
	pub const InitialGlobalN : u16 = 0;
	pub const InitialDifficulty: u64 = 10000;
	pub const MinimumDifficulty: u64 = 10000;
	pub const InitialActivityCutoff: u16 = 5000;
	pub const MaximumDifficulty: u64 = u64::MAX/4;
	pub const InitialAdjustmentInterval: u64 = 100;
	pub const InitialMaxRegistrationsPerBlock: u16 = 2;
	pub const InitialTargetRegistrationsPerInterval: u64 = 2;
	pub const InitialPrunningScore : u16 = u16::MAX;
}
impl pallet_paratensor::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type InitialIssuance = InitialIssuance;
	type InitialGlobalN = InitialGlobalN;

	type InitialMinAllowedWeights = InitialMinAllowedWeights;
	type InitialMaxAllowedMaxMinRatio = InitialMaxAllowedMaxMinRatio;
	type InitialBlocksPerStep = InitialBlocksPerStep;
	type InitialTempo = InitialTempo;
	type InitialDifficulty = InitialDifficulty;
	type InitialAdjustmentInterval = InitialAdjustmentInterval;
	type InitialTargetRegistrationsPerInterval = InitialTargetRegistrationsPerInterval;
	type InitialRho = InitialRho;
	type InitialKappa = InitialKappa;
	type InitialMaxAllowedUids = InitialMaxAllowedUids;
	type InitialValidatorBatchSize = InitialValidatorBatchSize;
	type InitialValidatorSequenceLen = InitialValidatorSequenceLen;
	type InitialValidatorEpochLen = InitialValidatorEpochLen;
	type InitialValidatorEpochsPerReset = InitialValidatorEpochsPerReset;
	type InitialIncentivePruningDenominator = InitialIncentivePruningDenominator;
	type InitialStakePruningDenominator = InitialStakePruningDenominator;
	type InitialStakePruningMin = InitialStakePruningMin;
	type InitialImmunityPeriod = InitialImmunityPeriod;
	type InitialActivityCutoff = InitialActivityCutoff;
	type InitialMaxRegistrationsPerBlock = InitialMaxRegistrationsPerBlock;
	type InitialPrunningScore = InitialPrunningScore;
}

// Build genesis storage according to the mock runtime.
//pub fn new_test_ext() -> sp_io::TestExternalities {
//	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
//}

// Build genesis storage according to the mock runtime.
#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	sp_tracing::try_init_simple();
	frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

#[allow(dead_code)]
pub fn test_ext_with_balances(balances : Vec<(u64, u128)>) -> sp_io::TestExternalities {
	sp_tracing::try_init_simple();
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	pallet_balances::GenesisConfig::<Test> { balances }
		.assimilate_storage(&mut t)
		.unwrap();

	t.into()
}

