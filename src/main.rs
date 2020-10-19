#![feature(external_doc)]
#![doc(include = "../README.md")]

use std::{
    collections::{HashMap, HashSet},
    f32::consts::{FRAC_PI_4, PI},
};

use bevy::prelude::*;

use rand::random;

/// An implementation of the classic game "Breakout"
fn main() {
    App::build()
        .add_default_plugins()
        .add_resource(ClearColor(Vec4::from(BACKGROUND_COLOR).into())) // the window's background colour
        .add_resource(Scoreboard { score: 0 })
        .add_resource(GameState::Starting)
        .add_startup_system(setup.system())
        .add_startup_system(start_game_system.system())
        // .add_system(keyboard_system.system())
        .add_system(start_pause_game_system.system())
        .add_system(ball_collision_system.system())
        .add_system(change_color_system.system())
        .add_system(ball_movement_system.system())
        .add_system(ball_rotation_system.system())
        .add_system(ball_trail_system.system())
        .add_system(paddle_movement_system.system())
        .add_system(scoreboard_system.system())
        .add_system(fps_system.system())
        .add_system(entity_count_system.system())
        .add_system(color_material_count_system.system())
        .add_system(color_handle_count_system.system())
        .add_system(fade_out_system.system())
        .add_system(check_win_condition_system.system())
        .add_system(render_game_state_text_system.system())
        .add_system(end_game_system.system())
        .run();
}

#[derive(Eq, PartialEq, Hash, Debug)]
enum Handlers {
    DoubleTapLeft,
    DoubleTapRight,
}

/// Determine whether two rectangles overlap during a frame.
///
/// The problem with Bevy's is that during a frame, one rectangle might be *very close*
/// to another rectangle, then the following frame, it has moved >50% of its "width"
/// into the rectangle, so this determines that the collision had approached from the
/// opposite direction. You can also have multiple collisions during a frame (not yet implemented), and multiple
/// frame collisions (not intentional). Additionally, it is possible to have both vertical and horizontal
/// collisions at the same time i.e. outside corner to outside corner, or outside corner to inside corner.
fn collide(
    ball_pos: Vec3,
    ball_size: Vec2,
    other_pos: Vec3,
    other_size: Vec2,
    ball_velocity: &Vec3,
    time_delta: f32,
) -> Option<WillCollide> {
    let a_min_prev = ball_pos.truncate() - ball_size / 2.0;
    let a_max_prev = ball_pos.truncate() + ball_size / 2.0;
    let a_min = a_min_prev + ball_velocity.truncate() * time_delta;
    let a_max = a_max_prev + ball_velocity.truncate() * time_delta;
    let b_min = other_pos.truncate() - other_size / 2.0;
    let b_max = other_pos.truncate() + other_size / 2.0;

    // check to see if the two rectangles are intersecting
    if a_min.x() < b_max.x()
        && a_max.x() > b_min.x()
        && a_min.y() < b_max.y()
        && a_max.y() > b_min.y()
    {
        let (x_collision, x_collision_site) = if a_max_prev.x() < b_min.x() && a_max.x() > b_min.x()
        {
            (CollisionX::Left, b_min.x() - ball_size.x() / 2.0)
        } else if a_min_prev.x() > b_max.x() && a_min.x() < b_max.x() {
            (CollisionX::Right, b_max.x() + ball_size.x() / 2.0)
        } else {
            (CollisionX::None, 0.0)
        };

        let (y_collision, y_collision_site) = if a_max_prev.y() < b_min.y() && a_max.y() > b_min.y()
        {
            (CollisionY::Bottom, b_min.y() - ball_size.y() / 2.0)
        } else if a_min_prev.y() > b_max.y() && a_min.y() < b_max.y() {
            (CollisionY::Top, b_max.y() + ball_size.y() / 2.0)
        } else {
            (CollisionY::None, 0.0)
        };

        Some(WillCollide {
            x: (x_collision, x_collision_site),
            y: (y_collision, y_collision_site),
        })
    } else {
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
enum CollisionX {
    Left,
    Right,
    None,
}

#[derive(Debug, PartialEq, Eq)]
enum CollisionY {
    Top,
    Bottom,
    None,
}

#[derive(Debug)]
struct WillCollide {
    x: (CollisionX, f32),
    y: (CollisionY, f32),
}

#[derive(Debug, Copy, Clone)]
enum Collider {
    BottomWall,
    OtherWall,
    Brick,
    Paddle,
}

// const BACKGROUND_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const BACKGROUND_COLOR: [f32; 4] = [0.7, 0.7, 0.7, 0.0];
const DESPAWN_TIME: f32 = 2.0;

struct Paddle {
    speed: f32,
}

#[derive(Debug)]
struct Ball {
    velocity: Vec3,
    rotation: f32,
    rotational_velocity: f32,
    collided: Option<(WillCollide, Collider, Color)>,
    spin: Spin,
    last_paddle_offset: f32,
}

struct GameStateText;

#[derive(Debug, Eq, PartialEq)]
enum Spin {
    Clockwise,
    CounterCw,
}

struct Scoreboard {
    score: usize,
}

struct Score;

struct Framerate;

struct EntityCount;

struct FadeOut {
    fade_out_time: f32,
    starting_color: Color,
}

struct Name(String);

struct DespawnOnEnd;

struct Brick(bool);

#[derive(PartialEq, Eq)]
enum GameState {
    Starting,
    Restarting,
    Playing,
    Paused,
    Win,
    Lose,
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // this is now relative to the PROJECT_ROOT/assets directory, will panic if not found
    let font = asset_server.load("FiraSans-Bold.ttf");
    // Add the game's entities to our world
    commands
        // cameras
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default())
        // scoreboard
        .spawn(TextComponents {
            text: Text {
                font: font.clone(),
                value: "".to_string(),
                style: TextStyle {
                    color: Color::rgb(0.2, 0.2, 0.8),
                    font_size: 40.0,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Score)
        // framerate
        .spawn(TextComponents {
            text: Text {
                font: font.clone(),
                value: "".to_string(),
                style: TextStyle {
                    color: Color::rgb(0.2, 0.2, 0.8),
                    font_size: 40.0,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(45.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Framerate)
        // entity count
        .spawn(TextComponents {
            text: Text {
                font: font.clone(),
                value: "".to_string(),
                style: TextStyle {
                    color: Color::rgb(0.2, 0.2, 0.8),
                    font_size: 40.0,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(85.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with(EntityCount)
        // color material count
        .spawn(TextComponents {
            text: Text {
                font: font.clone(),
                value: "".to_string(),
                style: TextStyle {
                    color: Color::rgb(0.2, 0.2, 0.8),
                    font_size: 40.0,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(125.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with(ColorMaterialCount)
        // color handle count
        .spawn(TextComponents {
            text: Text {
                font: font.clone(),
                value: "".to_string(),
                style: TextStyle {
                    color: Color::rgb(0.2, 0.2, 0.8),
                    font_size: 40.0,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(165.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with(ColorHandleCount)
        // game state text
        .spawn(TextComponents {
            text: Text {
                font: font.clone(),
                value: "".to_string(),
                style: TextStyle {
                    color: Color::rgb(1.0, 1.0, 1.0),
                    font_size: 100.0,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(240.0),
                    left: Val::Px(480.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with(GameStateText);

    // Add walls
    let wall_material = materials.add(Color::rgb(0.5, 0.5, 0.5).into());
    let wall_thickness = 10.0;
    let bounds = Vec2::new(900.0, 600.0);

    commands
        // left
        .spawn(SpriteComponents {
            material: wall_material.clone(),
            transform: Transform::from_translation(Vec3::new(-bounds.x() / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(wall_thickness, bounds.y() + wall_thickness)),
            ..Default::default()
        })
        .with(Collider::OtherWall)
        .with(Name("Left wall".into()))
        // right
        .spawn(SpriteComponents {
            material: wall_material.clone(),
            transform: Transform::from_translation(Vec3::new(bounds.x() / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(wall_thickness, bounds.y() + wall_thickness)),
            ..Default::default()
        })
        .with(Collider::OtherWall)
        .with(Name("Right wall".into()))
        // bottom
        .spawn(SpriteComponents {
            material: wall_material.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, -bounds.y() / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(bounds.x() + wall_thickness, wall_thickness)),
            ..Default::default()
        })
        .with(Collider::BottomWall)
        // .with(Collider::OtherWall)
        .with(Name("Bottom wall".into()))
        // top
        .spawn(SpriteComponents {
            material: wall_material.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, bounds.y() / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(bounds.x() + wall_thickness, wall_thickness)),
            ..Default::default()
        })
        .with(Collider::OtherWall)
        .with(Name("Top wall".into()));
}

fn end_game_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    materials: ResMut<Assets<ColorMaterial>>,
    mut scoreboard: ResMut<Scoreboard>,
    mut despawn_query: Query<(Entity, &DespawnOnEnd)>,
    // color_material_handle_query: Query<&Handle<ColorMaterial>>,
) {
    if *game_state == GameState::Restarting {
        for (entity, _) in &mut despawn_query.iter() {
            // below no longer required - Bevy now handles this for us
            // if let Ok(handle) = &color_material_handle_query.get::<Handle<ColorMaterial>>(entity) {
            //     materials.remove(handle);
            // }
            commands.despawn(entity);
        }
        scoreboard.score = 0;
        start_game_system(commands, materials);
        *game_state = GameState::Starting;
    }
}

fn start_game_system(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands
        // paddle
        .spawn(SpriteComponents {
            material: materials.add(Color::BLACK.into()),
            transform: Transform::from_translation(Vec3::new(0.0, -215.0, 20.0)),
            sprite: Sprite::new(Vec2::new(120.0, 30.0)),
            ..Default::default()
        })
        .with(Paddle { speed: 500.0 })
        .with(Collider::Paddle)
        .with(DespawnOnEnd)
        .with(Name("Paddle".into()))
        // ball
        .spawn(SpriteComponents {
            material: materials.add(Color::WHITE.into()),
            transform: Transform {
                translation: Vec3::new(0.0, -30.0, 10.0),
                rotation: Quat::from_rotation_z(FRAC_PI_4),
                ..Default::default()
            },
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            draw: Draw {
                is_transparent: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Ball {
            velocity: 400.0 * Vec3::new(1.0, -1.0, 0.0).normalize(),
            collided: None,
            rotation: FRAC_PI_4,
            rotational_velocity: 2.0 * PI, // radians per second
            spin: Spin::Clockwise,
            last_paddle_offset: 0.0,
        })
        .with(DespawnOnEnd)
        .with(Name("Ball".into()));

    // Add bricks
    let brick_rows = 4;
    let brick_columns = 5;
    let brick_spacing = 20.0;
    let brick_size = Vec2::new(150.0, 30.0);
    let bricks_width = brick_columns as f32 * (brick_size.x() + brick_spacing) - brick_spacing;
    // center the bricks and move them up a bit
    let bricks_offset = Vec3::new(-(bricks_width - brick_size.x()) / 2.0, 100.0, 0.0);

    for row in 0..brick_rows {
        let y_position = row as f32 * (brick_size.y() + brick_spacing);
        for column in 0..brick_columns {
            let brick_position = Vec3::new(
                column as f32 * (brick_size.x() + brick_spacing),
                y_position,
                0.0,
            ) + bricks_offset;

            let [r, g, b] = random::<[u8; 3]>();
            let color = Color::rgb_u8(r, g, b);
            commands
                // brick
                .spawn(SpriteComponents {
                    material: materials.add(color.into()),
                    sprite: Sprite::new(brick_size),
                    transform: Transform::from_translation(brick_position),
                    draw: Draw {
                        is_transparent: true,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(Collider::Brick)
                .with(Brick(true))
                .with(DespawnOnEnd)
                .with(Name(format!("Brick {}-{}", row, column).into()));
        }
    }
}

fn _keyboard_system(keyboard_input: Res<Input<KeyCode>>, time: Res<Time>) {
    let t = time.time_since_startup().as_nanos();
    #[derive(Debug)]
    enum Temp {
        JustPressed,
        Pressed,
        JustReleased,
    }
    use Temp::*;

    for key in &[
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::LControl,
        KeyCode::LAlt,
        KeyCode::RControl,
        KeyCode::RAlt,
        KeyCode::Space,
        KeyCode::R,
    ] {
        if keyboard_input.just_pressed(*key) {
            dbg!(t, JustPressed, *key);
        }
        if keyboard_input.pressed(*key) {
            dbg!(t, Pressed, *key);
        }
        if keyboard_input.just_released(*key) {
            dbg!(t, JustReleased, *key);
        }
    }

    // [src\main.rs:464] t = 42619449400
    // [src\main.rs:464] JustPressed = JustPressed
    // [src\main.rs:464] *key = RControl
    // [src\main.rs:467] t = 42619449400
    // [src\main.rs:467] Pressed = Pressed
    // [src\main.rs:467] *key = RControl
    // [src\main.rs:470] t = 42645905900
    // [src\main.rs:470] JustReleased = JustReleased
    // [src\main.rs:470] *key = RControl
    // frame A, justpressed yes + pressed yes(; frame B, pressed yes); frame C, justreleased yes
}

fn start_pause_game_system(mut game_state: ResMut<GameState>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_released(KeyCode::Space) {
        *game_state = match *game_state {
            GameState::Starting => GameState::Playing,
            GameState::Restarting => GameState::Restarting,
            GameState::Playing => GameState::Paused,
            GameState::Paused => GameState::Playing,
            GameState::Win => GameState::Restarting,
            GameState::Lose => GameState::Restarting,
        }
    } else if keyboard_input.just_released(KeyCode::R) {
        *game_state = GameState::Restarting;
    }
}

fn wrap(num: f32, min: f32, max: f32) -> f32 {
    if num < min {
        max - (min - num)
    } else if num > max {
        min - (max - num)
    } else {
        num
    }
}

fn ball_trail_system(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&Ball, &Transform, &Handle<ColorMaterial>)>,
) {
    if *game_state == GameState::Playing {
        for (_ball, &transform, material_handle) in &mut query.iter() {
            let mut transform = transform;
            transform.translation.set_z(0.0);
            let color = materials.get(material_handle).unwrap().color;
            let color = color_to_vec4(color).lerp(color_to_vec4(Color::WHITE), 0.4);
            let color: Color = color.into();
            let material = materials.add(color.into());
            commands
                .spawn(SpriteComponents {
                    material,
                    transform,
                    sprite: Sprite::new(Vec2::new(30.0, 30.0)),
                    draw: Draw {
                        is_transparent: true,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(DespawnOnEnd)
                .with(FadeOut {
                    fade_out_time: 1.0,
                    starting_color: color,
                });
        }
    }
}

fn ball_rotation_system(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut query: Query<(&Ball, &mut Transform)>,
) {
    if *game_state == GameState::Playing {
        for (ball, mut transform) in &mut query.iter() {
            // match ball.spin {
            //     Spin::Clockwise => {
            //         ball.rotation -= ball.rotational_velocity * time.delta_seconds;
            //     }
            //     Spin::CounterCw => {
            //         ball.rotation += ball.rotational_velocity * time.delta_seconds;
            //     }
            // }
            // ball.rotation = wrap(ball.rotation, 0.0, PI);
            // *rotation = Rotation::from_rotation_z(ball.rotation);
            // dbg!(&transform);
            let current_angle = transform.rotation.to_axis_angle().1;
            let new_angle = wrap(
                current_angle
                    + ball.rotational_velocity
                        * time.delta_seconds
                        * match ball.spin {
                            Spin::Clockwise => -1.0,
                            Spin::CounterCw => 1.0,
                        },
                0.0,
                PI,
            );
            transform.rotation = Quat::from_rotation_z(new_angle);
        }
    }
}

fn paddle_movement_system(
    time: Res<Time>,
    game_state: Res<GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    mut key_combos_resource: Local<Option<HashMap<Handlers, KeyCombo>>>,
    mut query: Query<(&Paddle, &mut Transform)>,
) {
    // initialise local
    if let None = *key_combos_resource {
        let mut h: HashMap<Handlers, KeyCombo> = HashMap::new();
        h.insert(
            Handlers::DoubleTapLeft,
            KeyCombo::new(
                vec![Keypress::new(KeyCode::Left), Keypress::new(KeyCode::Left)],
                0.5,
                0.25,
                false,
            ),
        );
        h.insert(
            Handlers::DoubleTapRight,
            KeyCombo::new(
                vec![Keypress::new(KeyCode::Right), Keypress::new(KeyCode::Right)],
                0.5,
                0.25,
                false,
            ),
        );
        *key_combos_resource = Some(h);
    }
    if *game_state == GameState::Playing {
        for (paddle, mut transform) in &mut query.iter() {
            let mut direction = 0.0;
            if keyboard_input.pressed(KeyCode::Left) {
                direction -= 1.0;
            }
            if keyboard_input.pressed(KeyCode::Right) {
                direction += 1.0;
            }
            // if both are pressed at the same time, we don't move, i.e. direction = 0.0
            if let Some(key_combos) = &mut *key_combos_resource {
                if let Some(handler) = key_combos.get_mut(&Handlers::DoubleTapLeft) {
                    if keyboard_input.pressed(KeyCode::Right) {
                        handler.reset();
                    } else if handler.done(&keyboard_input, time.delta_seconds) {
                        // temporary, instead increase the paddle speed temporarily
                        *transform.translation.x_mut() -= 180.0;
                    }
                }
                if let Some(handler) = key_combos.get_mut(&Handlers::DoubleTapRight) {
                    if keyboard_input.pressed(KeyCode::Left) {
                        handler.reset();
                    } else if handler.done(&keyboard_input, time.delta_seconds) {
                        // temporary, instead increase the paddle speed temporarily
                        *transform.translation.x_mut() += 180.0;
                    }
                }
            }

            *transform.translation.x_mut() += time.delta_seconds * direction * paddle.speed;

            // bound the paddle partially within the walls
            // paddle width is 120, arena bounds are -380 to 380
            *transform.translation.x_mut() =
                transform.translation.x().max(-500.0).min(500.0);
        }
    }
}

fn ball_movement_system(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut ball_query: Query<(&mut Ball, &mut Transform)>,
) {
    if *game_state == GameState::Playing {
        // clamp the timestep to stop the ball from escaping when the game starts
        let delta_seconds = f32::min(0.2, time.delta_seconds);

        for (mut ball, mut transform) in &mut ball_query.iter() {
            // either we continue in the current direction with current velocity
            // or we take two moves with flips, so we need a midpoint, and a new direction
            let handle_collision = match &ball.collided {
                None => None,
                Some((collision, collider, _color)) => {
                    let start = transform.translation;
                    let extrapolated = start + ball.velocity * delta_seconds;
                    // check if x is a collision first
                    let x_collided = collision.x.0 != CollisionX::None;
                    let y_collided = collision.y.0 != CollisionY::None;
                    let midpoint = f32::min(
                        if x_collided {
                            let x_collision_site = &collision.x.1;
                            let x_start = start.x();
                            let x_extrapolated = extrapolated.x();
                            (x_collision_site - x_start) / (x_extrapolated - x_start)
                        } else {
                            0.0
                        },
                        if y_collided {
                            let y_collision_site = &collision.y.1;
                            let y_start = start.y();
                            let y_extrapolated = extrapolated.y();
                            (y_collision_site - y_start) / (y_extrapolated - y_start)
                        } else {
                            0.0
                        },
                    );
                    let new_velocity = if let Collider::Paddle = collider {
                        if collision.y.0 == CollisionY::Top && ball.velocity.y() < 0.0 {
                            let magnitude = ball.velocity.length();
                            // max offset is half the width of the paddle (60) plus half the width of the ball (15)
                            let angle = ball.last_paddle_offset.max(-75.0).min(75.0) / 75.0
                                * (PI / 180.0 * 85.0);
                            let x = angle.sin();
                            let y = angle.cos();
                            let new_velocity = Vec3::new(x, y, 0.0) * magnitude;
                            new_velocity
                        } else {
                            ball.velocity
                        }
                    } else {
                        let mut new_velocity = ball.velocity.clone();
                        // reflect the ball when it collides
                        // only reflect if the ball's velocity is going in the opposite direction of the collision
                        // reflect velocity on the x-axis if we hit something on the x-axis
                        if (collision.x.0 == CollisionX::Left && ball.velocity.x() > 0.0)
                            || (collision.x.0 == CollisionX::Right && ball.velocity.x() < 0.0)
                        {
                            *new_velocity.x_mut() *= -1.0;
                        }
                        // reflect velocity on the y-axis if we hit something on the y-axis
                        if (collision.y.0 == CollisionY::Bottom && ball.velocity.y() > 0.0)
                            || (collision.y.0 == CollisionY::Top && ball.velocity.y() < 0.0)
                        {
                            *new_velocity.y_mut() *= -1.0;
                        }
                        let mut magnitude = new_velocity.length();
                        if let Collider::Brick = collider {
                            magnitude = magnitude + 30.0;
                            new_velocity *= magnitude / new_velocity.length();
                        } else if let Collider::OtherWall = collider {
                            magnitude = (magnitude - 20.0).max(100.0); // minimum velocity is 100
                            new_velocity *= magnitude / new_velocity.length();
                        }
                        new_velocity
                    };
                    Some((midpoint, new_velocity))
                }
            };
            if let Some((midpoint, new_velocity)) = handle_collision {
                // half move
                transform.translation += ball.velocity * delta_seconds * midpoint;
                // update velocity
                ball.velocity = new_velocity;
                ball.rotational_velocity = new_velocity.length() / 400.0 * 2.0 * PI;
                // finish the move
                transform.translation += ball.velocity * delta_seconds * (1.0 - midpoint);
            } else {
                transform.translation += ball.velocity * delta_seconds;
            }
            ball.collided = None;
        }
    }
}

fn scoreboard_system(scoreboard: Res<Scoreboard>, mut query: Query<(&mut Text, &Score)>) {
    for (mut text, _score_marker) in &mut query.iter() {
        let text_value = format!("Score: {}", scoreboard.score);
        if text.value != text_value {
            text.value = text_value;
        }
    }
}

fn fps_system(time: Res<Time>, mut query: Query<(&mut Text, &Framerate)>) {
    for (mut text, _framerate_marker) in &mut query.iter() {
        let text_value = format!("FPS: {:.0}", 1.0 / time.delta_seconds);
        if text.value != text_value {
            text.value = text_value;
        }
    }
}

fn entity_count_system(
    mut query: Query<(&mut Text, &EntityCount)>,
    mut entity_query: Query<Entity>,
) {
    for (mut text, _entity_count_marker) in &mut query.iter() {
        let mut entity_count = 0;
        for _ in &mut entity_query.iter() {
            entity_count += 1;
        }
        let text_value = format!("Entities: {}", entity_count);
        if text.value != text_value {
            text.value = text_value;
        }
    }
}

struct ColorMaterialCount;

fn color_material_count_system(
    color_query: Res<Assets<ColorMaterial>>,
    mut query: Query<(&mut Text, &ColorMaterialCount)>,
    // mut color_query: Query<color>,
) {
    for (mut text, _color_count_marker) in &mut query.iter() {
        let mut color_count = 0;
        for _ in &mut color_query.iter() {
            color_count += 1;
        }
        let text_value = format!("Color Materials: {}", color_count);
        if text.value != text_value {
            text.value = text_value;
        }
    }
}
struct ColorHandleCount;

fn color_handle_count_system(
    // color_handle_query: Res<Assets<ColorMaterial>>,
    mut query: Query<(&mut Text, &ColorHandleCount)>,
    mut color_handle_query: Query<&Handle<ColorMaterial>>,
) {
    for (mut text, _color_handle_count_marker) in &mut query.iter() {
        let mut color_handle_count = 0;
        for _ in &mut color_handle_query.iter() {
            color_handle_count += 1;
        }
        let text_value = format!("Color Handles: {}", color_handle_count);
        if text.value != text_value {
            text.value = text_value;
        }
    }
}

fn color_to_vec4(color: Color) -> Vec4 {
    let color: [f32; 4] = color.into();
    color.into()
}

fn ball_collision_system(
    mut commands: Commands,
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut scoreboard: ResMut<Scoreboard>,
    materials: Res<Assets<ColorMaterial>>,
    mut ball_query: Query<(
        Entity,
        &mut Ball,
        &Transform,
        &Sprite,
        &Handle<ColorMaterial>,
    )>,
    brick_query: Query<&mut Brick>,
    mut collider_query: Query<(
        Entity,
        &Collider,
        &Transform,
        &Sprite,
        &Name,
        &Handle<ColorMaterial>,
    )>,
) {
    if *game_state == GameState::Playing {
        let mut ball_count = 0;
        for (..) in &mut ball_query.iter() {
            ball_count += 1;
        }
        for (ball_entity, mut ball, ball_transform, sprite, ball_color_material_handle) in
            &mut ball_query.iter()
        {
            let ball_size = sprite.size;

            // check collision with walls, bricks and paddles
            for (
                collider_entity,
                collider,
                collider_transform,
                sprite,
                _name,
                collider_color_material_handle,
            ) in &mut collider_query.iter()
            {
                if let Some(collision) = collide(
                    ball_transform.translation,
                    ball_size,
                    collider_transform.translation,
                    sprite.size,
                    &ball.velocity,
                    time.delta_seconds,
                ) {
                    if let Collider::Paddle = *collider {
                        if collision.y.0 == CollisionY::Top && ball.velocity.y() < 0.0 {
                            ball.spin = if ball_transform.translation.x()
                                < collider_transform.translation.x()
                            {
                                Spin::CounterCw
                            } else {
                                Spin::Clockwise
                            };
                            // TODO: defer this to the movementsystem
                            ball.last_paddle_offset = ball_transform.translation.x()
                                - collider_transform.translation.x();
                        }
                    } else if let Collider::BottomWall = *collider {
                        let color = materials.get(ball_color_material_handle).unwrap().color;
                        commands.insert_one(
                            ball_entity,
                            FadeOut {
                                fade_out_time: DESPAWN_TIME,
                                starting_color: color,
                            },
                        );
                        commands.remove_one::<Ball>(ball_entity);
                        ball_count -= 1;
                        if ball_count <= 0 {
                            *game_state = GameState::Lose;
                            return;
                        }
                    } else if let Collider::Brick = *collider {
                        // scorable colliders should be despawned and increment the scoreboard on collision
                        commands.insert_one(
                            collider_entity,
                            FadeOut {
                                fade_out_time: DESPAWN_TIME,
                                starting_color: Color::WHITE,
                            },
                        );
                        commands.remove_one::<Collider>(collider_entity);
                        if let Some(mut brick) = brick_query.get_mut::<Brick>(collider_entity).ok()
                        {
                            brick.0 = false;
                        }
                        scoreboard.score += 1;
                    }

                    let color = materials
                        .get(collider_color_material_handle)
                        .unwrap()
                        .color;
                    // TODO: store the entity instead of copying the collider and color
                    ball.collided = Some((collision, *collider, color));
                    // TODO: I think this is a tempfix for the ball escaping the arena, i.e. it can only hit collide with one entity only
                    // nope, ball still escapes - the correct fix is to allow for multiple collisions in one frame
                    // (e.g. the paddle AND the side wall, a brick AND a wall, top AND side walls)
                    break;
                }
            }
        }
    }
}

fn change_color_system(
    game_state: Res<GameState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ball_query: Query<(&Ball, &Handle<ColorMaterial>)>,
    mut paddle_query: Query<(&Paddle, &Handle<ColorMaterial>)>,
) {
    if *game_state != GameState::Paused {
        for (ball, ball_material_handle) in &mut ball_query.iter() {
            if let Some((_, collider, new_color)) = ball.collided {
                let ball_material = materials.get_mut(ball_material_handle).unwrap();
                let old_color = color_to_vec4(ball_material.color);
                match collider {
                    Collider::Brick => {
                        let new_color: [f32; 4] = new_color.into();
                        let new_color: Vec4 = new_color.into();
                        ball_material.color = old_color.lerp(new_color, 0.5).into();
                    }
                    Collider::BottomWall => {}
                    Collider::OtherWall => {}
                    Collider::Paddle => {
                        if let Some((
                            WillCollide {
                                y: (CollisionY::Top, _),
                                ..
                            },
                            _collider,
                            _color,
                        )) = ball.collided
                        {
                            for (_paddle, paddle_material_handle) in &mut paddle_query.iter() {
                                let paddle_material =
                                    materials.get_mut(paddle_material_handle).unwrap();
                                paddle_material.color = old_color.into();
                            }
                        }
                    }
                }
            }
        }
    }
}

fn fade_out_system(
    mut commands: Commands,
    time: Res<Time>,
    game_state: Res<GameState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut despawn_query: Query<(Entity, &mut FadeOut, &Handle<ColorMaterial>)>,
) {
    let rgb = Vec4::new(1.0, 1.0, 1.0, 0.0);
    if *game_state != GameState::Paused && *game_state != GameState::Restarting {
        for (entity, mut fade_out, material_handle) in &mut despawn_query.iter() {
            if fade_out.fade_out_time == DESPAWN_TIME {
                let material = materials.get_mut(material_handle).unwrap();
                material.color = fade_out.starting_color;
            }
            fade_out.fade_out_time -= time.delta_seconds;
            if fade_out.fade_out_time > 0.0 {
                let material = materials.get_mut(material_handle).unwrap();
                // let color = color_to_vec4(material.color);
                let color = color_to_vec4(fade_out.starting_color);
                material.color = (color * rgb
                    + Vec4::new(0.0, 0.0, 0.0, fade_out.fade_out_time / DESPAWN_TIME))
                .into();
            } else {
                // end_game_system (GameState::Restarting) takes precedence on despawning, so that we don't
                // attempt to despawn the same entity in the same frame (crashes)
                commands.despawn(entity);
                materials.remove(material_handle);
            }
        }
    }
}

fn render_game_state_text_system(
    game_state: Res<GameState>,
    mut query: Query<(&mut Text, &GameStateText)>,
) {
    for (mut text, _game_state_text) in &mut query.iter() {
        let text_value = match *game_state {
            GameState::Starting => "Press Space to start",
            GameState::Playing => "",
            GameState::Restarting => "",
            GameState::Paused => "PAUSED",
            GameState::Win => "YOU WIN! :D",
            GameState::Lose => "YOU LOSE :(",
        }.into();
        if text.value != text_value {
            text.value = text_value;
        }
    }
}

fn check_win_condition_system(mut game_state: ResMut<GameState>, mut brick_query: Query<&Brick>) {
    let mut brick_count = 0;
    for brick in &mut brick_query.iter() {
        if brick.0 {
            brick_count += 1;
        }
    }
    if brick_count == 0 && *game_state == GameState::Playing {
        *game_state = GameState::Win;
    }
}

/// A key press (with or without modifiers)
#[derive(Clone)]
struct Keypress {
    key: KeyCode,
    modifiers: HashSet<KeyCode>,
}

impl Keypress {
    /// Register a key press (any key) - takes one argument, a [`KeyCode`]
    fn new(key: KeyCode) -> Self {
        Self {
            key,
            modifiers: HashSet::new(),
        }
    }
    /// Add a modifier (any key) - takes one argument, a [`KeyCode`]
    #[allow(dead_code)]
    fn with_modifier(&mut self, modifier: KeyCode) -> &mut Self {
        self.modifiers.insert(modifier);
        self
    }
    /// Returns true if this key was just pressed, and all registered modifiers are currently pressed.
    /// Takes one argument - pass it a reference to the keyboard input resource (`&Res<Input<KeyCode>>`)
    fn just_pressed(&self, input: &Res<Input<KeyCode>>) -> bool {
        if input.just_pressed(self.key) {
            for &modifier in &self.modifiers {
                if !input.pressed(modifier) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
    /// Returns true if this key was just released, and all registered modifiers are currently pressed.
    /// Takes one argument - pass it a reference to the keyboard input resource (`&Res<Input<KeyCode>>`)
    fn just_released(&self, input: &Res<Input<KeyCode>>) -> bool {
        if input.just_released(self.key) {
            for &modifier in &self.modifiers {
                if !input.pressed(modifier) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

enum WaitForKey {
    Press,
    Release,
}

struct KeyCombo {
    keypress_sequence: Vec<Keypress>,
    max_wait_for_release: f32,
    wait_for_release_timer: f32,
    max_wait_for_press: f32,
    wait_for_press_timer: f32,
    waiting_for: WaitForKey,
    index: usize,
    done_on_press: bool,
}

// if index is at 0, this thing waits for the first key in the sequence
// when this key is just pressed, it starts the
impl KeyCombo {
    /// Register a key combo
    ///
    /// # Arguments
    ///
    /// * `keypress_sequence`: what sequence of [key presses](Keypress) triggers a "done"
    /// * `max_wait_for_press`: how long to allow between key presses
    /// * `max_wait_for_release`: how long to allow a key to be held down to be treated as a key press
    /// * `done_on_press`: if true, this ends the key combo when the last key is pressed, not released
    ///
    /// # Example
    ///
    /// ```
    /// let key_combo = KeyCombo::new(
    ///     vec![Keypress::new(KeyCode::Left), Keypress::new(KeyCode::Left)],
    ///     0.5,
    ///     0.25,
    /// );
    /// ```
    fn new(
        keypress_sequence: Vec<Keypress>,
        max_wait_for_press: f32,
        max_wait_for_release: f32,
        done_on_press: bool,
    ) -> Self {
        Self {
            keypress_sequence,
            max_wait_for_release,
            wait_for_release_timer: 0.0,
            max_wait_for_press,
            wait_for_press_timer: 0.0,
            waiting_for: WaitForKey::Press,
            index: 0,
            done_on_press,
        }
    }
    fn reset(&mut self) {
        self.wait_for_release_timer = 0.0;
        self.wait_for_press_timer = 0.0;
        self.waiting_for = WaitForKey::Press;
        self.index = 0;
    }
    /// Check if a key combo has been fully entered
    fn done(&mut self, input: &Res<Input<KeyCode>>, delta_time: f32) -> bool {
        let current_key = &self.keypress_sequence[self.index];
        let mut reset = false;
        match self.waiting_for {
            WaitForKey::Press => {
                self.wait_for_press_timer += delta_time;
                if current_key.just_pressed(input) {
                    self.waiting_for = WaitForKey::Release;
                } else if self.wait_for_press_timer >= self.max_wait_for_press {
                    reset = true;
                }
            }
            WaitForKey::Release => {
                self.wait_for_release_timer += delta_time;
                if current_key.just_released(input) {
                    self.index += 1;
                    self.waiting_for = WaitForKey::Press;
                } else if self.wait_for_release_timer >= self.max_wait_for_release {
                    reset = true;
                }
            }
        }
        if reset {
            self.reset();
        }
        if (!self.done_on_press && self.index >= self.keypress_sequence.len())
            || (self.done_on_press && self.index >= self.keypress_sequence.len() - 1)
        {
            self.reset();
            true
        } else {
            false
        }
    }
}
