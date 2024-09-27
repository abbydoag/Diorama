use crate::color::Color;
use image::GenericImageView;

#[derive(Debug, Clone)]
pub struct Texture {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 2],
    pub texture: Option<Texture>,
    pub emission: Color
}

impl Material {
    pub fn new(diffuse: Color, specular: f32, albedo: [f32; 2], texture: Option<Texture>, emission: Color) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            texture,
            emission
        }
    }

    pub fn black() -> Self {
        Material {
            diffuse: Color::new(0, 0, 0),
            specular: 0.0,
            albedo: [0.0, 0.0],
            texture: None,
            emission: Color::new(0, 0, 0) //aun no tiene emisison
        }
    }

    //Cargar textura
    pub fn load_texture(path: &str) -> Option<Texture> {
        match image::open(path) {
            Ok(img) => {
                let (width, height) = img.dimensions();
                let img = img.to_rgba8();
                let data = img.into_raw();
                Some(Texture {
                    data,
                    width: width as usize,
                    height: height as usize,
                })
            },
            Err(e) => {
                println!("Error al cargar la textura {}: {:?}", path, e);
                None
            }
        }
    }    
}