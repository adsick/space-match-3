use bevy::{prelude::*, window::PresentMode};
use bevy_tweening::{AnimationSystem, Lens};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StandardMaterialLens {
    pub color_start: Color,
    pub color_end: Color,

    pub emissive_start: Color,
    pub emissive_end: Color,
}

impl Lens<StandardMaterial> for StandardMaterialLens {
    fn lerp(&mut self, mut target: Mut<StandardMaterial>, ratio: f32) {
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
    fn lerp(&mut self, mut target: Mut<PointLight>, ratio: f32) {
        target.color = self.color_start.mix(&self.color_end, ratio);
        target.intensity = self.intensity_start.lerp(self.intensity_end, ratio);
    }
}

pub fn toggle_vsync(mut window: Single<&mut Window>) {
    window.present_mode = match window.present_mode {
        PresentMode::AutoVsync => PresentMode::AutoNoVsync,
        PresentMode::AutoNoVsync => PresentMode::AutoVsync,
        _ => panic!(),
    };
}
