use bevy::prelude::*;

use crate::objects::components::{
    InteractionRole, RuntimeOwned, RuntimeOwner, SortLayer, WallMountedPlacement,
    WallOpeningComponent, WallVisualBounds, Wallprint,
};
use crate::presentation::{IsoProjection, world_to_iso};
use crate::store::{WallSurface, wall_surface_visual_offset, wall_surface_world_pos};
use crate::tools::NonInteractive;

/// Toggles for wall-mounted object debug overlays. All off by default.
#[derive(Resource, Default, Debug, Clone)]
pub struct WallDebugOverlaySettings {
    pub show_opening_bounds: bool,
    pub show_wallprint_bounds: bool,
    pub show_visual_bounds: bool,
    pub show_sort_anchor: bool,
    pub show_attachment_point: bool,
}

impl WallDebugOverlaySettings {
    pub fn any_enabled(&self) -> bool {
        self.show_opening_bounds
            || self.show_wallprint_bounds
            || self.show_visual_bounds
            || self.show_sort_anchor
            || self.show_attachment_point
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallDebugOverlayKind {
    OpeningBounds,
    WallprintBounds,
    VisualBounds,
    SortAnchor,
    AttachmentPoint,
}

impl WallDebugOverlayKind {
    pub fn color(self) -> Color {
        match self {
            WallDebugOverlayKind::OpeningBounds => Color::srgb(0.0, 1.0, 1.0),
            WallDebugOverlayKind::WallprintBounds => Color::srgb(1.0, 1.0, 0.0),
            WallDebugOverlayKind::VisualBounds => Color::srgb(1.0, 0.0, 1.0),
            WallDebugOverlayKind::SortAnchor => Color::srgb(0.0, 1.0, 0.0),
            WallDebugOverlayKind::AttachmentPoint => Color::srgb(1.0, 0.5, 0.0),
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct WallDebugOverlay {
    pub target: Entity,
    pub kind: WallDebugOverlayKind,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct WallDebugOverlaySegment;

pub struct WallDebugOverlayPlugin;

impl Plugin for WallDebugOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WallDebugOverlaySettings>()
            .add_systems(PostUpdate, update_wall_debug_overlays);
    }
}

fn wall_rect_points(
    surface: &WallSurface,
    projection: IsoProjection,
    offset_min: f32,
    offset_max: f32,
    height_min: f32,
    height_max: f32,
) -> [Vec2; 4] {
    let p1 = world_to_iso(wall_surface_world_pos(surface, offset_min), projection);
    let p2 = world_to_iso(wall_surface_world_pos(surface, offset_max), projection);
    let v_min = wall_surface_visual_offset(surface, projection, height_min);
    let v_max = wall_surface_visual_offset(surface, projection, height_max);
    [p1 + v_min, p2 + v_min, p2 + v_max, p1 + v_max]
}

fn spawn_outline(
    commands: &mut Commands,
    points: &[Vec2],
    color: Color,
    kind: WallDebugOverlayKind,
    target: Entity,
) {
    let n = points.len();
    for i in 0..n {
        let pa = points[i];
        let pb = points[(i + 1) % n];
        let mid = (pa + pb) * 0.5;
        let delta = pb - pa;
        let len = delta.length();
        if len < 0.1 {
            continue;
        }
        commands.spawn((
            Sprite::from_color(color, Vec2::new(len, 3.0)),
            Transform {
                translation: Vec3::new(
                    mid.x,
                    mid.y,
                    SortLayer::SelectionOverlay.base_z() + 1.0,
                ),
                rotation: Quat::from_rotation_z(delta.y.atan2(delta.x)),
                ..default()
            },
            Visibility::Visible,
            WallDebugOverlay { target, kind },
            WallDebugOverlaySegment,
            InteractionRole::Debug,
            RuntimeOwned { owner: RuntimeOwner::DebugOverlay },
            NonInteractive,
            Name::new(format!("WallDebugOverlay {:?}", kind)),
        ));
    }
}

fn spawn_cross(
    commands: &mut Commands,
    center: Vec2,
    size: f32,
    color: Color,
    kind: WallDebugOverlayKind,
    target: Entity,
) {
    for angle in [0.0_f32, std::f32::consts::FRAC_PI_2] {
        commands.spawn((
            Sprite::from_color(color, Vec2::new(size, 3.0)),
            Transform {
                translation: Vec3::new(
                    center.x,
                    center.y,
                    SortLayer::SelectionOverlay.base_z() + 1.0,
                ),
                rotation: Quat::from_rotation_z(angle),
                ..default()
            },
            Visibility::Visible,
            WallDebugOverlay { target, kind },
            WallDebugOverlaySegment,
            InteractionRole::Debug,
            RuntimeOwned { owner: RuntimeOwner::DebugOverlay },
            NonInteractive,
            Name::new(format!("WallDebugOverlayCross {:?}", kind)),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_wall_debug_overlays(
    mut commands: Commands,
    settings: Res<WallDebugOverlaySettings>,
    existing: Query<Entity, With<WallDebugOverlaySegment>>,
    projection: Res<IsoProjection>,
    surfaces: Query<&WallSurface>,
    openings: Query<(Entity, &WallOpeningComponent)>,
    wallprints: Query<(Entity, &Wallprint, &WallMountedPlacement)>,
    visual_bounds: Query<(Entity, &WallVisualBounds)>,
    mounted: Query<(Entity, &WallMountedPlacement)>,
) {
    // Despawn all previous overlay segments every frame
    for e in existing.iter() {
        commands.entity(e).try_despawn();
    }

    if !settings.any_enabled() {
        return;
    }

    if settings.show_opening_bounds {
        for (target, opening) in openings.iter() {
            let Some(surface) = surfaces.iter().find(|s| s.key == opening.segment_key) else {
                continue;
            };
            let pts = wall_rect_points(
                surface,
                *projection,
                opening.offset_min,
                opening.offset_max,
                opening.height_min,
                opening.height_max,
            );
            spawn_outline(
                &mut commands,
                &pts,
                WallDebugOverlayKind::OpeningBounds.color(),
                WallDebugOverlayKind::OpeningBounds,
                target,
            );
        }
    }

    if settings.show_wallprint_bounds {
        for (target, wallprint, _) in wallprints.iter() {
            for rect in &wallprint.rects {
                let Some(surface) = surfaces.iter().find(|s| s.key == rect.segment_key) else {
                    continue;
                };
                let pts = wall_rect_points(
                    surface,
                    *projection,
                    rect.offset_min,
                    rect.offset_max,
                    rect.height_min,
                    rect.height_max,
                );
                spawn_outline(
                    &mut commands,
                    &pts,
                    WallDebugOverlayKind::WallprintBounds.color(),
                    WallDebugOverlayKind::WallprintBounds,
                    target,
                );
            }
        }
    }

    if settings.show_visual_bounds {
        for (target, vb) in visual_bounds.iter() {
            let Some(surface) = surfaces.iter().find(|s| s.key == vb.segment_key) else {
                continue;
            };
            let pts = wall_rect_points(
                surface,
                *projection,
                vb.offset_min,
                vb.offset_max,
                vb.height_min,
                vb.height_max,
            );
            spawn_outline(
                &mut commands,
                &pts,
                WallDebugOverlayKind::VisualBounds.color(),
                WallDebugOverlayKind::VisualBounds,
                target,
            );
        }
    }

    if settings.show_sort_anchor {
        for (target, placement) in mounted.iter() {
            let att = placement.attachment;
            let Some(surface) = surfaces.iter().find(|s| s.key == att.segment_key) else {
                continue;
            };
            let world = wall_surface_world_pos(surface, att.offset_along_segment);
            let iso = world_to_iso(world, *projection);
            spawn_cross(
                &mut commands,
                iso,
                12.0,
                WallDebugOverlayKind::SortAnchor.color(),
                WallDebugOverlayKind::SortAnchor,
                target,
            );
        }
    }

    if settings.show_attachment_point {
        for (target, placement) in mounted.iter() {
            let att = placement.attachment;
            let Some(surface) = surfaces.iter().find(|s| s.key == att.segment_key) else {
                continue;
            };
            let world = wall_surface_world_pos(surface, att.offset_along_segment);
            let iso = world_to_iso(world, *projection);
            let v = wall_surface_visual_offset(surface, *projection, att.height_on_wall);
            spawn_cross(
                &mut commands,
                iso + v,
                10.0,
                WallDebugOverlayKind::AttachmentPoint.color(),
                WallDebugOverlayKind::AttachmentPoint,
                target,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::components::StoreObject;
    use crate::store::{StoreBoundarySide, StoreChunkCoord, WallSegmentKey};

    fn test_segment_key() -> WallSegmentKey {
        WallSegmentKey {
            chunk: StoreChunkCoord { x: 0, y: 0 },
            side: StoreBoundarySide::Top,
        }
    }

    #[test]
    fn settings_any_enabled() {
        let mut s = WallDebugOverlaySettings::default();
        assert!(!s.any_enabled());
        s.show_opening_bounds = true;
        assert!(s.any_enabled());
        s.show_opening_bounds = false;
        s.show_visual_bounds = true;
        assert!(s.any_enabled());
        s.show_visual_bounds = false;
        s.show_wallprint_bounds = true;
        assert!(s.any_enabled());
    }

    #[test]
    fn overlay_kinds_have_distinct_colors() {
        let kinds = [
            WallDebugOverlayKind::OpeningBounds,
            WallDebugOverlayKind::WallprintBounds,
            WallDebugOverlayKind::VisualBounds,
            WallDebugOverlayKind::SortAnchor,
            WallDebugOverlayKind::AttachmentPoint,
        ];
        let colors: Vec<Color> = kinds.iter().map(|k| k.color()).collect();
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(
                    colors[i], colors[j],
                    "Kinds {:?} and {:?} share the same color",
                    kinds[i], kinds[j]
                );
            }
        }
    }

    #[test]
    fn overlay_entities_are_not_store_object_and_are_non_interactive() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.init_asset::<Image>();
        crate::store::commands::register_test_messages(&mut app);

        app.insert_resource(IsoProjection::default());
        app.insert_resource(WallDebugOverlaySettings {
            show_opening_bounds: true,
            show_wallprint_bounds: true,
            show_visual_bounds: false,
            show_sort_anchor: false,
            show_attachment_point: false,
        });

        // Spawn a WallSurface so the overlay system can find a surface for the opening
        let seg_key = test_segment_key();
        app.world_mut().spawn(WallSurface {
            key: seg_key,
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(256.0, 0.0),
            length: 256.0,
            height: 192.0,
            thickness: 8.0,
            normal: Vec2::Y,
        });

        // Spawn a mock wall-mounted object with WallOpeningComponent and Wallprint
        use crate::objects::components::{
            WallAttachmentPoint, WallMountedPlacement, Wallprint, WallprintRect, WallOccupancyKind,
        };
        let att = WallAttachmentPoint {
            segment_key: seg_key,
            offset_along_segment: 100.0,
            height_on_wall: 56.0,
        };
        app.world_mut().spawn((
            WallOpeningComponent {
                segment_key: seg_key,
                offset_min: 64.0,
                offset_max: 136.0,
                height_min: 56.0,
                height_max: 128.0,
                glass_color: None,
                frame_color: None,
            },
            WallMountedPlacement { attachment: att },
            Wallprint {
                rects: vec![WallprintRect {
                    segment_key: seg_key,
                    offset_min: 64.0,
                    offset_max: 136.0,
                    height_min: 56.0,
                    height_max: 128.0,
                    occupancy_kind: WallOccupancyKind::Opening,
                }],
            },
        ));

        app.add_systems(Update, update_wall_debug_overlays);
        app.update();

        let world = app.world_mut();

        // No overlay entity must be StoreObject
        let store_obj_count = world
            .query_filtered::<Entity, (With<WallDebugOverlaySegment>, With<StoreObject>)>()
            .iter(world)
            .count();
        assert_eq!(store_obj_count, 0, "overlay entities must not be StoreObject");

        // All overlay entities must be NonInteractive
        let total_overlays = world
            .query::<&WallDebugOverlaySegment>()
            .iter(world)
            .count();
        let non_interactive_overlays = world
            .query_filtered::<Entity, (With<WallDebugOverlaySegment>, With<NonInteractive>)>()
            .iter(world)
            .count();
        assert_eq!(
            total_overlays, non_interactive_overlays,
            "all overlay entities must be NonInteractive"
        );

        // All overlay entities must be RuntimeOwned { DebugOverlay }
        let runtime_owned_overlays = world
            .query_filtered::<&RuntimeOwned, With<WallDebugOverlaySegment>>()
            .iter(world)
            .filter(|ro| ro.owner == RuntimeOwner::DebugOverlay)
            .count();
        assert_eq!(
            total_overlays, runtime_owned_overlays,
            "all overlay entities must be RuntimeOwned::DebugOverlay"
        );

        // With show_opening_bounds + show_wallprint_bounds enabled and a surface present,
        // we should have spawned some overlay segments (opening rect = 4 lines, wallprint = 4 lines)
        assert!(total_overlays > 0, "should have spawned overlay segments");
    }

    #[test]
    fn no_overlays_spawned_when_all_settings_off() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.init_asset::<Image>();

        app.insert_resource(IsoProjection::default());
        app.insert_resource(WallDebugOverlaySettings::default()); // all false

        let seg_key = test_segment_key();
        app.world_mut().spawn(WallSurface {
            key: seg_key,
            start: Vec2::ZERO,
            end: Vec2::new(256.0, 0.0),
            length: 256.0,
            height: 192.0,
            thickness: 8.0,
            normal: Vec2::Y,
        });

        use crate::objects::components::WallOpeningComponent;
        app.world_mut().spawn(WallOpeningComponent {
            segment_key: seg_key,
            offset_min: 64.0,
            offset_max: 136.0,
            height_min: 56.0,
            height_max: 128.0,
            glass_color: None,
            frame_color: None,
        });

        app.add_systems(Update, update_wall_debug_overlays);
        app.update();

        let world = app.world_mut();
        let count = world.query::<&WallDebugOverlaySegment>().iter(world).count();
        assert_eq!(count, 0, "no overlays when all settings are off");
    }
}
