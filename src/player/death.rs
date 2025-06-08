use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{color::palettes::css::RED, prelude::*, state::commands};

use crate::{
    PausableSystems, Pause,
    menus::Menu,
    red_gas::ExplosionDamage,
    screens::{GameState, Screen},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_damage_overlay)
        .add_systems(
            Update,
            check_explosion_damage
                .run_if(in_state(Screen::Gameplay))
                .run_if(not(in_state(GameState::Intro)))
                .in_set(PausableSystems),
        );
    // .add_systems(OnEnter(GameState::Dead), start_death_animation)
    // .add_systems(Update, animate_death.run_if(in_state(GameState::Dead)));
}

#[derive(Component)]
struct DamageOverlay {}

fn spawn_damage_overlay(
    mut commands: Commands,
    camera: Single<Entity, With<Camera3d>>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let damage_overlay = commands
        .spawn((
            DamageOverlay {},
            StateScoped(Screen::Gameplay),
            Transform::from_translation(Vec3::new(0.0, 0.0, -10.)),
            Mesh3d(meshes.add(Rectangle::from_length(100.))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: (RED * 10.).with_alpha(0.0).into(),
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            })),
        ))
        .id();

    commands.entity(*camera).add_child(damage_overlay);
}

fn check_explosion_damage(
    explosion_damage: Res<ExplosionDamage>,
    overlay: Single<&MeshMaterial3d<StandardMaterial>, With<DamageOverlay>>,

    mut game_state: ResMut<NextState<Menu>>,
    mut phys_time: ResMut<Time<Physics>>,
    mut pause_state: ResMut<NextState<Pause>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(material) = materials.get_mut(*overlay) else {
        return;
    };

    material.base_color.set_alpha(explosion_damage.0);
    // material.alpha_mode = AlphaMode::Mask(explosion_damage.0);

    if explosion_damage.0 >= 1.0 {
        phys_time.pause();
        pause_state.set(Pause(true));
        game_state.set(Menu::Death);
    }
}

// fn start_death_animation(mut death_animation: ResMut<DeathAnimation>) {
//     death_animation.t = 0.0;
// }

// fn animate_death(
//     overlay: Single<&MeshMaterial3d<StandardMaterial>, With<DamageOverlay>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//
//     mut death_animation: ResMut<DeathAnimation>,
//     mut game_state: ResMut<NextState<Menu>>,
//     mut phys_time: ResMut<Time<Physics>>,
// ) {
//     let Some(material) = materials.get_mut(*overlay) else {
//         return;
//     };
//
//     // material.base_color.set_alpha();
//
//     if explosion_damage.0 >= 1.0 {
//         phys_time.pause();
//         pause_state.set(Pause(true));
//         game_state.set(GameState::Dead);
//     }
// }
