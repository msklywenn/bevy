use bevy_core::Byteable;
use bevy_ecs::reflect::ReflectComponent;
use bevy_math::Vec3;
use bevy_reflect::Reflect;
use bevy_render::color::Color;
use bevy_transform::components::GlobalTransform;

/// A point light
#[derive(Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct PointLight {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        PointLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            intensity: 200.0,
            range: 20.0,
            radius: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct PointLightUniform {
    pub pos: [f32; 4],
    pub color: [f32; 4],
    // storing as a `[f32; 4]` for memory alignement
    pub light_params: [f32; 4],
}

unsafe impl Byteable for PointLightUniform {}

impl PointLightUniform {
    pub fn new(light: &PointLight, global_transform: &GlobalTransform) -> PointLightUniform {
        let (x, y, z) = global_transform.translation.into();

        // premultiply color by intensity
        // we don't use the alpha at all, so no reason to multiply only [0..3]
        let color: [f32; 4] = (light.color * light.intensity).into();

        PointLightUniform {
            pos: [x, y, z, 1.0],
            color,
            light_params: [1.0 / (light.range * light.range), light.radius, 0.0, 0.0],
        }
    }
}

/// A Directional light.
///
/// Directional lights don't exist in reality but they are a good
/// approximation for light sources VERY far away, like the sun or
/// the moon.
///
/// An `intensity` of 100000.0 is a good start for a sunlight.
#[derive(Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct DirectionalLight {
    pub color: Color,
    pub intensity: f32,
    direction: Vec3,
}

impl DirectionalLight {
    /// Create a new directional light component.
    ///
    /// # Panics
    /// Will panic if `direction` is not normalized.
    pub fn new(color: Color, intensity: f32, direction: Vec3) -> Self {
        assert!(
            direction.is_normalized(),
            "Light direction vector should have been normalized."
        );
        DirectionalLight {
            color,
            intensity,
            direction,
        }
    }

    /// Set direction of light.
    ///
    /// # Panics
    /// Will panic if `direction` is not normalized.
    pub fn set_direction(&mut self, direction: Vec3) {
        assert!(
            direction.is_normalized(),
            "Light direction vector should have been normalized."
        );
        self.direction = direction;
    }

    pub fn get_direction(&self) -> Vec3 {
        self.direction
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        DirectionalLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            intensity: 100000.0,
            direction: Vec3::new(0.0, -1.0, 0.0),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct DirectionalLightUniform {
    pub dir: [f32; 4],
    pub color: [f32; 4],
}

unsafe impl Byteable for DirectionalLightUniform {}

impl DirectionalLightUniform {
    pub fn new(light: &DirectionalLight) -> DirectionalLightUniform {
        // direction is negated to be ready for N.L
        let dir: [f32; 4] = [
            -light.direction.x,
            -light.direction.y,
            -light.direction.z,
            0.0,
        ];

        // convert from illuminance (lux) to candelas
        //
        // exposure is hard coded at the moment but should be replaced
        // by values coming from the camera
        // see: https://google.github.io/filament/Filament.html#imagingpipeline/physicallybasedcamera/exposuresettings
        const APERTURE: f32 = 4.0;
        const SHUTTER_SPEED: f32 = 1.0 / 250.0;
        const SENSITIVITY: f32 = 100.0;
        let ev100 = f32::log2(APERTURE * APERTURE / SHUTTER_SPEED) - f32::log2(SENSITIVITY / 100.0);
        let exposure = 1.0 / (f32::powf(2.0, ev100) * 1.2);
        let intensity = light.intensity * exposure;

        // premultiply color by intensity
        // we don't use the alpha at all, so no reason to multiply only [0..3]
        let color: [f32; 4] = (light.color * intensity).into();

        DirectionalLightUniform { dir, color }
    }
}

// Ambient light color.
#[derive(Debug)]
pub struct AmbientLight {
    pub color: Color,
    /// Color is premultiplied by brightness before being passed to the shader
    pub brightness: f32,
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: Color::rgb(1.0, 1.0, 1.0),
            brightness: 0.05,
        }
    }
}
