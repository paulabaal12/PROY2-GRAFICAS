use minifb::{Key, Window, WindowOptions};
use scene::Scene;
use vec3::Vec3;
use camera::Camera;
use material::Material;
use object::{Object, Cube};
use light::Light;

mod vec3;
mod ray;
mod camera;
mod material;
mod object;
mod scene;
mod light;
mod texture;

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

    let mut scene = Scene::new(camera);

    // Materiales
    let grass_material = Material::new(Vec3::new(0.1, 0.8, 0.1), 0.5, 0.0, 0.0, None);
    let dirt_material = Material::new(Vec3::new(0.6, 0.4, 0.2), 0.5, 0.0, 0.0, None);

    // Cubos
    let grass_cube = Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 0.0, 1.0), grass_material);
    let dirt_cube = Cube::new(Vec3::new(-1.0, -2.0, -1.0), Vec3::new(1.0, -1.0, 1.0), dirt_material);

    scene.add_object(Object::Cube(grass_cube));
    scene.add_object(Object::Cube(dirt_cube));

    // Luz
    let light = Light::new(Vec3::new(0.0, 10.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
    scene.add_light(light);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        scene.render(WIDTH as u32, HEIGHT as u32, &mut buffer);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
