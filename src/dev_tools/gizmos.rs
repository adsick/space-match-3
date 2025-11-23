use bevy::prelude::*;

pub fn configure_gizmos(mut gizmo_config: ResMut<GizmoConfigStore>) {
    gizmo_config
        .config_mut::<DefaultGizmoConfigGroup>()
        .0
        .line
        .width = 1.0;
}
