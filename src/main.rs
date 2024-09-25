mod framebuffer;
mod ray_intersect;
mod color;
mod camera;
mod light;
mod material;
mod cube;
mod texture; // Add this line to include the texture module

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
use crate::texture::Texture; // Add this line to import the Texture type


const ORIGIN_BIAS: f32 = 1e-4;
const SKYBOX_COLOR: Color = Color::new(68, 142, 228);

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
) -> Color {
    if depth > 3 {
        return SKYBOX_COLOR;
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
        return SKYBOX_COLOR;
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
        reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, light, depth + 1);
    }


    let mut refract_color = Color::black();
    let transparency = intersect.material.albedo[3];
    if transparency > 0.0 {
        let refract_dir = refract(&ray_direction, &intersect.normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_dir);
        refract_color = cast_ray(&refract_origin, &refract_dir, objects, light, depth + 1);
    }

    (specular) * (1.0 - reflectivity - transparency) + (reflect_color * reflectivity) + (refract_color * transparency);
    let u = intersect.u.unwrap_or(0.0);
    let v = intersect.v.unwrap_or(0.0);
    let texture_color = intersect.material.get_texture_color(u, v);

    let diffuse_intensity = intersect.normal.dot(&(-ray_direction)).max(0.0).min(1.0);
    let light_color = Color::new(texture_color[0], texture_color[1], texture_color[2]) * diffuse_intensity;
    let u = intersect.u.unwrap_or(0.0);
    let v = intersect.v.unwrap_or(0.0);
    let texture_color = intersect.material.get_texture_color(u, v);
    light_color
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Box<dyn RayIntersect>], camera: &Camera, light: &Light) {
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

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, 0);

            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
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
    let snow_texture2 = Rc::new(Texture::new("textures/snoww.png"));
    let door_texture = Rc::new(Texture::new("textures/door.png"));
    let wall_texture = Rc::new(Texture::new("textures/wood.png")); // Textura para las paredes

    // Suelo más grueso
    let ground = Cube::new(
        Vec3::new(-5.0, -1.5, -5.0),
        Vec3::new(8.0, 0.6, 8.0),
        Material::new(
            [1.0, 0.0, 0.0, 0.0],
            [255, 255, 255],
            50.0,
            1.0,
            Some(snow_texture.clone())
        ),
        Material::new(
            [1.0, 0.0, 0.0, 0.0],
            [255, 255, 255],
            50.0,
            1.0,
            Some(snow_texture2.clone())
        ),
        Material::new(
            [1.0, 0.0, 0.0, 0.0],
            [255, 255, 255],
            50.0,
            1.0,
            Some(snow_texture2.clone())
        )
    );

    // Paredes de la cabaña (cubos que rodean la puerta)
    let wall_front_left = Cube::new(
        Vec3::new(-2.5, -1.0, -2.5),
        Vec3::new(1.0, 2.5, 0.2), // Pared izquierda
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(wall_texture.clone())),
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(wall_texture.clone())),
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(wall_texture.clone()))
    );

    let wall_front_right = Cube::new(
        Vec3::new(1.5, -1.0, -2.5),
        Vec3::new(1.0, 2.5, 0.2), // Pared derecha
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(wall_texture.clone())),
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(wall_texture.clone())),
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(wall_texture.clone()))
    );

    let wall_back = Cube::new(
        Vec3::new(-2.5, -1.0, 2.3),
        Vec3::new(5.0, 2.5, 0.2), // Pared trasera
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(wall_texture.clone())),
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(wall_texture.clone())),
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(wall_texture.clone()))
    );

    // Pared lateral izquierda
    let wall_left = Cube::new(
        Vec3::new(-2.5, -1.0, -2.5),
        Vec3::new(0.2, 2.5, 5.0),
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(wall_texture.clone())),
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(wall_texture.clone())),
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(wall_texture.clone()))
    );

    // Pared lateral derecha
    let door = Cube::new(
        Vec3::new(2.3, -1.0, -2.5),
        Vec3::new(0.2, 2.5, 5.0),
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(door_texture.clone())),
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(door_texture.clone())),
        Material::new([1.0, 0.0, 0.0, 0.0], [139, 69, 19], 10.0, 1.0, Some(door_texture.clone()))
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
        Box::new(wall_front_left),
        Box::new(wall_front_right),
        Box::new(wall_back),
        Box::new(wall_left),
        Box::new(door),
    
    ];

    // El resto del código main() se mantiene igual
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

        render(&mut framebuffer, &objects, &camera, &light);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}