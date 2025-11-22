//! Shadow Mapping System
//!
//! Provides shadow texture management, cascade frustum calculation, and shadow map rendering.
//! Phase 4: Advanced rendering features

use glam::{Mat4, Vec3, Vec4};
use thiserror::Error;

/// Shadow texture configuration
#[derive(Debug, Clone)]
pub struct ShadowMapConfig {
    /// Shadow map resolution (width and height)
    pub resolution: u32,
    /// Number of cascades for directional lights (1-4)
    pub cascade_count: u32,
    /// PCF (Percentage Closer Filtering) sample count
    pub pcf_samples: u32,
    /// Shadow bias to prevent shadow acne
    pub bias: f32,
    /// Normal offset bias
    pub normal_bias: f32,
}

impl Default for ShadowMapConfig {
    fn default() -> Self {
        Self {
            resolution: 2048,
            cascade_count: 4,
            pcf_samples: 4,
            bias: 0.005,
            normal_bias: 0.05,
        }
    }
}

/// Shadow map errors
#[derive(Error, Debug)]
pub enum ShadowError {
    #[error("Invalid cascade count: {0} (must be 1-4)")]
    InvalidCascadeCount(u32),

    #[error("Invalid shadow map resolution: {0} (must be power of 2, 256-4096)")]
    InvalidResolution(u32),

    #[error("Invalid PCF sample count: {0} (must be 1, 4, 9, or 16)")]
    InvalidPCFSamples(u32),
}

/// Cascaded shadow map (CSM) splits
#[derive(Debug, Clone)]
pub struct CascadeSplits {
    /// Split distances in view space (cascade_count + 1 values)
    pub distances: Vec<f32>,
    /// Shadow matrices for each cascade
    pub matrices: Vec<Mat4>,
}

impl CascadeSplits {
    /// Calculate cascade splits using practical split scheme
    ///
    /// # Arguments
    /// * `near` - Camera near plane
    /// * `far` - Camera far plane
    /// * `cascade_count` - Number of cascades (1-4)
    /// * `lambda` - Split scheme parameter (0.0 = uniform, 1.0 = logarithmic, 0.5 = practical)
    ///
    /// # Returns
    /// Cascade split distances in view space
    pub fn calculate_splits(near: f32, far: f32, cascade_count: u32, lambda: f32) -> Vec<f32> {
        let mut splits = vec![near];

        for i in 1..=cascade_count {
            let i_f = i as f32;
            let count_f = cascade_count as f32;

            // Logarithmic split
            let log_split = near * (far / near).powf(i_f / count_f);

            // Uniform split
            let uniform_split = near + (far - near) * (i_f / count_f);

            // Practical split (blend of log and uniform)
            let split = lambda * log_split + (1.0 - lambda) * uniform_split;

            splits.push(split);
        }

        splits
    }

    /// Create new cascade splits from distances
    pub fn new(distances: Vec<f32>) -> Self {
        let cascade_count = (distances.len() - 1) as u32;
        Self {
            distances,
            matrices: vec![Mat4::IDENTITY; cascade_count as usize],
        }
    }

    /// Get cascade index for a given view-space depth
    pub fn get_cascade_index(&self, view_depth: f32) -> usize {
        for (i, window) in self.distances.windows(2).enumerate() {
            if view_depth >= window[0] && view_depth < window[1] {
                return i;
            }
        }
        // Beyond far plane - use last cascade
        (self.distances.len() - 2).max(0)
    }
}

/// Calculate directional light shadow matrix for a cascade
///
/// # Arguments
/// * `light_direction` - Light direction (normalized)
/// * `view_matrix` - Camera view matrix
/// * `projection_matrix` - Camera projection matrix
/// * `near_distance` - Cascade near distance
/// * `far_distance` - Cascade far distance
///
/// # Returns
/// Shadow matrix (light space projection)
pub fn calculate_directional_shadow_matrix(
    light_direction: Vec3,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    near_distance: f32,
    far_distance: f32,
) -> Mat4 {
    // Get frustum corners in world space
    let frustum_corners = calculate_frustum_corners_world_space(
        view_matrix,
        projection_matrix,
        near_distance,
        far_distance,
    );

    // Calculate frustum center
    let mut center = Vec3::ZERO;
    for corner in &frustum_corners {
        center += *corner;
    }
    center /= frustum_corners.len() as f32;

    // Create light view matrix (look at frustum center from light direction)
    let light_view = Mat4::look_at_rh(
        center - light_direction * 10.0, // Position light far back
        center,
        Vec3::Y, // Up vector
    );

    // Calculate AABB of frustum in light space
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);

    for corner in &frustum_corners {
        let light_space = light_view.transform_point3(*corner);
        min = min.min(light_space);
        max = max.max(light_space);
    }

    // Create orthographic projection for light (directional lights use ortho)
    let light_projection = Mat4::orthographic_rh(
        min.x, max.x, // left, right
        min.y, max.y, // bottom, top
        min.z, max.z, // near, far
    );

    light_projection * light_view
}

/// Calculate frustum corners in world space
fn calculate_frustum_corners_world_space(
    view_matrix: Mat4,
    projection_matrix: Mat4,
    near_distance: f32,
    far_distance: f32,
) -> Vec<Vec3> {
    let inv_view_proj = (projection_matrix * view_matrix).inverse();

    let mut corners = Vec::with_capacity(8);

    // NDC coordinates of frustum corners
    let ndc_corners = [
        // Near plane
        Vec4::new(-1.0, -1.0, -1.0, 1.0), // Bottom-left
        Vec4::new(1.0, -1.0, -1.0, 1.0),  // Bottom-right
        Vec4::new(1.0, 1.0, -1.0, 1.0),   // Top-right
        Vec4::new(-1.0, 1.0, -1.0, 1.0),  // Top-left
        // Far plane
        Vec4::new(-1.0, -1.0, 1.0, 1.0), // Bottom-left
        Vec4::new(1.0, -1.0, 1.0, 1.0),  // Bottom-right
        Vec4::new(1.0, 1.0, 1.0, 1.0),   // Top-right
        Vec4::new(-1.0, 1.0, 1.0, 1.0),  // Top-left
    ];

    for ndc in ndc_corners {
        let world_pos = inv_view_proj * ndc;
        let world_pos = world_pos / world_pos.w; // Perspective divide
        corners.push(world_pos.truncate());
    }

    corners
}

/// Calculate point light shadow matrices (6 faces for cubemap)
///
/// # Arguments
/// * `light_position` - Point light world position
/// * `near` - Shadow near plane
/// * `far` - Shadow far plane (light radius)
///
/// # Returns
/// 6 shadow matrices (one per cubemap face: +X, -X, +Y, -Y, +Z, -Z)
pub fn calculate_point_shadow_matrices(
    light_position: Vec3,
    near: f32,
    far: f32,
) -> Vec<Mat4> {
    let projection = Mat4::perspective_rh(
        std::f32::consts::FRAC_PI_2, // 90 degree FOV for each face
        1.0,                         // Aspect ratio 1:1 (square faces)
        near,
        far,
    );

    // Cubemap face directions and up vectors
    let faces = [
        (Vec3::X, Vec3::NEG_Y),   // +X face
        (Vec3::NEG_X, Vec3::NEG_Y), // -X face
        (Vec3::Y, Vec3::Z),       // +Y face
        (Vec3::NEG_Y, Vec3::NEG_Z), // -Y face
        (Vec3::Z, Vec3::NEG_Y),   // +Z face
        (Vec3::NEG_Z, Vec3::NEG_Y), // -Z face
    ];

    faces
        .iter()
        .map(|(direction, up)| {
            let view = Mat4::look_at_rh(light_position, light_position + *direction, *up);
            projection * view
        })
        .collect()
}

/// Calculate spot light shadow matrix
///
/// # Arguments
/// * `light_position` - Spot light world position
/// * `light_direction` - Spot light direction (normalized)
/// * `cone_angle` - Outer cone angle in radians
/// * `near` - Shadow near plane
/// * `far` - Shadow far plane (light radius)
///
/// # Returns
/// Shadow matrix (light space projection)
pub fn calculate_spot_shadow_matrix(
    light_position: Vec3,
    light_direction: Vec3,
    cone_angle: f32,
    near: f32,
    far: f32,
) -> Mat4 {
    // Create perspective projection for spot light cone
    let fov = cone_angle * 2.0; // Full cone angle
    let projection = Mat4::perspective_rh(
        fov,
        1.0, // Aspect ratio 1:1
        near,
        far,
    );

    // Create view matrix looking down the light direction
    let view = Mat4::look_at_rh(
        light_position,
        light_position + light_direction,
        Vec3::Y, // Up vector
    );

    projection * view
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cascade_splits_practical() {
        let splits = CascadeSplits::calculate_splits(0.1, 100.0, 4, 0.5);
        assert_eq!(splits.len(), 5); // 4 cascades + 1 = 5 splits
        assert_eq!(splits[0], 0.1); // Near
        assert_eq!(splits[4], 100.0); // Far
        // Middle splits should be between near and far
        for i in 1..4 {
            assert!(splits[i] > splits[i - 1]);
            assert!(splits[i] < splits[i + 1]);
        }
    }

    #[test]
    fn test_cascade_splits_uniform() {
        let splits = CascadeSplits::calculate_splits(0.0, 100.0, 4, 0.0);
        // Uniform splits: 0, 25, 50, 75, 100
        assert!((splits[1] - 25.0).abs() < 0.01);
        assert!((splits[2] - 50.0).abs() < 0.01);
        assert!((splits[3] - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_cascade_index() {
        let splits = CascadeSplits::new(vec![0.1, 10.0, 30.0, 60.0, 100.0]);

        assert_eq!(splits.get_cascade_index(5.0), 0); // First cascade
        assert_eq!(splits.get_cascade_index(20.0), 1); // Second cascade
        assert_eq!(splits.get_cascade_index(45.0), 2); // Third cascade
        assert_eq!(splits.get_cascade_index(80.0), 3); // Fourth cascade
        assert_eq!(splits.get_cascade_index(150.0), 3); // Beyond far - last cascade
    }

    #[test]
    fn test_point_shadow_matrices() {
        let light_pos = Vec3::new(0.0, 5.0, 0.0);
        let matrices = calculate_point_shadow_matrices(light_pos, 0.1, 10.0);

        assert_eq!(matrices.len(), 6); // 6 cubemap faces
        // All matrices should be valid (non-zero)
        for matrix in matrices {
            assert!(matrix.determinant().abs() > 0.0);
        }
    }

    #[test]
    fn test_spot_shadow_matrix() {
        let light_pos = Vec3::new(0.0, 5.0, 0.0);
        let light_dir = Vec3::NEG_Y; // Pointing down
        let cone_angle = std::f32::consts::FRAC_PI_4; // 45 degrees
        let matrix = calculate_spot_shadow_matrix(light_pos, light_dir, cone_angle, 0.1, 10.0);

        // Matrix should be valid
        assert!(matrix.determinant().abs() > 0.0);
    }

    #[test]
    fn test_shadow_map_config_default() {
        let config = ShadowMapConfig::default();
        assert_eq!(config.resolution, 2048);
        assert_eq!(config.cascade_count, 4);
        assert_eq!(config.pcf_samples, 4);
        assert!(config.bias > 0.0);
    }

    #[test]
    fn test_directional_shadow_matrix() {
        let light_dir = Vec3::NEG_Y; // Light from above
        let view = Mat4::look_at_rh(Vec3::new(0.0, 5.0, 10.0), Vec3::ZERO, Vec3::Y);
        let projection = Mat4::perspective_rh(
            std::f32::consts::FRAC_PI_4,
            16.0 / 9.0,
            0.1,
            100.0,
        );

        let shadow_matrix =
            calculate_directional_shadow_matrix(light_dir, view, projection, 0.1, 30.0);

        // Shadow matrix should be valid
        assert!(shadow_matrix.determinant().abs() > 0.0);
    }
}
