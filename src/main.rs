use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
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
        .init_resource::<MouseDragData>()
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
    mut cell_q: Query<(&mut Sprite, &Handle<Image>, &GlobalTransform, Entity), With<Cell>>,
) {
    let (camera, camera_global_transform, mut camera_transform, mut projection) =
        camera_q.single_mut(); //

    let primary_window = window_q.single();
    // let scale_factor = primary_window.scale_factor() as f32;
    let mut active_entity = None;

    // dragging
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(pos) = primary_window.cursor_position() {
            drag_data.is_dragging = true;
            drag_data.drag_start = pos;
            drag_data.camera_start = camera_transform.translation.clone();
            println!("camera pos at {}", camera_transform.translation.xy());
        }
    }

    if mouse_button.just_released(MouseButton::Left) {
        drag_data.is_dragging = false;
    }

    if drag_data.is_dragging {
        if let Some(pos) = primary_window.cursor_position() {
            let mut mouse_diff: Vec3 = (pos - drag_data.drag_start).extend(0.);

            mouse_diff.x *= -1.;

            mouse_diff *= projection.scale;

            camera_transform.translation = drag_data.camera_start + mouse_diff;

            println!(
                    "camera start {} and drag start {}, mouse_diff {}, pos {}, computing {}, huh {}",
                    drag_data.camera_start,
                    drag_data.drag_start,
                    mouse_diff,
                    pos,
                    drag_data.camera_start + mouse_diff,
                    Vec3::new(1., 2., 3.) + Vec3::new(1.1, 2.2, 3.3),
                );
        }
    }

    // find clicked cell
    if mouse_button.just_released(MouseButton::Left) {
        if let Some(pos) = primary_window.cursor_position() {
            if let Some(pos) = camera.viewport_to_world_2d(camera_global_transform, pos) {
                for (sprite, handle, node_transform, entity) in &mut cell_q.iter_mut() {
                    let size = sprite.custom_size.unwrap(); //sprite.rect.unwrap();

                    let x_min = node_transform.affine().translation.x - size.x / 2.; // + size.min.x;
                    let y_min = node_transform.affine().translation.y - size.y / 2.; // + size.min.y;
                    let x_max = node_transform.affine().translation.x + size.x / 2.; // + size.max.x;
                    let y_max = node_transform.affine().translation.y + size.y / 2.; // + size.max.y;

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
        if sprite.color == SQUARE_COLOR {
            sprite.color = Color::rgb(0.75, 0.75, 0.25);
        } else {
            sprite.color = SQUARE_COLOR;
        }
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

#[derive(Component)]
pub struct Cell;

#[derive(Component)]
pub struct Bomb;

#[derive(Resource, Default)]
struct MouseDragData {
    is_dragging: bool,
    drag_start: Vec2,
    camera_start: Vec3,
}
