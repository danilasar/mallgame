use bevy::prelude::*;

use crate::input::{InputAction, PointerContext};
use crate::objects::components::{Movable, Selected, WorldPos};
use crate::tools::{
    ActiveToolAction, MoveObjectCommitted, StartMoveObjectRequested, ToolContext, ToolDescriptor,
    ToolInputGate, ToolMode, ToolRegistry, ToolSet,
};

pub struct MoveToolPlugin;

impl Plugin for MoveToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolRegistry>();
        app.world_mut()
            .resource_mut::<ToolRegistry>()
            .register(ToolDescriptor {
                mode: ToolMode::Move,
                action: InputAction::ToolMove,
                label: "Move",
            });

        app.add_systems(
            Update,
            (
                start_move_object_requests.run_if(in_state(ToolMode::Move)),
                move_tool_system.run_if(in_state(ToolMode::Move)),
            )
                .chain()
                .in_set(ToolSet::ToolUpdate),
        )
        .add_systems(OnExit(ToolMode::Move), cancel_active_move_on_exit);
    }
}

pub fn start_move_object_requests(
    mut commands: Commands,
    mut requests: MessageReader<StartMoveObjectRequested>,
    movable: Query<(), With<Movable>>,
    selected: Query<Entity, With<Selected>>,
    positions: Query<&WorldPos>,
    mut tool: ResMut<ToolContext>,
) {
    for request in requests.read() {
        if movable.get(request.entity).is_err() {
            continue;
        }
        for selected_entity in &selected {
            commands.entity(selected_entity).remove::<Selected>();
        }
        commands.entity(request.entity).insert(Selected);

        if let Ok(world_pos) = positions.get(request.entity) {
            tool.active = Some(ActiveToolAction::Moving {
                entity: request.entity,
                original_world_pos: world_pos.0,
                current_world_pos: world_pos.0,
                valid: true,
            });
        }
    }
}

pub fn move_tool_system(
    mut commands: Commands,
    pointer: Res<PointerContext>,
    gate: Res<ToolInputGate>,
    movable: Query<(), With<Movable>>,
    mut positions: Query<&mut WorldPos>,
    selected: Query<Entity, With<Selected>>,
    mut tool: ResMut<ToolContext>,
    mut committed: MessageWriter<MoveObjectCommitted>,
) {
    tool.sync_from_pointer(&pointer);

    if !gate.can_use_world() {
        return;
    }

    if gate.cancel_requested {
        cancel_active_move(&mut tool, &mut positions);
        return;
    }

    let pointer_world = tool.pointer_world;
    if let Some(ActiveToolAction::Moving {
        entity,
        original_world_pos,
        current_world_pos,
        valid,
    }) = tool.active.as_mut()
    {
        *current_world_pos = pointer_world;
        if let Ok(mut world_pos) = positions.get_mut(*entity) {
            world_pos.0 = *current_world_pos;
        }

        if gate.primary_click_released {
            let entity = *entity;
            let old_pos = *original_world_pos;
            let new_pos = *current_world_pos;
            let is_valid = *valid;

            if is_valid {
                committed.write(MoveObjectCommitted {
                    entity,
                    old_pos,
                    new_pos,
                });
            } else if let Ok(mut world_pos) = positions.get_mut(entity) {
                world_pos.0 = old_pos;
            }
            tool.active = None;
        }
        return;
    }

    if gate.primary_click_released {
        if let Some(entity) = tool.hovered.filter(|entity| movable.get(*entity).is_ok()) {
            for selected_entity in &selected {
                commands.entity(selected_entity).remove::<Selected>();
            }
            commands.entity(entity).insert(Selected);

            if let Ok(world_pos) = positions.get(entity) {
                tool.active = Some(ActiveToolAction::Moving {
                    entity,
                    original_world_pos: world_pos.0,
                    current_world_pos: pointer_world,
                    valid: true,
                });
            }
        }
    }
}

pub fn cancel_active_move(tool: &mut ToolContext, positions: &mut Query<&mut WorldPos>) {
    if let Some(ActiveToolAction::Moving {
        entity,
        original_world_pos,
        ..
    }) = tool.active.take()
    {
        if let Ok(mut world_pos) = positions.get_mut(entity) {
            world_pos.0 = original_world_pos;
        }
    }
}

pub fn cancel_active_move_on_exit(
    mut tool: ResMut<ToolContext>,
    mut positions: Query<&mut WorldPos>,
) {
    cancel_active_move(&mut tool, &mut positions);
}
