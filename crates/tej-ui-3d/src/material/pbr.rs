//! Physically Based Rendering (PBR) Metallic-Roughness shader with Spherical Harmonics.
//!
//! Implements Cook-Torrance microfacet specular GGX with Smith masking, Fresnel-Schlick,
//! and 9-coefficient Spherical Harmonics ($L \le 2$) for zero-texture-fetch ambient irradiance.

use crate::math::Vec3;
use tej_ui_hal::Color;

/// 9-Coefficient Spherical Harmonics ($L \le 2$) ambient irradiance environment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphericalHarmonics9 {
    /// Coefficients for $Y_{0,0}, Y_{1,-1}, Y_{1,0}, Y_{1,1}, Y_{2,-2}, Y_{2,-1}, Y_{2,0}, Y_{2,1}, Y_{2,2}$.
    pub coeffs: [Vec3; 9],
}

impl Default for SphericalHarmonics9 {
    fn default() -> Self {
        Self::studio_lighting()
    }
}

impl SphericalHarmonics9 {
    /// Neutral warm studio environment irradiance coefficients.
    #[must_use]
    pub const fn studio_lighting() -> Self {
        Self {
            coeffs: [
                Vec3::new(0.35, 0.36, 0.38), // L00
                Vec3::new(0.05, 0.05, 0.06), // L1-1
                Vec3::new(0.12, 0.12, 0.14), // L10 (Sky/Ground gradient)
                Vec3::new(0.04, 0.04, 0.05), // L11
                Vec3::new(0.02, 0.02, 0.02), // L2-2
                Vec3::new(0.01, 0.01, 0.01), // L2-1
                Vec3::new(0.03, 0.03, 0.04), // L20
                Vec3::new(0.01, 0.01, 0.01), // L21
                Vec3::new(0.02, 0.02, 0.02), // L22
            ],
        }
    }

    /// Evaluates diffuse ambient irradiance $E(\vec{n})$ for a given unit normal.
    #[must_use]
    pub fn eval(&self, n: Vec3) -> Vec3 {
        let x = n.x;
        let y = n.y;
        let z = n.z;

        // Spherical Harmonic basis polynomials (Stupid Spherical Harmonics Tricks - Sloan 2008)
        let c0 = 0.282_095;
        let c1 = 0.488_603;
        let c2 = 1.092_548;
        let c3 = 0.315_392;
        let c4 = 0.546_274;

        let mut irr = self.coeffs[0] * c0;
        irr = irr + self.coeffs[1] * (c1 * y);
        irr = irr + self.coeffs[2] * (c1 * z);
        irr = irr + self.coeffs[3] * (c1 * x);
        irr = irr + self.coeffs[4] * (c2 * x * y);
        irr = irr + self.coeffs[5] * (c2 * y * z);
        irr = irr + self.coeffs[6] * (c3 * (3.0 * z * z - 1.0));
        irr = irr + self.coeffs[7] * (c2 * x * z);
        irr = irr + self.coeffs[8] * (c4 * (x * x - y * y));

        Vec3::new(irr.x.max(0.0), irr.y.max(0.0), irr.z.max(0.0))
    }
}

/// Surface material parameters for PBR evaluation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PbrMaterialParams {
    pub albedo: Color,
    pub roughness: f32,
    pub metallic: f32,
}

impl PbrMaterialParams {
    /// Creates PBR surface parameters.
    #[must_use]
    pub const fn new(albedo: Color, roughness: f32, metallic: f32) -> Self {
        Self {
            albedo,
            roughness,
            metallic,
        }
    }
}

/// Evaluates Cook-Torrance Metallic-Roughness PBR shading for a single surface fragment.
#[must_use]
pub fn pbr_shade_fragment(
    normal: Vec3,
    view_dir: Vec3,
    light_dir: Vec3,
    light_color: Vec3,
    mat: PbrMaterialParams,
    sh: &SphericalHarmonics9,
) -> (u8, u8, u8) {
    let normal_n = normal.normalize();
    let view_v = view_dir.normalize();
    let light_l = light_dir.normalize();
    let half_vec = (view_v + light_l).normalize();

    let ndotv = normal_n.dot(view_v).max(1e-4);
    let ndotl = normal_n.dot(light_l).max(0.0);
    let ndoth = normal_n.dot(half_vec).max(0.0);
    let vdoth = view_v.dot(half_vec).max(0.0);

    let albedo_vec = Vec3::new(
        f32::from(mat.albedo.r) / 255.0,
        f32::from(mat.albedo.g) / 255.0,
        f32::from(mat.albedo.b) / 255.0,
    );

    // Dielectric base reflectivity F0 = 0.04 (plastic/wood), pure metal F0 = albedo
    let f0 = Vec3::new(0.04, 0.04, 0.04) * (1.0 - mat.metallic) + albedo_vec * mat.metallic;

    // 1. Fresnel-Schlick: F(v, h) = F0 + (1 - F0) * (1 - v.h)^5
    let f_term = f0 + (Vec3::new(1.0, 1.0, 1.0) - f0) * (1.0 - vdoth).powi(5);

    // 2. Normal Distribution: GGX / Trowbridge-Reitz
    let alpha = (mat.roughness * mat.roughness).max(0.001);
    let alpha2 = alpha * alpha;
    let denom_d = (ndoth * ndoth * (alpha2 - 1.0) + 1.0).max(1e-5);
    let d_term = alpha2 / (std::f32::consts::PI * denom_d * denom_d);

    // 3. Geometry Masking: Smith Schlick-GGX
    let k = ((mat.roughness + 1.0) * (mat.roughness + 1.0)) / 8.0;
    let g_v = ndotv / (ndotv * (1.0 - k) + k);
    let g_l = ndotl / (ndotl * (1.0 - k) + k);
    let g_term = g_v * g_l;

    // Specular Cook-Torrance BRDF: (D * F * G) / (4 * ndotv * ndotl)
    let spec_numerator = f_term * (d_term * g_term);
    let spec_denom = (4.0 * ndotv * ndotl).max(1e-4);
    let specular_brdf = spec_numerator * (1.0 / spec_denom);

    // Diffuse Lambertian: (1 - F) * (1 - metallic) * (albedo / PI)
    let kd = (Vec3::new(1.0, 1.0, 1.0) - f_term) * (1.0 - mat.metallic);
    let diffuse_brdf = albedo_vec * (1.0 / std::f32::consts::PI);

    // Direct lighting contribution
    let direct = (kd * diffuse_brdf + specular_brdf) * light_color * ndotl;

    // Ambient diffuse irradiance from 9-coefficient Spherical Harmonics
    let ambient_irr = sh.eval(normal_n);
    let ambient = kd * albedo_vec * ambient_irr;

    // Tone mapping and clamp
    let total = direct + ambient;
    let mapped_r = (total.x / (total.x + 1.0) * 255.0).clamp(0.0, 255.0);
    let mapped_g = (total.y / (total.y + 1.0) * 255.0).clamp(0.0, 255.0);
    let mapped_b = (total.z / (total.z + 1.0) * 255.0).clamp(0.0, 255.0);

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    (mapped_r as u8, mapped_g as u8, mapped_b as u8)
}
