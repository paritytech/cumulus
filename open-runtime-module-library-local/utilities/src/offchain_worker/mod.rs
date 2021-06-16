/// Error which may occur while executing the off-chain code.
#[derive(PartialEq)]
pub enum OffchainErr {
	OffchainStore,
	SubmitTransaction,
	NotValidator,
	OffchainLock,
}

impl sp_std::fmt::Debug for OffchainErr {
	fn fmt(&self, fmt: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		match *self {
			OffchainErr::OffchainStore => write!(fmt, "Failed to manipulate offchain store"),
			OffchainErr::SubmitTransaction => write!(fmt, "Failed to submit transaction"),
			OffchainErr::NotValidator => write!(fmt, "Is not validator"),
			OffchainErr::OffchainLock => write!(fmt, "Failed to manipulate offchain lock"),
		}
	}
}
