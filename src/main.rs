use minifb::{Key, Window, WindowOptions};
use scene::Scene;
use vec3::Vec3;
use camera::Camera;
// use material::Material; // Removed unused import
use object::{Object, Cube};
use light::Light;
use texture::Texture;
use std::rc::Rc;

mod vec3;
mod ray;
mod camera;
mod material;
mod object;
mod scene;
mod light;
mod texture;
mod cube;

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
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Crear la cámara
    let mut camera = Camera::new(
        Vec3::new(0.0, 5.0, 20.0),  // posición ajustada
        Vec3::new(0.0, 0.0, 0.0),   // objetivo
        Vec3::new(0.0, 1.0, 0.0),   // arriba
        45.0,                       // campo de visión
        WIDTH as f32 / HEIGHT as f32, // relación de aspecto
        0.1,                        // plano cercano
        100.0,                      // plano lejano
    );

    let mut scene = Scene::new(camera.clone());

    // Texturas
    let grass_texture = Rc::new(Texture::load("textures/grass.png"));
    let grass_side_texture = Rc::new(Texture::load("textures/grass.png"));
    let dirt_texture = Rc::new(Texture::load("textures/dirt.png"));

    // Crear suelo voxelizado
    let voxel_size = 2.75;
    let ground = create_voxelized_cube(
        Vec3::new(-10.0, -2.75, -10.0),
        Vec3::new(10.0, 0.0, 10.0),
        grass_texture,
        grass_side_texture,
        dirt_texture,
        voxel_size,
    );

    println!("Número de cubos creados: {}", ground.len()); // Añadir mensaje de depuración

    for cube in ground {
        scene.add_object(Object::Cube(cube));
    }

    // Luz
    let light = Light::new(Vec3::new(0.0, 10.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
    scene.add_light(light);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Movimiento de la cámara
        const MOVE_SPEED: f32 = 0.5;
        let mut camera_moved = false;

        if window.is_key_down(Key::Left) {
            camera.position.x -= MOVE_SPEED;
            camera_moved = true;
        }
        if window.is_key_down(Key::Right) {
            camera.position.x += MOVE_SPEED;
            camera_moved = true;
        }
        if window.is_key_down(Key::Up) {
            camera.position.z -= MOVE_SPEED;
            camera_moved = true;
        }
        if window.is_key_down(Key::Down) {
            camera.position.z += MOVE_SPEED;
            camera_moved = true;
        }

        // Actualizar la cámara en la escena si se ha movido
        if camera_moved {
            scene.set_camera(camera.clone());
            println!("Posición de la cámara: {:?}", camera.position); // Añadir mensaje de depuración
        }

        // Renderizar la escena
        scene.render(WIDTH as u32, HEIGHT as u32, &mut buffer);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

pub fn create_voxelized_cube(
    min: Vec3,
    max: Vec3,
    top_texture: Rc<Texture>,
    side_texture: Rc<Texture>,
    bottom_texture: Rc<Texture>,
    voxel_size: f32,
) -> Vec<Cube> {
    let mut cubes = Vec::new();

    let x_steps = ((max.x - min.x) / voxel_size).ceil() as i32;
    let y_steps = ((max.y - min.y) / voxel_size).ceil() as i32;
    let z_steps = ((max.z - min.z) / voxel_size).ceil() as i32;

    for i in 0..x_steps {
        for j in 0..y_steps {
            for k in 0..z_steps {
                let cube_min = Vec3::new(
                    min.x + i as f32 * voxel_size,
                    min.y + j as f32 * voxel_size,
                    min.z + k as f32 * voxel_size,
                );

                let cube_max = Vec3::new(
                    (cube_min.x + voxel_size).min(max.x),
                    (cube_min.y + voxel_size).min(max.y),
                    (cube_min.z + voxel_size).min(max.z),
                );

                let cube = Cube::new_with_textures(
                    cube_min,
                    cube_max,
                    Rc::clone(&top_texture),
                    Rc::clone(&side_texture),
                    Rc::clone(&bottom_texture),
                );

                cubes.push(cube);
            }
        }
    }

    cubes
}
