use nalgebra_glm::Vec3;
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::material::Material;

pub struct RectangularPrism {
    pub center: Vec3,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub material: Material,
}

impl RayIntersect for RectangularPrism {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;
        let half_depth = self.depth / 2.0;

        let min = self.center - Vec3::new(half_width, half_height, half_depth);
        let max = self.center + Vec3::new(half_width, half_height, half_depth);
        
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
            return Intersect::empty(); // No hay intersección
        }

        let t = if t_near_val < 0.0 { t_far_val } else { t_near_val };
        let intersection_point = ray_origin + ray_direction * t;

        let normal = calculate_normal(&intersection_point, &self.center, self.width, self.height, self.depth);

        // Calcular u y v para mapeo de texturas
        let u = match normal.x {
            1.0 => (intersection_point.z - min.z) / self.depth, // Cara derecha
            -1.0 => (intersection_point.z - max.z) / self.depth, // Cara izquierda
            _ => (intersection_point.x - min.x) / self.width,   // Para las caras Y y Z
        };
        
        let v = match normal.y {
            1.0 => (intersection_point.z - min.z) / self.depth, // Cara superior
            -1.0 => (intersection_point.z - max.z) / self.depth, // Cara inferior
            _ => (intersection_point.y - min.y) / self.height,   // Para las caras X y Z
        };

        Intersect::new(intersection_point, normal, t, self.material.clone(), u, v) // Clonar material
    }
}

// Vector normal
fn calculate_normal(point: &Vec3, center: &Vec3, width: f32, height: f32, depth: f32) -> Vec3 {
    let half_width = width / 2.0;
    let half_height = height / 2.0;
    let half_depth = depth / 2.0;
    let x_diff = (point.x - center.x).abs() - half_width;
    let y_diff = (point.y - center.y).abs() - half_height;
    let z_diff = (point.z - center.z).abs() - half_depth;

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