#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementInvalidReason {
    IntersectsBlockingObject,
    OutsideOwnedStoreArea,
    OutsideWorldBounds,
}
