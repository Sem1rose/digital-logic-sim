use std::sync::OnceLock;

use bevy::ui::Pressed;
use bevy::ui_widgets::{Button, observe};
use bevy::window::PrimaryWindow;
use bevy::{
    color::palettes::css::*,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    feathers::{FeathersPlugins, dark_theme::create_dark_theme, theme::UiTheme},
    picking::hover::Hovered,
    prelude::*,
};
use rand;

#[derive(Component)]
struct Placing;
#[derive(Component)]
struct Selected;
#[derive(Component)]
struct HoverMesh;

#[derive(Component, Clone, Copy)]
struct UID(u64);
fn new_uid() -> UID {
    UID(rand::random())
}

#[derive(Component, Default)]
#[require(Transform)]
struct Position(Vec2);

#[derive(Component)]
#[require(Position, Hovered)]
struct Chip;

static HOVERED_MATERIAL: OnceLock<Handle<ColorMaterial>> = OnceLock::new();
static SELECTED_MATERIAL: OnceLock<Handle<ColorMaterial>> = OnceLock::new();

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        FeathersPlugins,
        MeshPickingPlugin,
        FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                enabled: true,
                frame_time_graph_config: FrameTimeGraphConfig {
                    enabled: true,
                    min_fps: 30.0,
                    target_fps: 144.0,
                },
                ..default()
            },
        },
    ))
    .insert_resource(UiTheme(create_dark_theme()))
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            (move_placing, project_positions, place_or_cancel).chain(),
            button_system,
            check_hover,
        ),
    )
    .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn((Camera2d, Transform::from_xyz(0., 0., 0.)));
    commands
        .spawn(Node {
            width: percent(100),
            height: percent(30),
            align_items: AlignItems::End,
            justify_content: JustifyContent::Start,
            align_self: AlignSelf::End,
            ..default()
        })
        .with_child(button("nigga"));

    HOVERED_MATERIAL.get_or_init(|| materials.add(Color::srgba(175., 175., 175., 0.4)));
    SELECTED_MATERIAL.get_or_init(|| materials.add(Color::srgba(200., 200., 200., 0.8)));
}

fn place_or_cancel(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    placing_chips: Query<Entity, (With<Chip>, With<Placing>)>,
    selected_chips: Query<Entity, (With<Chip>, With<Selected>)>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        placing_chips
            .iter()
            .for_each(|x| _ = commands.entity(x).remove::<Placing>());
        selected_chips
            .iter()
            .for_each(|x| _ = commands.entity(x).remove::<Selected>());
    } else if mouse.just_pressed(MouseButton::Right) {
        placing_chips
            .iter()
            .for_each(|x| _ = commands.entity(x).despawn());
    }
}

fn button(capt: impl ToString) -> impl Bundle {
    (
        Button,
        Hovered::default(),
        InheritedVisibility::default(),
        Node {
            width: px(80),
            height: px(25),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(
            Text::new(capt.to_string()),
            TextLayout::justify(Justify::Center),
            TextFont {
                font_size: FontSize::Px(16.),
                ..default()
            }
        )],
        observe(
            |mut activate: On<PointerPress>, mut mouse: ResMut<ButtonInput<MouseButton>>| {
                activate.propagate(false);
                mouse.clear();
            },
        ),
        observe(
            |mut activate: On<PointerRelease>,
             mut mouse: ResMut<ButtonInput<MouseButton>>,
             commands: Commands,
             meshes: ResMut<Assets<Mesh>>,
             materials: ResMut<Assets<ColorMaterial>>,
             window: Single<&Window>,
             camera: Single<(&Camera, &GlobalTransform)>| {
                spawn_chip(commands, meshes, materials, window, camera);

                activate.propagate(false);
                mouse.clear();
            },
        ),
    )
}

fn button_system(
    mut interactions: Query<(&Hovered, Has<Pressed>, &mut BackgroundColor), With<Button>>,
) {
    for (hovered, pressed, mut color) in &mut interactions {
        match (hovered.get(), pressed) {
            (_, true) => {
                *color = BackgroundColor(Color::from(ORANGE));
            }
            (true, false) => {
                *color = BackgroundColor(Color::from(ORANGE_RED));
            }
            (false, false) => {
                *color = BackgroundColor(Color::from(RED));
            }
        }
    }
}
fn check_hover(
    query: Query<(&Hovered, Has<Selected>, &Children)>,
    mut child_query: Query<(&mut MeshMaterial2d<ColorMaterial>, &mut Visibility), With<HoverMesh>>,
) {
    for (hovered, selected, children) in query {
        for &child in children {
            if let Ok((mut material, mut visibility)) = child_query.get_mut(child) {
                if hovered.get() || selected {
                    *visibility = Visibility::Visible;

                    *material = MeshMaterial2d(if selected {
                        SELECTED_MATERIAL.get().unwrap().clone()
                    } else {
                        HOVERED_MATERIAL.get().unwrap().clone()
                    });

                    break;
                } else {
                    *visibility = Visibility::Hidden;

                    break;
                }
            }
        }
    }
}
fn spawn_chip(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let mesh = meshes.add(Rectangle::from_size(Vec2::ONE * 100.));
    let hover_mesh = meshes.add(Rectangle::from_size(Vec2::ONE * 110.));
    let material = materials.add(Color::hsl(rand::random_range(0.0..360.0), 1., 0.5));

    let uid = new_uid();
    commands
        .spawn((
            Chip,
            Placing,
            InheritedVisibility::default(),
            uid,
            Hovered::default(),
            Position(
                window
                    .cursor_position()
                    .and_then(|cursor_pos| camera.0.viewport_to_world_2d(camera.1, cursor_pos).ok())
                    .unwrap_or(Vec2::MAX),
            ),
            observe(
                |mut pressed: On<PointerPress>,
                 mut mouse: ResMut<ButtonInput<MouseButton>>,
                 query: Query<(), Without<Placing>>,
                 mut commands: Commands| {
                    if query.get(pressed.entity).is_ok() {
                        commands.entity(pressed.entity).insert(Selected);

                        pressed.propagate(false);
                        mouse.clear();
                    }
                },
            ),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2d::new(uid.0.to_string()),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((Mesh2d(mesh), MeshMaterial2d(material)));
            parent.spawn((
                HoverMesh,
                Visibility::Hidden,
                Mesh2d(hover_mesh),
                Transform::from_xyz(0., 0., -1.),
                MeshMaterial2d(HOVERED_MATERIAL.get().unwrap().clone()),
            ));
        });
}

fn project_positions(mut positionables: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in &mut positionables {
        transform.translation = position.0.extend(1.);
    }
}

fn move_placing(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut positionables: Query<&mut Position, (With<Placing>, With<Chip>)>,
) {
    if let Some(cursor_world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.0.viewport_to_world(camera.1, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        // if let Some(cursor_position) = window.cursor_position() {
        for mut position in &mut positionables {
            position.0 = cursor_world_position;
        }
    }
}
