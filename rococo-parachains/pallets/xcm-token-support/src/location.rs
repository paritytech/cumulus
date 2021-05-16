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
    matches!(junction, Some(Parent) | Some(Parachain { id: _ }))
}

impl Parse for MultiLocation {
    fn chain_part(&self) -> Option<MultiLocation> {
        match (self.first(), self.at(1)) {
            (Some(Parent), Some(Parachain { id })) => Some((Parent, Parachain { id: *id }).into()),
            (Some(Parent), _) => Some(Parent.into()),
            (Some(Parachain { id }), _) => Some(Parachain { id: *id }.into()),
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