use std::f32::consts::{FRAC_PI_4, PI};

use bevy::{prelude::*, render::pass::ClearColor};

// use std::collections::HashMap;

// use serde::{Serialize, Deserialize};

/// An implementation of the classic game "Breakout"
fn main() {
    App::build()
        .add_default_plugins()
        .add_resource(ClearColor(Color::rgb(0.7, 0.7, 0.7))) // the window's background colour
        .add_resource(Scoreboard { score: 0 })
        .add_resource(GameState::Starting)
        .add_startup_system(setup.system())
        .add_startup_system(start_game_system.system())
        .add_system(start_pause_game_system.system())
        .add_system(ball_collision_system.system())
        .add_stage_after("update", "do_things")
        .add_system_to_stage("do_things", ball_movement_system.system())
        .add_system_to_stage("do_things", ball_rotation_system.system())
        .add_system_to_stage("do_things", paddle_movement_system.system())
        .add_system_to_stage("do_things", scoreboard_system.system())
        .add_system_to_stage("do_things", fps_system.system())
        .add_system_to_stage("do_things", despawn_system.system())
        .add_system_to_stage("do_things", check_game_state_system.system())
        .add_system_to_stage("do_things", render_game_state_text_system.system())
        .add_system_to_stage("do_things", end_game_system.system())
        .run();
}

/// Determine whether two rectangles overlap during a frame.
///
/// The problem with Bevy's is that during a frame, one rectangle might be *very close*
/// to another rectangle, then the following frame, it has moved >50% of its "width"
/// into the rectangle, so this determines that the collision had approached from the
/// opposite direction. You can also have multiple collisions during a frame, and multiple
/// frame collisions. Additionally, it is possible to have both vertical and horizontal collisions
/// i.e. outside corner to outside corner, or outside corner to inside corner.
///
/// Instead, you should know its previous position in order to
/// determine the direction of collision, e.g. during previous frame, A's right edge was
/// left of B's left edge, therefore collision left. You should also interpolate the
/// collision location as well. The movement system should then handle this collision,
/// i.e. move X% towards the collision site, turn around, move (100-X)% away from the collision
/// site.
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

const DESPAWN_TIME: f32 = 5.0;

// #[derive(Default, Debug, Serialize, Deserialize)]
// pub struct ConfigData
// {
//     keyboard_key_bindings: HashMap<KeyCode, String>,
//     // mouse_button_binding: Option<HashMap<MouseButton, String>>,
//     // mouse_axis_binding: Option<HashMap<Axis, String>>,
// }

//T: unique component
struct Paddle {
    speed: f32,
}

//T: unique component
struct Ball {
    velocity: Vec3,
    rotation: f32,
    rotational_velocity: f32,
    collided: Option<(WillCollide, Collider)>,
    spin: Spin,
}

struct GameStateText;

#[derive(Eq, PartialEq)]
enum Spin {
    Clockwise,
    CounterCw,
}

//T: global resource
struct Scoreboard {
    score: usize,
}

struct Score;

const MAX_SCORE: usize = 20;

struct Framerate;

struct ToBeDespawned(f32);

struct Name(String);

struct DespawnOnEnd;

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

#[derive(PartialEq, Eq)]
enum GameState {
    Starting,
    Restarting,
    Playing,
    Paused,
    Win,
    Lose,
}

//T: main startup system
fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Add the game's entities to our world
    commands
        // cameras
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default())
        // scoreboard
        .spawn(TextComponents {
            text: Text {
                //relative to project directory, will panic if not found
                font: asset_server.load("assets/FiraSans-Bold.ttf").unwrap(),
                value: "Score:".to_string(),
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
                //relative to project directory, will panic if not found
                font: asset_server.load("assets/FiraSans-Bold.ttf").unwrap(),
                value: "FPS:".to_string(),
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
        // game state text
        .spawn(TextComponents {
            text: Text {
                font: asset_server.load("assets/FiraSans-Bold.ttf").unwrap(),
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
            material: wall_material,
            translation: Translation(Vec3::new(-bounds.x() / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(wall_thickness, bounds.y() + wall_thickness)),
            ..Default::default()
        })
        .with(Collider::OtherWall)
        .with(Name("Left wall".into()))
        // right
        .spawn(SpriteComponents {
            material: wall_material,
            translation: Translation(Vec3::new(bounds.x() / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(wall_thickness, bounds.y() + wall_thickness)),
            ..Default::default()
        })
        .with(Collider::OtherWall)
        .with(Name("Right wall".into()))
        // bottom
        .spawn(SpriteComponents {
            material: wall_material,
            translation: Translation(Vec3::new(0.0, -bounds.y() / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(bounds.x() + wall_thickness, wall_thickness)),
            ..Default::default()
        })
        .with(Collider::BottomWall)
        .with(Name("Bottom wall".into()))
        // top
        .spawn(SpriteComponents {
            material: wall_material,
            translation: Translation(Vec3::new(0.0, bounds.y() / 2.0, 0.0)),
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
) {
    if *game_state == GameState::Restarting {
        for (entity, _) in &mut despawn_query.iter() {
            commands.despawn(entity);
        }
        *game_state = GameState::Starting;
        scoreboard.score = 0;
        start_game_system(commands, materials);
    }
}

fn start_game_system(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands
        // paddle
        .spawn(SpriteComponents {
            material: materials.add(Color::rgb(0.8, 0.2, 0.2).into()),
            translation: Translation(Vec3::new(0.0, -215.0, 0.0)),
            sprite: Sprite::new(Vec2::new(120.0, 30.0)),
            ..Default::default()
        })
        .with(Paddle { speed: 500.0 })
        .with(Collider::Paddle)
        .with(DespawnOnEnd)
        .with(Name("Paddle".into()))
        // ball
        .spawn(SpriteComponents {
            material: materials.add(Color::rgb(0.8, 0.2, 0.2).into()),
            translation: Translation(Vec3::new(0.0, -50.0, 1.0)),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            rotation: Rotation::from_rotation_z(FRAC_PI_4), // 45 degrees
            ..Default::default()
        })
        .with(Ball {
            velocity: 400.0 * Vec3::new(0.5, -0.5, 0.0).normalize(),
            collided: None,
            rotation: FRAC_PI_4,
            rotational_velocity: 2.0 * PI, // radians per second
            spin: Spin::Clockwise,
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
            commands
                // brick
                .spawn(SpriteComponents {
                    material: materials.add(Color::rgb(0.2, 0.2, 0.8).into()),
                    sprite: Sprite::new(brick_size),
                    translation: Translation(brick_position),
                    draw: Draw {
                        is_transparent: true,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(Collider::Brick)
                .with(DespawnOnEnd)
                .with(Name(format!("Brick {}-{}", row, column).into()));
        }
    }
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

fn ball_rotation_system(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut query: Query<(&mut Ball, &mut Rotation)>,
) {
    if *game_state == GameState::Playing {
        for (mut ball, mut rotation) in &mut query.iter() {
            match ball.spin {
                Spin::Clockwise => {
                    ball.rotation -= ball.rotational_velocity * time.delta_seconds;
                }
                Spin::CounterCw => {
                    ball.rotation += ball.rotational_velocity * time.delta_seconds;
                }
            }
            ball.rotation = wrap(ball.rotation, 0.0, PI);
            *rotation = Rotation::from_rotation_z(ball.rotation);
        }
    }
}

fn paddle_movement_system(
    time: Res<Time>,
    game_state: Res<GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Paddle, &mut Translation)>,
) {
    if *game_state == GameState::Playing {
        for (paddle, mut translation) in &mut query.iter() {
            let mut direction = 0.0;
            if keyboard_input.pressed(KeyCode::Left) {
                direction -= 1.0;
            }

            if keyboard_input.pressed(KeyCode::Right) {
                direction += 1.0;
            }

            *translation.0.x_mut() += time.delta_seconds * direction * paddle.speed;

            // bound the paddle within the walls
            // *translation.0.x_mut() = f32::max(-380.0, f32::min(380.0, translation.0.x()));
            // paddle with is 120, half of which is 60
            *translation.0.x_mut() = f32::max(-440.0, f32::min(440.0, translation.0.x()));
        }
    }
}

fn ball_movement_system(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut ball_query: Query<(&mut Ball, &mut Translation)>,
) {
    if *game_state == GameState::Playing {
        // clamp the timestep to stop the ball from escaping when the game starts
        let delta_seconds = f32::min(0.2, time.delta_seconds);
        
        for (mut ball, mut translation) in &mut ball_query.iter() {
            // either we continue in the current direction with current velocity
            // or we take two moves with flips, so we need a midpoint, flipx and flipy
            let handle_collision = match &ball.collided {
                None => None,
                Some((collision, collider)) => {
                    let start = translation.0;
                    let extrapolated = start + ball.velocity * delta_seconds;
                    // check if x is a collision first
                    let x_collided = collision.x.0 != CollisionX::None;
                    let y_collided = collision.y.0 != CollisionY::None;
                    let midpoint = if x_collided {
                        let x_collision_site = &collision.x.1;
                        let x_start = start.x();
                        let x_extrapolated = extrapolated.x();
                        (x_collision_site - x_start)/(x_extrapolated - x_start)
                    } else if y_collided {
                        let y_collision_site = &collision.y.1;
                        let y_start = start.y();
                        let y_extrapolated = extrapolated.y();
                        (y_collision_site - y_start)/(y_extrapolated - y_start)
                    } else {
                        0.0
                    };
                    let mut flip_x = false;
                    let mut flip_y = false;
                    if let Collider::Paddle = collider {
                        if collision.y.0 == CollisionY::Top && ball.velocity.y() < 0.0 {
                            if (ball.spin == Spin::CounterCw
                                && ball.velocity.x() > 0.0)
                                || (ball.spin == Spin::Clockwise
                                    && ball.velocity.x() < 0.0)
                            {
                                // *ball.velocity.x_mut() = -ball.velocity.x(); // FlipX
                                flip_x = true;
                            };
                            // *ball.velocity.y_mut() = -ball.velocity.y(); // FlipY
                            flip_y = true;
                        }
                    // }
                    } else {
                        // reflect the ball when it collides
                        // only reflect if the ball's velocity is going in the opposite direction of the collision
                        // reflect velocity on the x-axis if we hit something on the x-axis
                        if (collision.x.0 == CollisionX::Left && ball.velocity.x() > 0.0)
                            || (collision.x.0 == CollisionX::Right && ball.velocity.x() < 0.0)
                        {
                            // *ball.velocity.x_mut() = -ball.velocity.x(); // FlipX
                            flip_x = true;
                        }
                        // reflect velocity on the y-axis if we hit something on the y-axis
                        if (collision.y.0 == CollisionY::Bottom && ball.velocity.y() > 0.0)
                            || (collision.y.0 == CollisionY::Top && ball.velocity.y() < 0.0)
                        {
                            // *ball.velocity.y_mut() = -ball.velocity.y(); // FlipY
                            flip_y = true;
                        }
                    };
                    Some((midpoint, flip_x, flip_y))
                },
            };
            if let Some((midpoint, flip_x, flip_y)) = handle_collision {
                println!("bounce, before {:?}, {}, {}", translation.0, delta_seconds, midpoint);
                // half move
                translation.0 += ball.velocity * delta_seconds * midpoint;
                // flip velocities
                if flip_x {
                    *ball.velocity.x_mut() = -ball.velocity.x();
                }
                if flip_y {
                    *ball.velocity.y_mut() = -ball.velocity.y();
                }
                // println!("bounce, mid {:?}, {}, {}", translation.0, delta_seconds, midpoint);
                // finish the move
                translation.0 += ball.velocity * delta_seconds * (1.0 - midpoint);
                // println!("bounce, after {:?}, {}, {}", translation.0, delta_seconds, midpoint);
            } else {
                // println!("normal, before {:?}, {}", translation.0, delta_seconds);
                translation.0 += ball.velocity * delta_seconds;
                // println!("normal, after {:?}, {}", translation.0, delta_seconds);
            }
            ball.collided = None;
        }
    }
}

fn scoreboard_system(scoreboard: Res<Scoreboard>, mut query: Query<(&mut Text, &Score)>) {
    for (mut text, _score) in &mut query.iter() {
        text.value = format!("Score: {}", scoreboard.score);
    }
}

fn fps_system(time: Res<Time>, mut query: Query<(&mut Text, &Framerate)>) {
    for (mut text, _framerate) in &mut query.iter() {
        text.value = format!("FPS: {:.0}", 1.0 / time.delta_seconds);
    }
}

fn ball_collision_system(
    mut commands: Commands,
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut ball_query: Query<(&mut Ball, &Translation, &Sprite)>,
    mut collider_query: Query<(Entity, &Collider, &Translation, &Sprite, &Name)>,
) {
    if *game_state == GameState::Playing {
        for (mut ball, ball_translation, sprite) in &mut ball_query.iter() {
            let ball_size = sprite.size;

            // check collision with walls, bricks and paddles
            for (collider_entity, collider, translation, sprite, name) in &mut collider_query.iter()
            {
                if let Some(collision) = collide(
                    ball_translation.0,
                    ball_size,
                    translation.0,
                    sprite.size,
                    &ball.velocity,
                    time.delta_seconds,
                ) {
                    // println!(
                    //     "ball collided with {} ({:?}) at {:?}",
                    //     name.0, collider, collision
                    // );
                    if let Collider::Paddle = *collider {
                        if collision.y.0 == CollisionY::Top && ball.velocity.y() < 0.0 {
                            // println!("ball {}, {}", ball_translation.0.x(), translation.0.x());
                            ball.spin = if ball_translation.0.x() < translation.0.x() {
                                Spin::CounterCw
                            } else {
                                Spin::Clockwise
                            };
                            // if (ball_translation.0.x() < translation.0.x()
                            //     && ball.velocity.x() > 0.0)
                            //     || (ball_translation.0.x() > translation.0.x()
                            //         && ball.velocity.x() < 0.0)
                            // {
                            //     *ball.velocity.x_mut() = -ball.velocity.x();
                            // };
                            // *ball.velocity.y_mut() = -ball.velocity.y();
                        }
                    } else if let Collider::BottomWall = *collider {
                        *game_state = GameState::Lose;
                    } else {
                        // scorable colliders should be despawned and increment the scoreboard on collision
                        if let Collider::Brick = *collider {
                            commands.insert_one(collider_entity, ToBeDespawned(DESPAWN_TIME));
                            commands.remove_one::<Collider>(collider_entity);
                        }

                        // reflect the ball when it collides
                        // only reflect if the ball's velocity is going in the opposite direction of the collision
                        // reflect velocity on the x-axis if we hit something on the x-axis
                        // if (collision.x.0 == CollisionX::Left && ball.velocity.x() > 0.0)
                        //     || (collision.x.0 == CollisionX::Right && ball.velocity.x() < 0.0)
                        // {
                        //     *ball.velocity.x_mut() = -ball.velocity.x();
                        // }
                        // reflect velocity on the y-axis if we hit something on the y-axis
                        // if (collision.y.0 == CollisionY::Bottom && ball.velocity.y() > 0.0)
                        //     || (collision.y.0 == CollisionY::Top && ball.velocity.y() < 0.0)
                        // {
                        //     *ball.velocity.y_mut() = -ball.velocity.y();
                        // }
                    }

                    ball.collided = Some((collision, *collider));

                    // break;
                }
                // }
            }
        }
    }
}

fn despawn_system(
    mut commands: Commands,
    time: Res<Time>,
    game_state: Res<GameState>,
    mut scoreboard: ResMut<Scoreboard>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut despawn_query: Query<(Entity, &mut ToBeDespawned, &Handle<ColorMaterial>)>,
) {
    if *game_state != GameState::Paused {
        for (entity, mut despawn_time, material_handle) in &mut despawn_query.iter() {
            if despawn_time.0 == DESPAWN_TIME {
                scoreboard.score += 1;
            }
            despawn_time.0 -= time.delta_seconds;
            if despawn_time.0 > 0.0 {
                let material = materials.get_mut(&material_handle).unwrap();
                // material.color = Color::rgb(1.0, 1.0, 1.0) * Vec3::new(0.7, 0.7, 0.7).lerp(Vec3::new(0.2, 0.2, 0.8), despawn_time.0 / DESPAWN_TIME);
                material.color = Color::rgba(0.2, 0.2, 0.8, despawn_time.0 / DESPAWN_TIME);
            } else {
                commands.despawn(entity);
            }
        }
    }
}

fn render_game_state_text_system(
    game_state: Res<GameState>,
    mut query: Query<(&mut Text, &GameStateText)>,
) {
    for (mut text, _game_state_text) in &mut query.iter() {
        text.value = match *game_state {
            GameState::Starting => "Press Space to start",
            GameState::Playing => "",
            GameState::Restarting => "",
            GameState::Paused => "PAUSED",
            GameState::Win => "YOU WIN! :D",
            GameState::Lose => "YOU LOSE :(",
        }
        .into();
    }
}

fn check_game_state_system(mut game_state: ResMut<GameState>, scoreboard: Res<Scoreboard>) {
    if scoreboard.score == MAX_SCORE && *game_state == GameState::Playing {
        *game_state = GameState::Win;
    }
}
