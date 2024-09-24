mod camera;
mod material;
mod object;
mod scene;
mod texture;
mod vec3;

use minifb::{Key, Window, WindowOptions};
use scene::Scene;
use vec3::Vec3;
use camera::Camera;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn main() {
    let mut window = Window::new(
        "Diorama",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut scene = Scene::new();

    // Crear la cámara
    let mut camera = Camera::new(
        Vec3::new(0.0, 5.0, 10.0),  // posición
        Vec3::new(0.0, 0.0, 0.0),   // objetivo
        Vec3::new(0.0, 1.0, 0.0),   // arriba
        45.0,                       // campo de visión
        WIDTH as f32 / HEIGHT as f32, // relación de aspecto
        0.1,                        // plano cercano
        100.0,                      // plano lejano
    );

    // Materiales
    let grass_material = material::Material::new(Vec3::new(0.1, 0.8, 0.1), 0.5, 0.0, 0.3, Some("textures/grass.png"));
    let dirt_material = material::Material::new(Vec3::new(0.6, 0.4, 0.2), 0.5, 0.0, 0.1, Some("textures/dirt.png"));

    // Crear el plano de grama y tierra
    let size = 10;
    for x in -size..=size {
        for z in -size..=size {
            let material = if (x + z) % 2 == 0 { &grass_material } else { &dirt_material };
            scene.add_object(object::Object::Cube(object::Cube::new(
                Vec3::new(x as f32, 0.0, z as f32),
                Vec3::new((x + 1) as f32, 1.0, (z + 1) as f32),
                material.clone(),
            )));
        }
    }

    // Luces
    scene.add_light(scene::Light::new(
        Vec3::new(0.0, 50.0, -10.0),  // posición
        Vec3::new(1.0, 1.0, 0.9),     // color (luz del sol)
    ));

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        scene.render(WIDTH as u32, HEIGHT as u32, &mut buffer);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}