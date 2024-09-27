mod framebuffer;
mod ray_intersect;
mod cube;
mod rectangular_prism;
mod color;
mod camera;
mod light;
mod material;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use nalgebra_glm::{Vec3, normalize};
use std::time::Duration;
use std::f32::consts::PI;

use crate::color::Color;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::cube::Cube;
use crate::rectangular_prism::RectangularPrism;
use crate::framebuffer::Framebuffer;
use crate::camera::Camera;
use crate::light::Light;
use crate::material::Material;

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

pub fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, cubes: &[Cube], rectangles: &[RectangularPrism], light: &Light) -> Color {
    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    // Combina la intersección de cubos y prismas rectangulares
    let objects = cubes.iter().map(|obj| obj as &dyn RayIntersect)
        .chain(rectangles.iter().map(|obj| obj as &dyn RayIntersect));

    for object in objects {
        let tmp = object.ray_intersect(ray_origin, ray_direction);
        if tmp.is_intersecting {
            // Early exit si la distancia es mayor que el zbuffer
            if tmp.distance < zbuffer {
                zbuffer = tmp.distance;
                intersect = tmp;
            } else {
                continue;
            }
        }
    }

    if !intersect.is_intersecting {
        return Color::new(9, 20, 55); // Color de fondo
    }

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal);

    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let mut diffuse = intersect.material.diffuse * intersect.material.albedo[0] * diffuse_intensity * light.intensity;

    // Manejo de texturas
    if let Some(texture) = intersect.material.texture.as_ref() {
        let u = intersect.u; 
        let v = intersect.v; 
        let texture_width = texture.width;
        let texture_height = texture.height;

        let texture_x = (u * texture_width as f32).clamp(0.0, (texture_width - 1) as f32) as usize;
        let texture_y = (v * texture_height as f32).clamp(0.0, (texture_height - 1) as f32) as usize;
        let texture_index = (texture_y * texture_width + texture_x) * 4;

        let pixel_color = &texture.data[texture_index..texture_index + 4];
        let tex_color = Color::new(pixel_color[0], pixel_color[1], pixel_color[2]);

        diffuse += tex_color * intersect.material.albedo[0] * diffuse_intensity * light.intensity;
    }

    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
    let specular = light.color * intersect.material.albedo[1] * specular_intensity * light.intensity;
    //luz
    let emission = intersect.material.emission * 1.8;

    diffuse + specular + emission
}

pub fn render(framebuffer: &mut Framebuffer, cubes: &[Cube], rectangles: &[RectangularPrism],camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
            let rotated_direction = camera.base_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, cubes, rectangles,light);

            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    let mut window = Window::new(
        "Diorama",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    let wood_texture = Material::load_texture("textures/wood.png");
    let wood = Material::new(
        Color::new(101, 62, 4),
        20.0,
        [0.6, 0.2],
        wood_texture,
        Color::new(0, 0, 0)
    );
    let grass_texture = Material::load_texture("textures/grass.png");
    let grass = Material::new(
        Color::new(29,	60,	14), 
        7.0, 
        [0.7, 0.1],
        grass_texture,
        Color::new(0, 0, 0)
    );
    let leaves_texture = Material::load_texture("textures/leaves.png");
    let leaves = Material::new(
        Color::new(29,	60,	14), 
        7.0, 
        [0.7, 0.1],
        leaves_texture,
        Color::new(0, 0, 0)
    );
    let wall_texture = Material::load_texture("textures/wall.png");
    let wall = Material::new(
        Color::new(206, 100, 0),
        15.0,
        [0.6, 0.3],
        wall_texture,
        Color::new(0, 0, 0)
    );
    let roof_texture = Material::load_texture("textures/roof.png");
    let roof = Material::new(
        Color::new(38,55,71),
        14.0,
        [0.6, 0.2],
        roof_texture,
        Color::new(0, 0, 0)
    );
    let water_texture = Material::load_texture("textures/water.png");
    let water = Material::new(
        Color::new(61, 133, 198),
        5.0,
        [0.7, 0.04],
        water_texture,
        Color::new(0, 0, 0)
    );


    let windows = Material::new(
        Color::new(253, 237, 191), 
        0.0, 
        [1.0, 0.0], // Solo emisión
        None,
        Color::new(253, 237, 191)* 2.0
    );
    //luna/sol
    let light_cube_texture = Material::load_texture("textures/moon.png");
    let light_cube = Cube {
        center: Vec3::new(0.0, 5.0, -5.0),
        side_length: 1.0,
        material: Material::new(
            Color::new(228, 246, 255)*1.5, 
            11.0, 
            [0.5, 0.5], 
            light_cube_texture,
            Color::new(228, 246, 255)* 1.5
        )
    };

    let cubes = [
        light_cube,
        //Arbol 1 hojas
        Cube {
            center: Vec3::new(1.7, 1.2, -3.2),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(1.0, 1.3, -2.8),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(0.8, 0.9, -3.4),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(0.5, 1.2, -3.5),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(1.2, 1.6, -3.3),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(1.0, 1.1, -3.8),
            side_length: 0.74,
            material: leaves.clone()
        },
        //Arbol 2 hojas
        Cube {
            center: Vec3::new(-3.3, 0.8, -4.3),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(-3.0, 1.5, -4.0),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(-2.9, 1.2, -4.2),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(-2.6, 1.1, -3.61),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(-3.4, 1.0, -3.7),
            side_length: 0.74,
            material: leaves.clone()
        },
        //Arbol 3 hojas
        Cube {
            center: Vec3::new(-1.2, 0.7, -1.8),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(-1.3, 1.3, -2.2),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(-1.92, 1.1, -2.2),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(-1.0, 1.0, -2.75),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(-1.7, 0.9, -2.4),
            side_length: 0.74,
            material: leaves.clone()
        },
        //arbol 4 hojas
        Cube {
            center: Vec3::new(-1.0, 0.7, -5.8),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(-1.0, 1.7, -6.0),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(-1.5, 1.4, -6.1),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(-0.5, 1.2, -6.2),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(-1.3, 1.1, -5.9),
            side_length: 0.74,
            material: leaves.clone()
        },
        Cube {
            center: Vec3::new(-0.3, 1.0, -5.95),
            side_length: 0.74,
            material: leaves.clone()
        },
        //techo orilla frente
        Cube {
            center: Vec3::new(3.5, 0.45, 2.3),
            side_length: 0.4,
            material: wood.clone()
        },
        Cube {
            center: Vec3::new(3.9, 0.6, 2.3),
            side_length: 0.4,
            material: wood.clone()
        },
        Cube {
            center: Vec3::new(4.3, 0.7, 2.3),
            side_length: 0.4,
            material: wood.clone()
        },
        Cube {
            center: Vec3::new(4.7, 0.6, 2.3),
            side_length: 0.4,
            material: wood.clone()
        },
        Cube {
            center: Vec3::new(5.1, 0.45, 2.3),
            side_length: 0.4,
            material: wood.clone()
        },
        //techo orilla atras
        Cube {
            center: Vec3::new(3.5, 0.45, -1.7),
            side_length: 0.4,
            material: wood.clone()
        },
        Cube {
            center: Vec3::new(3.9, 0.6, -1.7),
            side_length: 0.4,
            material: wood.clone()
        },
        Cube {
            center: Vec3::new(4.3, 0.7, -1.7),
            side_length: 0.4,
            material: wood.clone()
        },
        Cube {
            center: Vec3::new(4.7, 0.6, -1.7),
            side_length: 0.4,
            material: wood.clone()
        },
        Cube {
            center: Vec3::new(5.1, 0.45, -1.7),
            side_length: 0.4,
            material: wood.clone()
        }
    ];
    let rectangles =[
        //base
        RectangularPrism{
            center: Vec3::new(1.0, -0.9, -2.0),
            width: 9.0,
            height: 0.3,
            depth: 9.0,
            material: grass.clone()
        },
        //arboles (troncos)
        RectangularPrism{
            center: Vec3::new(-1.0, 0.3, -6.0),
            width: 0.6,
            height: 3.0,
            depth: 0.6,
            material: wood.clone()
        },RectangularPrism{
            center: Vec3::new(-3.0, 0.0, -4.0),
            width: 0.6,
            height: 2.0,
            depth: 0.6,
            material: wood.clone()
        },
        RectangularPrism{
            center: Vec3::new(-1.5, 0.2,-2.4),
            width: 0.6,
            height: 2.5,
            depth: 0.6,
            material: wood.clone()
        },
        RectangularPrism{
            center: Vec3::new(1.0, 0.0,-3.3),
            width: 0.6,
            height: 2.0,
            depth: 0.6,
            material: wood.clone()
        },
        //casa
        RectangularPrism{
            center: Vec3::new(4.3, -0.1,0.3),
            width: 1.8,
            height: 1.3,
            depth: 4.0,
            material: wall.clone()
        },
        //ventanas
        RectangularPrism {
            center: Vec3::new(3.35, 0.13, -0.9),
            width: 0.04,
            height: 0.45,
            depth: 0.5,
            material: windows.clone()
        },
        RectangularPrism {
            center: Vec3::new(3.35, 0.13,1.5),
            width: 0.04,
            height: 0.45,
            depth: 0.5,
            material: windows.clone()
        },
        RectangularPrism {
            center: Vec3::new(4.3, 0.15,2.4),
            width: 0.4,
            height: 0.4,
            depth: 0.04,
            material: windows.clone()
        },
        //puerta
        RectangularPrism {
            center: Vec3::new(3.39, -0.3,0.4),
            width: 0.03,
            height: 0.9,
            depth: 0.5,
            material: wood.clone()
        },        
        //techo placa
        RectangularPrism{
            center: Vec3::new(4.3, 0.49,0.3),
            width: 1.95,
            height: 0.1,
            depth: 4.0,
            material: roof.clone()
        },
        RectangularPrism{
            center: Vec3::new(4.3, 0.57,0.3),
            width: 1.8,
            height: 0.1,
            depth: 4.0,
            material: roof.clone()
        },
        RectangularPrism{
            center: Vec3::new(4.3, 0.66,0.28),
            width: 1.65,
            height: 0.1,
            depth: 3.6,
            material: roof.clone()
        },
        RectangularPrism{
            center: Vec3::new(4.3, 0.75,0.28),
            width: 1.5,
            height: 0.1,
            depth: 3.6,
            material: roof.clone()
        },
        //lago
        RectangularPrism{
            center: Vec3::new(-0.9,-0.79,0.2),
            width: 2.0,
            height: 0.1,
            depth: 3.0,
            material: water.clone()
        },
        RectangularPrism{
            center: Vec3::new(-0.9,-0.79,0.2),
            width: 2.5,
            height: 0.1,
            depth: 2.6,
            material: water.clone()
        },
        RectangularPrism{
            center: Vec3::new(-0.9,-0.79,0.2),
            width: 3.0,
            height: 0.1,
            depth: 2.3,
            material: water.clone()
        },
        RectangularPrism{
            center: Vec3::new(-0.9,-0.79,0.2),
            width: 3.5,
            height: 0.1,
            depth: 1.9,
            material: water.clone()
        },
        //muelle
        RectangularPrism{
            center: Vec3::new(0.15,-0.7,0.2),
            width: 1.4,
            height: 0.1,
            depth: 0.8,
            material: wood.clone()
        },
        RectangularPrism{
            center: Vec3::new(-0.4,-0.65,-0.2),
            width: 0.2,
            height: 0.2,
            depth: 0.2,
            material: wood.clone()
        },
        RectangularPrism{
            center: Vec3::new(-0.4,-0.65,0.6),
            width: 0.2,
            height: 0.2,
            depth: 0.2,
            material: wood.clone()
        },
    ];

    let mut camera = Camera::new(
        Vec3::new(-1.0, 1.0, 9.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    //día y noche
    let mut light = Light::new(
        Vec3::new(0.0, 5.1, 0.1),
        Color::new(255 ,236,183),
        1.7,
    );
    let new_light_intensity = 0.2;
    let mut light_on = false;

    let rotation_speed = PI / 10.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, rotation_speed);
        }
        if window.is_key_down(Key::W) {
            camera.adjust_zoom(0.9);
        }
        if window.is_key_down(Key::S) {
            camera.adjust_zoom(1.1);
        }

        if window.is_key_pressed(Key::L, KeyRepeat::No) {
            if light_on {
                //Día
                light.intensity = 1.7;
            } else {
                //Noche
                light.intensity = new_light_intensity;
            }
            light_on = !light_on;
        }

        render(&mut framebuffer, &cubes, &rectangles,&camera, &light);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}