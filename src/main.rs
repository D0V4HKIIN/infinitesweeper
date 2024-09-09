use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

use std::collections::HashSet;
use std::collections::VecDeque;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const SQUARE_SIZE: f32 = 20.0;
const SQUARE_COLOR_SUS: Color = Color::rgb(0.25, 0.25, 0.25);
const SQUARE_COLOR_SAFE: Color = Color::rgb(0.75, 0.75, 0.75);
const SQUARE_COLOR_BOMB: Color = Color::rgb(0.75, 0.25, 0.25);
const SQUARE_COLOR_IDK: Color = Color::rgb(0.25, 0.25, 0.75);
const SCROLL_SPEED: f32 = 0.1;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_mouse)
        .init_resource::<MouseDragData>()
        .init_resource::<GeneratedCells>()
        .run();
}

fn setup(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<ColorMaterial>>,
    generated_cells_res: ResMut<GeneratedCells>,
) {
    commands.spawn(Camera2dBundle::default());

    let generated_cells = &mut generated_cells_res.into_inner().generated_cells;
    generate_cells([0, 0], generated_cells, &mut commands);
}

fn is_bomb(pos: [isize; 2]) -> bool {
    let mut hasher = DefaultHasher::new();
    pos.hash(&mut hasher);
    hasher.finish() % 10 == 1 // if == 0 [0,0] bomb
}

fn neighboring_bombs(pos: [isize; 2]) -> usize {
    is_bomb([pos[0] - 1, pos[1]]) as usize
        + is_bomb([pos[0] + 1, pos[1]]) as usize
        + is_bomb([pos[0], pos[1] - 1]) as usize
        + is_bomb([pos[0], pos[1] + 1]) as usize
        + is_bomb([pos[0] - 1, pos[1] - 1]) as usize
        + is_bomb([pos[0] + 1, pos[1] - 1]) as usize
        + is_bomb([pos[0] - 1, pos[1] + 1]) as usize
        + is_bomb([pos[0] + 1, pos[1] + 1]) as usize
}

fn insert_cell(
    pos: [isize; 2],
    commands: &mut Commands,
    generated_cells: &mut HashSet<[isize; 2]>,
    show: bool,
) -> bool {
    // don't regenerate
    if generated_cells.contains(&pos) {
        return neighboring_bombs(pos) == 0;
    }

    let float_pos = Vec3::new(pos[0] as f32, pos[1] as f32, 0.);
    let neighbors_bomb = neighboring_bombs(pos);

    let mut square_color = SQUARE_COLOR_IDK;

    if show {
        if neighbors_bomb != 0 {
            let regular_font_handle: Handle<Font> = Default::default();

            let text_style = TextStyle {
                font: regular_font_handle.clone(),
                font_size: 20.0,
                ..default()
            };

            commands.spawn(Text2dBundle {
                text: Text::from_section(neighbors_bomb.to_string(), text_style.clone()),
                transform: Transform::from_translation(
                    float_pos * (SQUARE_SIZE + 1.) + Vec3::new(0., 0., 1.),
                ),
                ..default()
            });

            square_color = SQUARE_COLOR_SUS;
        } else {
            square_color = SQUARE_COLOR_SAFE;
        }
    }

    // Rectangle
    let mut cell = commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: square_color,
            custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
            ..default()
        },
        transform: Transform::from_translation(float_pos * (SQUARE_SIZE + 1.)),
        ..default()
    });
    cell.insert(Cell { pos });

    if is_bomb(pos) {
        cell.insert(Bomb);
        // lol
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
    generated_cells.insert(pos);

    return neighbors_bomb == 0;
}

// finds which cell has been clicked
fn handle_mouse(
    mut commands: Commands,
    mouse_button: Res<Input<MouseButton>>,
    mut drag_data: ResMut<MouseDragData>,
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
    mut cell_q: Query<(
        &mut Sprite,
        &Handle<Image>,
        &GlobalTransform,
        Entity,
        &Cell,
        Option<&Bomb>,
    )>,
    generated_cells_res: ResMut<GeneratedCells>,
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

    let generated_cells = &mut generated_cells_res.into_inner().generated_cells;

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
            for (sprite, handle, node_transform, entity, cell, bomb) in &mut cell_q.iter_mut() {
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
                    active_entity = Some((sprite, handle, node_transform, entity, cell, bomb));
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
    if let Some((mut sprite, _handle, _node_transform, _entity, cell, bomb)) = active_entity {
        match bomb {
            // you lost
            Some(_) => sprite.color = SQUARE_COLOR_BOMB,
            None => {
                if neighboring_bombs(cell.pos) == 0 {
                    sprite.color = SQUARE_COLOR_SAFE;
                } else {
                    sprite.color = SQUARE_COLOR_SUS
                }
            }
        }

        // generate_cells(&mut generation_data, &mut commands);
        generate_cells(cell.pos, generated_cells, &mut commands);
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
fn generate_cells(
    pos: [isize; 2],
    generated_cells: &mut HashSet<[isize; 2]>,
    commands: &mut Commands,
) {
    let mut to_expand: VecDeque<[isize; 2]> = VecDeque::new();
    let mut to_generate: VecDeque<[isize; 2]> = VecDeque::new();
    to_expand.push_back(pos);
    let mut generations: usize = 0;
    let mut empty_generations: usize = 0;
    while generations < 100 && empty_generations < 1000 {
        let next_pos = match to_expand.pop_front() {
            Some(p) => p,
            None => return,
        };

        let mut generated = false;

        for offset in [
            [-1, 0],
            [1, 0],
            [0, -1],
            [0, 1],
            [-1, -1],
            [-1, 1],
            [1, -1],
            [1, 1],
        ] {
            let neighbor_pos = [next_pos[0] + offset[0], next_pos[1] + offset[1]];

            if !generated_cells.contains(&neighbor_pos) {
                generated = true;
            }

            if insert_cell(neighbor_pos, commands, generated_cells, true) {
                if !to_expand.contains(&neighbor_pos) {
                    to_expand.push_back(neighbor_pos);
                }
            } else {
                to_generate.push_back(neighbor_pos);
            }
        }

        if generated {
            generations += 1;
        } else {
            empty_generations += 1;
        }
    }
    for cell_pos in to_generate {
        for offset in [
            [-1, 0],
            [1, 0],
            [0, -1],
            [0, 1],
            [-1, -1],
            [-1, 1],
            [1, -1],
            [1, 1],
        ] {
            let neighbor_pos = [cell_pos[0] + offset[0], cell_pos[1] + offset[1]];

            insert_cell(neighbor_pos, commands, generated_cells, false);
        }
    }
}

#[derive(Component)]
pub struct Cell {
    pos: [isize; 2],
}

#[derive(Component)]
pub struct Bomb;

// struct GenerationData {
//     pos: [isize; 2],
//     origin: [isize; 2],
//     directions: [[isize; 2]; 4],
//     dir: usize,
//     size: isize,
// }

#[derive(Resource)]
struct GeneratedCells {
    generated_cells: HashSet<[isize; 2]>,
}

// impl Default for GenerationData {
//     fn default() -> GenerationData {
//         GenerationData {
//             pos: [0, 0],
//             origin: [0, 0],
//             directions: [[1, 0], [0, -1], [-1, 0], [0, 1]],
//             dir: 0,
//             size: 1,
//         }
//     }
// }
//
impl Default for GeneratedCells {
    fn default() -> GeneratedCells {
        GeneratedCells {
            generated_cells: HashSet::new(),
        }
    }
}

// pub trait New<T> {
//     fn new(loc: T) -> Self;
// }
// impl New<[isize; 2]> for GenerationData {
//     fn new(loc: [isize; 2]) -> GenerationData {
//         GenerationData {
//             pos: loc,
//             origin: loc,
//             directions: [[1, 0], [0, -1], [-1, 0], [0, 1]],
//             dir: 0,
//             size: 1,
//         }
//     }
// }

#[derive(Resource, Default)]
struct MouseDragData {
    is_dragging: bool,
    is_actually_dragging: bool,
    drag_start: Vec2,
    camera_start: Vec3,
}
