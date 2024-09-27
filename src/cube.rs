use nalgebra_glm::Vec3;
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::material::Material;

pub struct Cube {
    pub center: Vec3,
    pub side_length: f32,
    pub material: Material,
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let half_size = self.side_length / 2.0;
        let min = self.center - Vec3::new(half_size, half_size, half_size);
        let max = self.center + Vec3::new(half_size, half_size, half_size);
        
        let t_min = (min - ray_origin).component_div(ray_direction);
        let t_max = (max - ray_origin).component_div(ray_direction);

        let t_near = Vec3::new(
            t_min.x.min(t_max.x),
            t_min.y.min(t_max.y),
            t_min.z.min(t_max.z),
        );
        let t_far = Vec3::new(
            t_min.x.max(t_max.x),
            t_min.y.max(t_max.y),
            t_min.z.max(t_max.z),
        );

        let t_near_val = t_near.x.max(t_near.y).max(t_near.z);
        let t_far_val = t_far.x.min(t_far.y).min(t_far.z);

        if t_near_val > t_far_val || t_far_val < 0.0 {
            return Intersect::empty(); // No intersección
        }

        let t = if t_near_val < 0.0 { t_far_val } else { t_near_val };
        let intersection_point = ray_origin + ray_direction * t;

        let normal = calculate_normal(&intersection_point, &self.center, self.side_length);
        let u = match normal.x {
            1.0 => (intersection_point.z - min.z) / self.side_length, // Cara derecha
            -1.0 => (intersection_point.z - max.z) / self.side_length, // Cara izquierda
            _ => (intersection_point.x - min.x) / self.side_length, // Para las caras Y y Z
        };
        
        let v = match normal.y {
            1.0 => (intersection_point.z - min.z) / self.side_length, // Cara superior
            -1.0 => (intersection_point.z - max.z) / self.side_length, // Cara inferior
            _ => (intersection_point.y - min.y) / self.side_length, // Para las caras X y Z
        };
        Intersect::new(intersection_point, normal, t, self.material.clone(), u, v) // Clonar material
    }
}

// Vector normal
fn calculate_normal(point: &Vec3, center: &Vec3, side_length: f32) -> Vec3 {
    let half_side = side_length / 2.0;
    let x_diff = (point.x - center.x).abs() - half_side;
    let y_diff = (point.y - center.y).abs() - half_side;
    let z_diff = (point.z - center.z).abs() - half_side;

    // Cara más cercana
    if x_diff >= y_diff && x_diff >= z_diff {
        if point.x > center.x {
            return Vec3::new(1.0, 0.0, 0.0); // Cara derecha
        } else {
            return Vec3::new(-1.0, 0.0, 0.0); // Cara izquierda
        }
    } else if y_diff >= x_diff && y_diff >= z_diff {
        if point.y > center.y {
            return Vec3::new(0.0, 1.0, 0.0); // Cara superior
        } else {
            return Vec3::new(0.0, -1.0, 0.0); // Cara inferior
        }
    } else {
        if point.z > center.z {
            return Vec3::new(0.0, 0.0, 1.0); // Cara frontal
        } else {
            return Vec3::new(0.0, 0.0, -1.0); // Cara trasera
        }
    }
}