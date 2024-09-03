use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

use std::collections::HashSet;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const SQUARE_SIZE: f32 = 20.0;
const SQUARE_COLOR: Color = Color::rgb(0.25, 0.25, 0.75);
const SCROLL_SPEED: f32 = 0.1;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_mouse)
        .init_resource::<MouseDragData>()
        .init_resource::<GenerationData>()
        .run();
}

fn setup(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<ColorMaterial>>,
    mut generation_data: ResMut<GenerationData>,
) {
    commands.spawn(Camera2dBundle::default());

    insert_cell([0, 0], &mut commands, &mut generation_data);
}

fn is_bomb(pos: [isize; 2]) -> bool {
    let mut hasher = DefaultHasher::new();
    pos.hash(&mut hasher);
    hasher.finish() % 5 == 1 // if == 0 [0,0] bomb
}

fn insert_cell(pos: [isize; 2], commands: &mut Commands, generation_data: &mut GenerationData) {
    let float_pos = Vec3::new(pos[0] as f32, pos[1] as f32, 0.);
    // Rectangle
    let mut cell = commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: SQUARE_COLOR,
            custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
            ..default()
        },
        transform: Transform::from_translation(float_pos * (SQUARE_SIZE + 1.)),
        ..default()
    });
    cell.insert(Cell);

    if is_bomb(pos) {
        cell.insert(Bomb);
        let regular_font_handle: Handle<Font> = Default::default();

        let text_style = TextStyle {
            font: regular_font_handle.clone(),
            font_size: 20.0,
            ..default()
        };

        commands.spawn(Text2dBundle {
            text: Text::from_section("B", text_style.clone()),
            transform: Transform::from_translation(
                float_pos * (SQUARE_SIZE + 1.) + Vec3::new(0., 0., 1.),
            ),
            ..default()
        });
    }

    // remember for later
    generation_data.generated_cells.insert(pos);
}

// finds which cell has been clicked
fn handle_mouse(
    mut commands: Commands,
    mouse_button: Res<Input<MouseButton>>,
    mut drag_data: ResMut<MouseDragData>,
    mut generation_data: ResMut<GenerationData>,
    mut scroll_event: EventReader<MouseWheel>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut camera_q: Query<
        (
            &mut Camera,
            &GlobalTransform,
            &mut Transform,
            &mut OrthographicProjection,
        ),
        Without<Cell>,
    >, // without cell because cell_q also has a Globaltransform thing idk
    mut cell_q: Query<(&mut Sprite, &Handle<Image>, &GlobalTransform, Entity), With<Cell>>,
) {
    let (camera, camera_global_transform, mut camera_transform, mut projection) =
        camera_q.single_mut(); //

    let primary_window = window_q.single();
    // let scale_factor = primary_window.scale_factor() as f32;
    let mut active_entity = None;

    let mouse_window_pos;
    match primary_window.cursor_position() {
        Some(pos) => mouse_window_pos = pos,
        _ => return,
    }

    let mouse_viewport_pos;
    match camera.viewport_to_world_2d(camera_global_transform, mouse_window_pos) {
        Some(pos) => mouse_viewport_pos = pos,
        _ => return,
    }

    // dragging
    if mouse_button.just_pressed(MouseButton::Left) {
        drag_data.is_dragging = true;
        drag_data.drag_start = mouse_window_pos;
        drag_data.camera_start = camera_transform.translation.clone();
    }

    if drag_data.is_dragging {
        let mut mouse_diff: Vec3 = (mouse_window_pos - drag_data.drag_start).extend(0.);

        if mouse_diff.length() > 0.1 {
            drag_data.is_actually_dragging = true;
        }

        mouse_diff.x *= -1.;

        mouse_diff *= projection.scale;

        camera_transform.translation = drag_data.camera_start + mouse_diff;
    }

    // find clicked cell
    if mouse_button.just_released(MouseButton::Left) {
        if !drag_data.is_actually_dragging {
            for (sprite, handle, node_transform, entity) in &mut cell_q.iter_mut() {
                let size = sprite.custom_size.unwrap(); //sprite.rect.unwrap();

                let x_min = node_transform.affine().translation.x - size.x / 2.; // + size.min.x;
                let y_min = node_transform.affine().translation.y - size.y / 2.; // + size.min.y;
                let x_max = node_transform.affine().translation.x + size.x / 2.; // + size.max.x;
                let y_max = node_transform.affine().translation.y + size.y / 2.; // + size.max.y;

                if x_min < mouse_viewport_pos.x
                    && mouse_viewport_pos.x < x_max
                    && y_min < mouse_viewport_pos.y
                    && mouse_viewport_pos.y < y_max
                {
                    active_entity = Some((sprite, handle, node_transform, entity));
                }
            }
        }
    }

    // release dragging
    if mouse_button.just_released(MouseButton::Left) {
        drag_data.is_dragging = false;
        drag_data.is_actually_dragging = false;
    }

    // update color of clicked cell
    if let Some((mut sprite, _handle, _node_transform, _entity)) = active_entity {
        if sprite.color == SQUARE_COLOR {
            sprite.color = Color::rgb(0.75, 0.75, 0.25);
        } else {
            sprite.color = SQUARE_COLOR;
        }
        generate_cells(&mut generation_data, &mut commands);
    }

    // zoom using scrolling
    for event in scroll_event.read() {
        match event.unit {
            MouseScrollUnit::Line => {
                // let mut projection = camera_projection.single_mut();
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

// should generate neighbours that are empty with a limit or smth
fn generate_cells(generation_data: &mut GenerationData, commands: &mut Commands) {
    for _ in 0..3 {
        if generation_data.location[generation_data.dir % 2] >= generation_data.size
            || generation_data.location[generation_data.dir % 2] <= -generation_data.size
        {
            generation_data.dir += 1;
            if generation_data.dir >= 4 {
                generation_data.dir = 0;
                generation_data.size += 1;
            }
        }
        generation_data.location[0] += generation_data.directions[generation_data.dir][0];
        generation_data.location[1] += generation_data.directions[generation_data.dir][1];
        insert_cell(generation_data.location, commands, generation_data);
    }
}

#[derive(Component)]
pub struct Cell;

#[derive(Component)]
pub struct Bomb;

#[derive(Resource)]
struct GenerationData {
    location: [isize; 2],
    directions: [[isize; 2]; 4],
    dir: usize,
    size: isize,
    generated_cells: HashSet<[isize; 2]>,
}

impl Default for GenerationData {
    fn default() -> GenerationData {
        GenerationData {
            location: [0, 0],
            directions: [[1, 0], [0, -1], [-1, 0], [0, 1]],
            dir: 0,
            size: 1,
            generated_cells: HashSet::new(),
        }
    }
}

#[derive(Resource, Default)]
struct MouseDragData {
    is_dragging: bool,
    is_actually_dragging: bool,
    drag_start: Vec2,
    camera_start: Vec3,
}
