use bevy::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct SelectionState {
    pub primary: Option<Entity>,
}

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionState>()
            .add_systems(Update, cleanup_stale_selection);
    }
}

fn cleanup_stale_selection(
    mut selection: ResMut<SelectionState>,
    entities: Query<Entity>,
) {
    if let Some(primary) = selection.primary {
        if entities.get(primary).is_err() {
            info!("Clearing stale selection: {:?}", primary);
            selection.primary = None;
        }
    }
}
