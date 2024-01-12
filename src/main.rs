use std::ops::Mul;

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    math::vec3,
    prelude::*,
    window::PrimaryWindow,
};

const SQUARE_SIZE: f32 = 20.0;
const SQUARE_COLOR: Color = Color::rgb(0.25, 0.25, 0.75);
const SCROLL_SPEED: f32 = 0.1;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_mouse)
        .run();
}

fn setup(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    insert_cell(Transform::from_translation(Vec3::new(0., 0., 0.)), commands)
}

fn insert_cell(pos: Transform, mut commands: Commands) {
    // Rectangle
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: SQUARE_COLOR,
                custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                ..default()
            },
            transform: pos,
            ..default()
        })
        .insert(Cell);
}

// finds which cell has been clicked
fn handle_mouse(
    mouse_button: Res<Input<MouseButton>>,
    mut scroll_event: EventReader<MouseWheel>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut camera_projection: Query<&mut OrthographicProjection, With<Camera>>,
    mut cell_q: Query<(&mut Sprite, &Handle<Image>, &GlobalTransform, Entity), With<Cell>>,
) {
    let (camera, camera_transform) = camera_q.single();
    let primary_window = window_q.single();
    let scale_factor = primary_window.scale_factor() as f32;
    let mut active_entity = None;

    // find clicked cell
    if mouse_button.just_released(MouseButton::Left) {
        if let Some(pos) = primary_window.cursor_position() {
            if let Some(pos) = camera.viewport_to_world_2d(camera_transform, pos) {
                for (sprite, handle, node_transform, entity) in &mut cell_q.iter_mut() {
                    let size = sprite.custom_size.unwrap(); //sprite.rect.unwrap();

                    let x_min = node_transform.affine().translation.x - size.x / 2.; // + size.min.x;
                    let y_min = node_transform.affine().translation.y - size.y / 2.; // + size.min.y;
                    let x_max = node_transform.affine().translation.x + size.x / 2.; // + size.max.x;
                    let y_max = node_transform.affine().translation.y + size.y / 2.; // + size.max.y;

                    println!(
                        "entity is in rect ({}, {}), ({}, {})",
                        x_min, y_min, x_max, y_max
                    );

                    if x_min < pos.x && pos.x < x_max && y_min < pos.y && pos.y < y_max {
                        println!("found entity");
                        active_entity = Some((sprite, handle, node_transform, entity));
                    }
                }
            }
        }
    }

    // update color of clicked cell
    if let Some((mut sprite, _handle, _node_transform, _entity)) = active_entity {
        println!("active entity: {:#?}", sprite);
        sprite.color = Color::rgb(0.75, 0.75, 0.25);
    }

    // zoom using scrolling
    for event in scroll_event.read() {
        match event.unit {
            MouseScrollUnit::Line => {
                let mut projection = camera_projection.single_mut();
                projection.scale = (projection.scale - event.y * SCROLL_SPEED).clamp(0.1, 100.);
            }
            MouseScrollUnit::Pixel => {
                println!(
                    "NOT SUPPORTED Scroll (pixel units): vertical: {}, horizontal: {}",
                    event.y, event.x
                );
            }
        }
    }
}

#[derive(Component)]
pub struct Cell;

#[derive(Component)]
pub struct Bomb;
