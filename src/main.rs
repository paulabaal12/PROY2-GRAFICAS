mod framebuffer;
mod ray_intersect;
mod color;
mod camera;
mod light;
mod material;
mod cube;
mod texture; 

use minifb::{Window, WindowOptions, Key};
use image::open;
use nalgebra_glm::{Vec3, normalize};
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

const SKYBOX_COLOR_CELESTE: Color = Color::new(197, 237, 248);
const SKYBOX_COLOR_AZUL_OSCURO: Color = Color::new(14, 21, 101);
static mut SKYBOX_COLOR: Color = Color::new(68, 142, 228);

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
    skybox_color: &Color,
) -> Color {
    if depth > 3 {
        return *skybox_color;
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
        return *skybox_color;
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
        reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, light, depth + 1, skybox_color);
    }

    let mut refract_color = Color::black();
    let transparency = intersect.material.albedo[3];
    if transparency > 0.0 {
        let refract_dir = refract(&ray_direction, &intersect.normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_dir);
        refract_color = cast_ray(&refract_origin, &refract_dir, objects, light, depth + 1, skybox_color);
    }

    specular * (1.0 - reflectivity - transparency) + (reflect_color * reflectivity) + (refract_color * transparency);
    let u = intersect.u.unwrap_or(0.0);
    let v = intersect.v.unwrap_or(0.0);
    let texture_color = intersect.material.get_texture_color(u, v);

    let diffuse_intensity = intersect.normal.dot(&(-ray_direction)).max(0.0).min(1.0);
    let light_color = Color::new(texture_color[0], texture_color[1], texture_color[2]) * diffuse_intensity;
    light_color
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Box<dyn RayIntersect>], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    unsafe {
        let skybox_color = SKYBOX_COLOR;
        
        for y in 0..framebuffer.height {
            for x in 0..framebuffer.width {
                let screen_x = (2.0 * x as f32) / width - 1.0;
                let screen_y = -(2.0 * y as f32) / height + 1.0;

                let screen_x = screen_x * aspect_ratio * perspective_scale;
                let screen_y = screen_y * perspective_scale;

                let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));

                let rotated_direction = camera.base_change(&ray_direction);

                let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, 0, &skybox_color);

                framebuffer.set_current_color(pixel_color.to_hex());
                framebuffer.point(x, y);
            }
        }
    }
}


fn main() {
    let window_width = 600;
    let window_height = 400;
    let framebuffer_width = 600;
    let framebuffer_height = 400;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

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




    let mut camera = Camera::new(
        Vec3::new(0.0, 5.0, 15.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let light = Light::new(
        Vec3::new(10.0, 15.0, 10.0),
        [255, 255, 255],
        1.0,
    );

    let rotation_speed = PI / 10.0;

    let objects: Vec<Box<dyn RayIntersect>> = vec![
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

    ];

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

        if window.is_key_down(Key::D) {
            unsafe { SKYBOX_COLOR = SKYBOX_COLOR_CELESTE; }
        } 

        if window.is_key_down(Key::N) {
            unsafe { SKYBOX_COLOR = SKYBOX_COLOR_AZUL_OSCURO; }
        } 


        render(&mut framebuffer, &objects, &camera, &light);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}