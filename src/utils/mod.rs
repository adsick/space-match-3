use bevy::{
    app::{App, Update}, color::{Color, Mix}, math::VectorSpace, pbr::{MeshMaterial3d, PointLight, StandardMaterial}, prelude::IntoScheduleConfigs
};
use bevy_tweening::{
    AnimationSystem, Lens, Targetable, asset_animator_system, component_animator_system,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        asset_animator_system::<StandardMaterial, MeshMaterial3d<StandardMaterial>>
            .in_set(AnimationSystem::AnimationUpdate),
    )
    .add_systems(
        Update,
        component_animator_system::<PointLight>.in_set(AnimationSystem::AnimationUpdate),
    );
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StandardMaterialLens {
    pub color_start: Color,
    pub color_end: Color,

    pub emissive_start: Color,
    pub emissive_end: Color,
}

impl Lens<StandardMaterial> for StandardMaterialLens {
    fn lerp(&mut self, target: &mut dyn Targetable<StandardMaterial>, ratio: f32) {
        target.base_color = self.color_start.mix(&self.color_end, ratio);
        target.emissive = self.emissive_start.mix(&self.emissive_end, ratio).into();
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PointLightLens {
    pub color_start: Color,
    pub color_end: Color,

    pub intensity_start: f32,
    pub intensity_end: f32,
}

impl Lens<PointLight> for PointLightLens {
    fn lerp(&mut self, target: &mut dyn Targetable<PointLight>, ratio: f32) {
        target.color = self.color_start.mix(&self.color_end, ratio);
        target.intensity = self.intensity_start.lerp(self.intensity_end, ratio);
    }
}
