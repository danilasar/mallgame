use bevy::prelude::*;

use crate::objects::prototypes::{
    BuildObjectId, BuildRibbonTab, BuildSelectionState, CatalogAvailability, ObjectCatalog,
    SelectBuildPrototypeRequested,
};
use crate::tools::{ActivateToolRequested, ToolActivationKind, ToolMode};
use crate::ui::buttons::{UiFonts, label_text, ui_button, ui_text};
use crate::ui::{BlocksWorldInput, UiRoot, UiSet};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RibbonState {
    pub active_tab: BuildRibbonTab,
    pub is_open: bool,
}

impl Default for RibbonState {
    fn default() -> Self {
        Self {
            active_tab: BuildRibbonTab::Fixtures,
            is_open: false,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
struct RibbonRoot;

#[derive(Component, Debug, Clone, Copy)]
struct RibbonTabButton(BuildRibbonTab);

#[derive(Component, Debug, Clone)]
struct RibbonItemButton {
    prototype_id: BuildObjectId,
}

#[derive(Component, Debug, Clone, Copy)]
struct RibbonCloseButton;

pub struct BuildRibbonPlugin;

impl Plugin for BuildRibbonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RibbonState>()
            .add_systems(Startup, setup_ribbon_ui.after(crate::ui::setup_ui_root))
            .add_systems(
                Update,
                (
                    sync_ribbon_visibility_system,
                    ribbon_tab_button_system,
                    ribbon_item_button_system,
                    ribbon_close_button_system,
                    render_ribbon_content_system,
                )
                    .chain()
                    .in_set(UiSet::Requests),
            );
    }
}

fn sync_ribbon_visibility_system(
    tool_mode: Res<State<ToolMode>>,
    mut ribbon_state: ResMut<RibbonState>,
) {
    if tool_mode.is_changed() {
        match *tool_mode.get() {
            ToolMode::Build | ToolMode::Expansion => {
                ribbon_state.is_open = true;
                if *tool_mode.get() == ToolMode::Expansion {
                    ribbon_state.active_tab = BuildRibbonTab::Store;
                }
            }
            _ => {
                ribbon_state.is_open = false;
            }
        }
    }
}

fn setup_ribbon_ui(mut commands: Commands, root: Query<Entity, With<UiRoot>>) {
    let Some(root) = root.iter().next() else {
        return;
    };

    let ribbon = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(18.0),
                right: Val::Px(118.0),
                bottom: Val::Px(18.0),
                height: Val::Px(160.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(0.0),
                ..default()
            },
            BlocksWorldInput,
            Interaction::default(),
            RibbonRoot,
            Name::new("BuildRibbon"),
        ))
        .id();
    commands.entity(root).add_child(ribbon);
}

fn ribbon_tab_button_system(
    mut state: ResMut<RibbonState>,
    query: Query<(&Interaction, &RibbonTabButton), Changed<Interaction>>,
    mut activation: MessageWriter<ActivateToolRequested>,
) {
    for (interaction, button) in &query {
        if *interaction == Interaction::Pressed {
            state.active_tab = button.0;

            let target_mode = match button.0 {
                BuildRibbonTab::Store => ToolMode::Expansion,
                _ => ToolMode::Build,
            };

            activation.write(ActivateToolRequested {
                mode: target_mode,
                kind: ToolActivationKind::Replace,
            });
        }
    }
}

fn ribbon_item_button_system(
    query: Query<(&Interaction, &RibbonItemButton), Changed<Interaction>>,
    mut selection: MessageWriter<SelectBuildPrototypeRequested>,
) {
    for (interaction, button) in &query {
        if *interaction == Interaction::Pressed {
            selection.write(SelectBuildPrototypeRequested {
                prototype_id: button.prototype_id.clone(),
            });
        }
    }
}

fn ribbon_close_button_system(
    mut state: ResMut<RibbonState>,
    query: Query<&Interaction, (With<RibbonCloseButton>, Changed<Interaction>)>,
    mut activation: MessageWriter<ActivateToolRequested>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            state.is_open = false;
            activation.write(ActivateToolRequested {
                mode: ToolMode::Cursor,
                kind: ToolActivationKind::Replace,
            });
        }
    }
}

fn render_ribbon_content_system(
    mut commands: Commands,
    state: Res<RibbonState>,
    selection: Res<BuildSelectionState>,
    catalog: Res<ObjectCatalog>,
    fonts: Res<UiFonts>,
    mut root_query: Query<(Entity, &mut Node), With<RibbonRoot>>,
    children_query: Query<&Children>,
) {
    if !state.is_changed() && !selection.is_changed() && !catalog.is_changed() {
        return;
    }

    let Ok((root, mut root_node)) = root_query.single_mut() else {
        return;
    };

    if !state.is_open {
        root_node.display = Display::None;
        return;
    }
    root_node.display = Display::Flex;

    // Despawn old content
    if let Ok(children) = children_query.get(root) {
        for &child in children {
            commands.entity(child).despawn();
        }
    }

    commands.entity(root).with_children(|parent| {
        // 1. Tabs Row
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|tabs_row| {
                for tab in [
                    BuildRibbonTab::Fixtures,
                    BuildRibbonTab::Service,
                    BuildRibbonTab::Decor,
                    BuildRibbonTab::Walls,
                    BuildRibbonTab::Store,
                ] {
                    let is_active = state.active_tab == tab;
                    let bg = if is_active {
                        Color::srgba(0.18, 0.16, 0.14, 0.98)
                    } else {
                        Color::srgba(0.12, 0.10, 0.08, 0.85)
                    };

                    tabs_row
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                border_radius: BorderRadius::top(Val::Px(6.0)),
                                ..default()
                            },
                            BackgroundColor(bg),
                            RibbonTabButton(tab),
                        ))
                        .with_child(label_text(tab.label(), &fonts));
                }

                // Spacer
                tabs_row.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });

                // Close button
                tabs_row
                    .spawn((
                        ui_button("X", 32.0, 28.0),
                        RibbonCloseButton,
                    ))
                    .insert(BackgroundColor(Color::srgba(0.3, 0.1, 0.1, 0.8)))
                    .with_child(label_text("X", &fonts));
            });

        // 2. Main Content Area
        parent
            .spawn((
                Node {
                    height: Val::Px(126.0),
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    border_radius: BorderRadius::bottom(Val::Px(8.0)),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(16.0),
                    overflow: Overflow::clip_x(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.18, 0.16, 0.14, 0.98)),
            ))
            .with_children(|content_area| {
                if state.active_tab == BuildRibbonTab::Store {
                    content_area.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..default()
                    }).with_children(|col| {
                        col.spawn(ui_text("Expansion", 14.0, Color::WHITE, &fonts));
                        col.spawn(ui_text("Select an adjacent 4x4 block on the map to expand your store area.", 12.0, Color::srgb(0.7, 0.7, 0.6), &fonts));
                    });
                    return;
                }

                if state.active_tab == BuildRibbonTab::Walls {
                    content_area.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..default()
                    }).with_children(|col| {
                        col.spawn(ui_text("Wall-mounted preview objects", 14.0, Color::WHITE, &fonts));
                        col.spawn(ui_text("Preview only in Stage 5B.2. Click to attach the dev wall decor placeholder to a wall surface.", 12.0, Color::srgb(0.7, 0.7, 0.6), &fonts));
                    });
                }

                // Filter prototypes for active tab
                let mut prototypes: Vec<_> = catalog
                    .prototypes
                    .values()
                    .filter(|p| {
                        p.catalog.ribbon_tab == state.active_tab
                            && p.catalog.availability == CatalogAvailability::Available
                    })
                    .collect();

                prototypes.sort_by_key(|p| (p.catalog.ribbon_group, p.catalog.sort_order));

                for proto in prototypes {
                    let is_selected = selection.selected_prototype_id.as_ref() == Some(&proto.id);
                    let border_color = if is_selected {
                        Color::srgb(0.9, 0.7, 0.3)
                    } else {
                        Color::NONE
                    };

                    content_area.spawn((
                        Button,
                        Node {
                            width: Val::Px(84.0),
                            height: Val::Px(100.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::all(Val::Px(4.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor::all(border_color),
                        BackgroundColor(Color::srgb(0.12, 0.11, 0.10)),
                        RibbonItemButton { prototype_id: proto.id.clone() },
                    )).with_children(|btn| {
                        // Icon placeholder
                        btn.spawn(Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            ..default()
                        }).insert(BackgroundColor(Color::srgb(0.2, 0.2, 0.2)));

                        btn.spawn(ui_text(&proto.display.display_name, 11.0, Color::WHITE, &fonts));
                    });
                }
            });
    });
}
