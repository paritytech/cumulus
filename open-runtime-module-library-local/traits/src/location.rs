use xcm::v0::{
	Junction::{self, *},
	MultiAsset, MultiLocation,
};

pub trait Parse {
	/// Returns the "chain" location part. It could be parent, sibling
	/// parachain, or child parachain.
	fn chain_part(&self) -> Option<MultiLocation>;
	/// Returns "non-chain" location part.
	fn non_chain_part(&self) -> Option<MultiLocation>;
}

fn is_chain_junction(junction: Option<&Junction>) -> bool {
	matches!(junction, Some(Parent) | Some(Parachain(_)))
}

impl Parse for MultiLocation {
	fn chain_part(&self) -> Option<MultiLocation> {
		match (self.first(), self.at(1)) {
			(Some(Parent), Some(Parachain(id))) => Some((Parent, Parachain(*id)).into()),
			(Some(Parent), _) => Some(Parent.into()),
			(Some(Parachain(id)), _) => Some(Parachain(*id).into()),
			_ => None,
		}
	}

	fn non_chain_part(&self) -> Option<MultiLocation> {
		let mut location = self.clone();
		while is_chain_junction(location.first()) {
			let _ = location.take_first();
		}

		if location != MultiLocation::Null {
			Some(location)
		} else {
			None
		}
	}
}

pub trait Reserve {
	/// Returns assets reserve location.
	fn reserve(&self) -> Option<MultiLocation>;
}

impl Reserve for MultiAsset {
	fn reserve(&self) -> Option<MultiLocation> {
		if let MultiAsset::ConcreteFungible { id, .. } = self {
			id.chain_part()
		} else {
			None
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const PARACHAIN: Junction = Parachain(1);
	const GENERAL_INDEX: Junction = GeneralIndex { id: 1 };

	fn concrete_fungible(id: MultiLocation) -> MultiAsset {
		MultiAsset::ConcreteFungible { id, amount: 1 }
	}

	#[test]
	fn parent_as_reserve_chain() {
		assert_eq!(
			concrete_fungible(MultiLocation::X2(Parent, GENERAL_INDEX)).reserve(),
			Some(Parent.into())
		);
	}

	#[test]
	fn sibling_parachain_as_reserve_chain() {
		assert_eq!(
			concrete_fungible(MultiLocation::X3(Parent, PARACHAIN, GENERAL_INDEX)).reserve(),
			Some((Parent, PARACHAIN).into())
		);
	}

	#[test]
	fn child_parachain_as_reserve_chain() {
		assert_eq!(
			concrete_fungible(MultiLocation::X2(PARACHAIN, GENERAL_INDEX)).reserve(),
			Some(PARACHAIN.into())
		);
	}

	#[test]
	fn no_reserve_chain() {
		assert_eq!(
			concrete_fungible(MultiLocation::X1(GeneralKey("DOT".into()))).reserve(),
			None
		);
	}

	#[test]
	fn non_chain_part_works() {
		assert_eq!(MultiLocation::X1(Parent).non_chain_part(), None);
		assert_eq!(MultiLocation::X2(Parent, PARACHAIN).non_chain_part(), None);
		assert_eq!(MultiLocation::X1(PARACHAIN).non_chain_part(), None);

		assert_eq!(
			MultiLocation::X2(Parent, GENERAL_INDEX).non_chain_part(),
			Some(GENERAL_INDEX.into())
		);
		assert_eq!(
			MultiLocation::X3(Parent, GENERAL_INDEX, GENERAL_INDEX).non_chain_part(),
			Some((GENERAL_INDEX, GENERAL_INDEX).into())
		);
		assert_eq!(
			MultiLocation::X3(Parent, PARACHAIN, GENERAL_INDEX).non_chain_part(),
			Some(GENERAL_INDEX.into())
		);
		assert_eq!(
			MultiLocation::X2(PARACHAIN, GENERAL_INDEX).non_chain_part(),
			Some(GENERAL_INDEX.into())
		);
	}
}
