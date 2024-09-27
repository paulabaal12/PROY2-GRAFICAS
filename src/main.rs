mod framebuffer;
mod ray_intersect;
mod color;
mod camera;
mod light;
mod material;
mod cube;
mod texture; 

use minifb::{Window, WindowOptions, Key};
use image::{open, DynamicImage, GenericImageView};
use nalgebra_glm::{Vec3, normalize};
use std::sync::Arc;
use std::time::Duration;
use std::f32::consts::PI;
use std::rc::Rc;

use crate::color::Color;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::framebuffer::Framebuffer;
use crate::camera::Camera;
use crate::light::Light;
use crate::cube::Cube;
use crate::material::Material;
use crate::texture::Texture; 


const ORIGIN_BIAS: f32 = 1e-4;

pub struct Skybox {
    front: Arc<DynamicImage>,
    back: Arc<DynamicImage>,
    left: Arc<DynamicImage>,
    right: Arc<DynamicImage>,
    top: Arc<DynamicImage>,
    bottom: Arc<DynamicImage>,
}

impl Skybox {
    fn new(
        front_path: &str,
        back_path: &str,
        left_path: &str,
        right_path: &str,
        top_path: &str,
        bottom_path: &str,
    ) -> Self {
        Skybox {
            front: Arc::new(image::open(front_path).expect("Failed to load front skybox image")),
            back: Arc::new(image::open(back_path).expect("Failed to load back skybox image")),
            left: Arc::new(image::open(left_path).expect("Failed to load left skybox image")),
            right: Arc::new(image::open(right_path).expect("Failed to load right skybox image")),
            top: Arc::new(image::open(top_path).expect("Failed to load top skybox image")),
            bottom: Arc::new(image::open(bottom_path).expect("Failed to load bottom skybox image")),
        }
    }

    fn get_color(&self, direction: &Vec3) -> Color {
        let abs_x = direction.x.abs();
        let abs_y = direction.y.abs();
        let abs_z = direction.z.abs();

        let (u, v, image) = if abs_x >= abs_y && abs_x >= abs_z {
            if direction.x > 0.0 {
                ((direction.z / abs_x + 1.0) / 2.0, (direction.y / abs_x + 1.0) / 2.0, &self.right)
            } else {
                ((-direction.z / abs_x + 1.0) / 2.0, (direction.y / abs_x + 1.0) / 2.0, &self.left)
            }
        } else if abs_y >= abs_x && abs_y >= abs_z {
            if direction.y > 0.0 {
                ((direction.x / abs_y + 1.0) / 2.0, (-direction.z / abs_y + 1.0) / 2.0, &self.top)
            } else {
                ((direction.x / abs_y + 1.0) / 2.0, (direction.z / abs_y + 1.0) / 2.0, &self.bottom)
            }
        } else {
            if direction.z > 0.0 {
                ((-direction.x / abs_z + 1.0) / 2.0, (direction.y / abs_z + 1.0) / 2.0, &self.front)
            } else {
                ((direction.x / abs_z + 1.0) / 2.0, (direction.y / abs_z + 1.0) / 2.0, &self.back)
            }
        };

        let (width, height) = image.dimensions();
        let pixel = image.get_pixel((u * width as f32) as u32, ((1.0 - v) * height as f32) as u32);
        Color::new(pixel[0], pixel[1], pixel[2])
    }
}

fn offset_origin(intersect: &Intersect, direction: &Vec3) -> Vec3 {
    let offset = intersect.normal * ORIGIN_BIAS;
    if direction.dot(&intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn refract(incident: &Vec3, normal: &Vec3, eta_t: f32) -> Vec3 {
    let cosi = -incident.dot(normal).max(-1.0).min(1.0);
    let (n_cosi, eta, n_normal);

    if cosi < 0.0 {
        n_cosi = -cosi;
        eta = 1.0 / eta_t;
        n_normal = -normal;
    } else {
        n_cosi = cosi;
        eta = eta_t;
        n_normal = *normal;
    }

    let k = 1.0 - eta * eta * (1.0 - n_cosi * n_cosi);

    if k < 0.0 {
        reflect(incident, &n_normal)
    } else {
        eta * incident + (eta * n_cosi - k.sqrt()) * n_normal
    }
}

//fresnel
fn fresnel(incident: &Vec3, normal: &Vec3, eta_i: f32, eta_t: f32) -> f32 {
    let cosi = incident.dot(normal).max(-1.0).min(1.0);
    let mut etai = eta_i;
    let mut etat = eta_t;

    let mut n_cosi = cosi;
    if cosi > 0.0 {
        etai = eta_t;
        etat = eta_i;
        n_cosi = cosi;
    } else {
        n_cosi = -cosi;
    }
    let sint = etai / etat * (1.0 - n_cosi * n_cosi).sqrt();

    if sint >= 1.0 {
        return 1.0;  
    }

    let cost = (1.0 - sint * sint).sqrt();
    let r_orth = ((etat * n_cosi) - (etai * cost)) / ((etat * n_cosi) + (etai * cost));
    let r_parl = ((etai * n_cosi) - (etat * cost)) / ((etai * n_cosi) + (etat * cost));
    (r_orth * r_orth + r_parl * r_parl) / 2.0
}


fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Box<dyn RayIntersect>],
) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let light_distance = (light.position - intersect.point).magnitude();

    let shadow_ray_origin = offset_origin(intersect, &light_dir);
    let mut shadow_intensity = 0.0;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            let distance_ratio = shadow_intersect.distance / light_distance;
            shadow_intensity = 1.0 - distance_ratio.powf(2.0).min(1.0);
            break;
        }
    }

    shadow_intensity
}

fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Box<dyn RayIntersect>],
    light: &Light,
    depth: u32,
    skybox: &Skybox,
) -> Color {
    if depth > 3 {
        return skybox.get_color(ray_direction);
    }

    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }
    if !intersect.is_intersecting {
        return skybox.get_color(ray_direction);
    }

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);
    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
    let specular = Color::new(light.color[0], light.color[1], light.color[2]) * intersect.material.albedo[1] * specular_intensity * light_intensity;

    let mut reflect_color = Color::black();
    let reflectivity = intersect.material.albedo[2];
    if reflectivity > 0.0 {
        let reflect_dir = reflect(&ray_direction, &intersect.normal).normalize();
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, light, depth + 1, skybox);
    }

    let mut refract_color = Color::black();
    let transparency = intersect.material.albedo[3];
    if transparency > 0.0 {
        let refract_dir = refract(&ray_direction, &intersect.normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_dir);
        refract_color = cast_ray(&refract_origin, &refract_dir, objects, light, depth + 1, skybox);
    }

    specular * (1.0 - reflectivity - transparency) + (reflect_color * reflectivity) + (refract_color * transparency);
    let u = intersect.u.unwrap_or(0.0);
    let v = intersect.v.unwrap_or(0.0);
    let texture_color = intersect.material.get_texture_color(u, v);

    let diffuse_intensity = intersect.normal.dot(&(-ray_direction)).max(0.0).min(1.0);
    let light_color = Color::new(texture_color[0], texture_color[1], texture_color[2]) * diffuse_intensity;
    light_color
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Box<dyn RayIntersect>], camera: &Camera, light: &Light, skybox: &Skybox) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();
    let inv_width = 1.0 / width;
    let inv_height = 1.0 / height;

    for y in 0..framebuffer.height {
        let screen_y = -(2.0 * y as f32 * inv_height) + 1.0;
        let scaled_y = screen_y * perspective_scale;

        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32 * inv_width) - 1.0;
            let scaled_x = screen_x * aspect_ratio * perspective_scale;

            let ray_direction = Vec3::new(scaled_x, scaled_y, -1.0).normalize();
            let rotated_direction = camera.base_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, 0, skybox);

            let index = y * framebuffer.width + x;
            framebuffer.buffer[index] = pixel_color.to_hex();
        }
    }
}

fn update_light(light: &mut Light, angle: f32, is_night: bool) {
    let radius = 15.0;
    light.position.x = radius * angle.cos();
    light.position.y = radius * angle.sin();

    if is_night {
        light.color = [50, 50, 100]; 
        light.intensity = -0.3;
    } else {
        light.color = [255, 255, 255];
        light.intensity = 3.0;
    }
}



fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let low_res_width = framebuffer_width / 2;
    let low_res_height = framebuffer_height / 2;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut low_res_framebuffer = Framebuffer::new(low_res_width, low_res_height);

    
    //día 
    let original_skybox = Skybox::new(
        "textures/skybox/right.png",
        "textures/skybox/right.png",
        "textures/skybox/right.png",
        "textures/skybox/right.png",
        "textures/skybox/right.png",
        "textures/skybox/right.png",
    );

    //simular noche
    let alternate_skybox = Skybox::new(
        "textures/skybox/altern.png",
        "textures/skybox/altern.png",
        "textures/skybox/altern.png",
        "textures/skybox/altern.png",
        "textures/skybox/altern.png",
        "textures/skybox/altern.png",
    );

    let mut current_skybox = &original_skybox;

    let mut window = Window::new(
        "ICEEE",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    let snow_texture = Rc::new(Texture::new("textures/snow.png"));
    let snow_texture2 = Rc::new(Texture::new("textures/snow1.png"));
    let door_texture = Rc::new(Texture::new("textures/door2.png"));
    let wall_texture = Rc::new(Texture::new("textures/paredd.png")); 
    let techo_texture = Rc::new(Texture::new("textures/madera.png")); 
    let tronco_texture = Rc::new(Texture::new("textures/tronco.png")); 
    let hoja_texture = Rc::new(Texture::new("textures/flor.png"));
    let piedra_texture = Rc::new(Texture::new("textures/piedra3.png"));

    let agua_textures = vec![
        Rc::new(Texture::new("textures/agua1.png")),
        Rc::new(Texture::new("textures/agua2.png")),
        Rc::new(Texture::new("textures/agua3.png")),
        Rc::new(Texture::new("textures/agua4.png")),
    ];

    let mut animation_frame = 2;


    let ground = Cube::new(
        Vec3::new(-5.0, -1.5, -5.0),
        Vec3::new(8.0, 0.6, 8.0),
        snow_texture.clone(),
        snow_texture.clone(),
        snow_texture2.clone(),
    );

    //Puerta
    let door = Cube::new(
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.5, 4.0, 3.0),  
        door_texture.clone(),
        door_texture.clone(),
        door_texture.clone(),
    );
    
    //wall1
    let wall1 = Cube::new(
        Vec3::new(3.0, 0.0, 0.0), 
        Vec3::new(1.0, 4.0, 3.0), 
        wall_texture.clone(),
        wall_texture.clone(),
        wall_texture.clone(),
    );
    

    let wall2 = Cube::new(
        Vec3::new(-1.0, 0.0, 0.0), 
        Vec3::new(5.0, 4.0, 3.0), 
        wall_texture.clone(),
        wall_texture.clone(),
        wall_texture.clone(),
    );
    

let back_left = Cube::new(
    Vec3::new(-1.0, 0.0, -1.5),      
    Vec3::new(5.0, 4.0, 0.1),         
    wall_texture.clone(),
    wall_texture.clone(),
    wall_texture.clone(),
);


let back_right = Cube::new(
    Vec3::new(3.0, 0.0, -1.5),       
    Vec3::new(5.0, 4.0, 0.1),         
    wall_texture.clone(),
    wall_texture.clone(),
    wall_texture.clone(),
);


// Tronco del árbol 
let tronco = Cube::new(
    Vec3::new(-4.0, -1.5, -0.5),  
    Vec3::new(-3.0, 3.5, 0.3),  
    tronco_texture.clone(),
    tronco_texture.clone(),
    tronco_texture.clone(),
);

// Hoja del árbol 
let hoja1 = Cube::new(
    Vec3::new(-5.0, 3.5, -0.5),    
    Vec3::new(-2.0, 4.5, 1.5),    
    hoja_texture.clone(),           
    hoja_texture.clone(),
    hoja_texture.clone(),
);

// Hoja del árbol
let hoja2 = Cube::new(
    Vec3::new(-4.5, 4.5, -0.5),    
    Vec3::new(-2.5, 5.5, 1.5),      
    hoja_texture.clone(),            
    hoja_texture.clone(),
    hoja_texture.clone(),
);

// Hoja del árbol
let hoja3 = Cube::new(
    Vec3::new(-4.75, 3.5, -1.5),    
    Vec3::new(-2.25, 4.5, 1.5),      
    hoja_texture.clone(),             
    hoja_texture.clone(),
    hoja_texture.clone(),
);

// Hoja del árbol 
let hoja4 = Cube::new(
    Vec3::new(-4.75, 4.5, 0.5),     
    Vec3::new(-2.25, 5.5, 1.5),      
    hoja_texture.clone(),             
    hoja_texture.clone(),
    hoja_texture.clone(),
);


// Techo de la casa
let techo1 = Cube::new(
    Vec3::new(-2.0, 5.0, -2.0),  
    Vec3::new(6.0, 4.0, 3.0),  
    techo_texture.clone(),
    techo_texture.clone(),
    techo_texture.clone(),
);

let techo2 = Cube::new(
    Vec3::new(-1.0, 5.5, -2.0),  
    Vec3::new(5.0, 4.0, 3.0),   
    techo_texture.clone(),
    techo_texture.clone(),
    techo_texture.clone(),
);

let techo3 = Cube::new(
    Vec3::new(0.0, 6.0, -2.0),  
    Vec3::new(4.0, 4.0, 3.0),   
    techo_texture.clone(),
    techo_texture.clone(),
    techo_texture.clone(),
);

// Tronco del árbol 2
let tronco2 = Cube::new(
    Vec3::new(5.5, -1.5, 6.0),   
    Vec3::new(6.5, 3.5, 6.5),    
    tronco_texture.clone(),
    tronco_texture.clone(),
    tronco_texture.clone(),
);

// Hoja del árbol 1 
let hoja1_2 = Cube::new(
    Vec3::new(4.3, 3.5, 6.0),   
    Vec3::new(6.8, 4.5, 7.0),   
    hoja_texture.clone(),           
    hoja_texture.clone(),
    hoja_texture.clone(),
);

// Hoja del árbol 
let hoja2_2 = Cube::new(
    Vec3::new(4.8, 4.5, 6.0),    
    Vec3::new(6.6, 5.0, 7.0),     
    hoja_texture.clone(),           
    hoja_texture.clone(),
    hoja_texture.clone(),
);

// Hoja del árbol 
let hoja3_2 = Cube::new(
    Vec3::new(4.5, 3.5, 5.8),   
    Vec3::new(6.5, 4.5, 6.5),     
    hoja_texture.clone(),           
    hoja_texture.clone(),
    hoja_texture.clone(),
);


let hoja4_2 = Cube::new(
    Vec3::new(4.8, 4.0, 6.5),   
    Vec3::new(6.6, 5.0, 7.5),     
    hoja_texture.clone(),           
    hoja_texture.clone(),
    hoja_texture.clone(),
);


let mut jacuzzi = Cube::new(
    Vec3::new(-4.0, -1.5, 4.5),  
    Vec3::new(1.0, 1.0, 6.5),    
    agua_textures[0].clone(),      
    agua_textures[0].clone(),
    piedra_texture.clone(),
);



    let mut camera = Camera::new(
        Vec3::new(0.0, 5.0, 15.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut light = Light::new(
        Vec3::new(10.0, 15.0, 10.0),
        [255, 255, 255],
        1.0,
    );

    let rotation_speed = PI / 10.0;
    let zoom_speed = 1.0;

    let mut objects: Vec<Box<dyn RayIntersect>> = vec![
        Box::new(ground),
        Box::new(door),
        Box::new(tronco),
        Box::new(hoja1),
        Box::new(hoja2),
        Box::new(hoja3),
        Box::new(hoja4),
        Box::new(wall1),
        Box::new(wall2),
        Box::new(back_left),
        Box::new(back_right),
        Box::new(techo1),
        Box::new(techo2),
        Box::new(techo3),
        Box::new(tronco2),
        Box::new(hoja1_2),
        Box::new(hoja2_2),
        Box::new(hoja3_2),
        Box::new(hoja4_2),
        Box::new(jacuzzi),

    ];

    let mut light_angle: f32 = 0.0;
    let light_angle_speed: f32 = PI / 180.0; 
    let mut is_night: bool = false;


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
         // Zoom de la cámara
        if window.is_key_down(Key::Y) {
            camera.zoom(-zoom_speed);  // alejar
        }
        if window.is_key_down(Key::X) {
            camera.zoom(zoom_speed);   // acercar
        }

        if window.is_key_down(Key::N) {
            is_night = true; //  noche
            current_skybox = &alternate_skybox; 
        }
    
       
        if window.is_key_down(Key::D) {
            is_night = false; //  día
            current_skybox = &original_skybox; 
        }
    
        // Actualiza el ángulo de la luz y la posición
        light_angle += light_angle_speed;
        if light_angle > 2.0 * PI {
            light_angle -= 2.0 * PI;
        }
        update_light(&mut light, light_angle, is_night);



        animation_frame = (animation_frame + 3) % agua_textures.len();

        jacuzzi = Cube::new(
            Vec3::new(-4.0, -1.5, 4.5), 
            Vec3::new(1.0, 1.2, 7.0),
            agua_textures[animation_frame].clone(),
            agua_textures[animation_frame].clone(),
            piedra_texture.clone(),
        );
        

        

        let len = objects.len();
        objects[len - 1] = Box::new(jacuzzi); 


        render(&mut framebuffer, &objects, &camera, &light, current_skybox);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}