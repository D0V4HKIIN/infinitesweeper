use bevy::{prelude::*, window::{PrimaryWindow, exit_on_primary_closed}};

const SQUARE_SIZE: f32 = 20.0;
const SQUARE_COLOR: Color = Color::rgb(0.25, 0.25, 0.75);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Game>()
        .add_systems(Startup, setup)
        .add_systems(Update, expand_board)
        .add_systems(Update, handle_cell_click)
        // .add_systems(Update, exit_on_primary_closed)
        .run();
}

fn setup(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Rectangle
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: SQUARE_COLOR,
                custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        })
        .insert(Cell);
}

// finds which cell has been clicked
fn handle_cell_click(
    mouse_input: Res<Input<MouseButton>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut cell_q: Query<(&mut Sprite, &Handle<Image>, &GlobalTransform, Entity), With<Cell>>,
) {
    let (camera, camera_transform) = camera_q.single();
    let primary_window = window_q.single();
    let scale_factor = primary_window.scale_factor() as f32;
    let mut active_entity = None;

    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(pos) = primary_window.cursor_position() {
            if let Some(pos) = camera.viewport_to_world_2d(camera_transform, pos) {
                println!("clicked at {}, {}", pos.x, pos.y);
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

    if let Some((mut sprite, handle, node_transform, entity)) = active_entity {
        println!("active entity: {:#?}", sprite);
        sprite.color = Color::rgb(0.75, 0.75, 0.25);
    }
}

fn expand_board(game: ResMut<Game>, mut commands: Commands, time: Res<Time>) {
    // println!("{:#?}", game.bombs);
}

#[derive(Resource, Default)]
struct Game {
    bombs: Vec<(i32, i32)>,
}

#[derive(Component)]
pub struct Cell;
