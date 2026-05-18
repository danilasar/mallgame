#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PlacementInvalidReason {
    IntersectsBlockingObject,
    OutsideOwnedStoreArea,
    OutsideWorldBounds,
    WallSurfaceMissing,
    WallAttachmentInvalid,
    WallMountedOverlap,
    DoorAccessBlocked,
}
