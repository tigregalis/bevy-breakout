use std::f32::consts::{FRAC_PI_4, PI};

use bevy::{prelude::*, render::pass::ClearColor};

use rand::random;

// use std::collections::HashMap;

// use serde::{Serialize, Deserialize};

/// An implementation of the classic game "Breakout"
fn main() {
    App::build()
        .add_default_plugins()
        .add_resource(ClearColor(BACKGROUND_COLOR)) // the window's background colour
        .add_resource(Scoreboard { score: 0 })
        .add_resource(GameState::Starting)
        .add_startup_system(setup.system())
        .add_startup_system(start_game_system.system())
        .add_system(start_pause_game_system.system())
        .add_system(ball_collision_system.system())
        .add_system(change_color_system.system())
        .add_system(ball_movement_system.system())
        .add_system(ball_rotation_system.system())
        .add_system(paddle_movement_system.system())
        .add_system(scoreboard_system.system())
        .add_system(fps_system.system())
        .add_system(despawn_system.system())
        .add_system(check_game_state_system.system())
        .add_system(render_game_state_text_system.system())
        .add_system(end_game_system.system())
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

const BACKGROUND_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const DESPAWN_TIME: f32 = 2.0;

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
            material: materials.add(Color::WHITE.into()),
            translation: Translation(Vec3::new(0.0, -50.0, 1.0)),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            rotation: Rotation::from_rotation_z(FRAC_PI_4), // 45 degrees
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
        .with(Name("Ball".into()))
        // ball
        .spawn(SpriteComponents {
            material: materials.add(Color::WHITE.into()),
            translation: Translation(Vec3::new(0.0, -30.0, 1.0)),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            rotation: Rotation::from_rotation_z(FRAC_PI_4), // 45 degrees
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
                    translation: Translation(brick_position),
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

            // bound the paddle partially within the walls
            // paddle width is 120, arena bounds are -380 to 380
            *translation.0.x_mut() = translation.0.x().max(-500.0).min(500.0);
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
            // or we take two moves with flips, so we need a midpoint, and a new direction
            let handle_collision = match &ball.collided {
                None => None,
                Some((collision, collider, _color)) => {
                    let start = translation.0;
                    let extrapolated = start + ball.velocity * delta_seconds;
                    // check if x is a collision first
                    let x_collided = collision.x.0 != CollisionX::None;
                    let y_collided = collision.y.0 != CollisionY::None;
                    let midpoint = if x_collided {
                        let x_collision_site = &collision.x.1;
                        let x_start = start.x();
                        let x_extrapolated = extrapolated.x();
                        (x_collision_site - x_start) / (x_extrapolated - x_start)
                    } else if y_collided {
                        let y_collision_site = &collision.y.1;
                        let y_start = start.y();
                        let y_extrapolated = extrapolated.y();
                        (y_collision_site - y_start) / (y_extrapolated - y_start)
                    } else {
                        0.0
                    };
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
                            magnitude = (magnitude + 50.0).max(100.0);
                            new_velocity *= magnitude / new_velocity.length();
                        } else if let Collider::OtherWall = collider {
                            magnitude = (magnitude - 25.0).max(100.0);
                            new_velocity *= magnitude / new_velocity.length();
                        }
                        new_velocity
                    };
                    Some((midpoint, new_velocity))
                }
            };
            if let Some((midpoint, new_velocity)) = handle_collision {
                // half move
                translation.0 += ball.velocity * delta_seconds * midpoint;
                // update velocity
                ball.velocity = new_velocity;
                ball.rotational_velocity = new_velocity.length() / 400.0 * 2.0 * PI;
                // finish the move
                translation.0 += ball.velocity * delta_seconds * (1.0 - midpoint);
            } else {
                translation.0 += ball.velocity * delta_seconds;
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
    mut ball_query: Query<(Entity, &mut Ball, &Translation, &Sprite)>,
    brick_query: Query<&mut Brick>,
    mut collider_query: Query<(
        Entity,
        &Collider,
        &Translation,
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
        for (ball_entity, mut ball, ball_translation, sprite) in &mut ball_query.iter() {
            let ball_size = sprite.size;

            // check collision with walls, bricks and paddles
            for (collider_entity, collider, translation, sprite, _name, material_handle) in
                &mut collider_query.iter()
            {
                if let Some(collision) = collide(
                    ball_translation.0,
                    ball_size,
                    translation.0,
                    sprite.size,
                    &ball.velocity,
                    time.delta_seconds,
                ) {
                    if let Collider::Paddle = *collider {
                        if collision.y.0 == CollisionY::Top && ball.velocity.y() < 0.0 {
                            ball.spin = if ball_translation.0.x() < translation.0.x() {
                                Spin::CounterCw
                            } else {
                                Spin::Clockwise
                            };
                            ball.last_paddle_offset = ball_translation.0.x() - translation.0.x();
                        }
                    } else if let Collider::BottomWall = *collider {
                        commands.insert_one(ball_entity, ToBeDespawned(DESPAWN_TIME));
                        commands.remove_one::<Ball>(ball_entity);
                        ball_count -= 1;
                        if ball_count <= 0 {
                            *game_state = GameState::Lose;
                            return;
                        }
                    } else {
                        // scorable colliders should be despawned and increment the scoreboard on collision
                        if let Collider::Brick = *collider {
                            commands.insert_one(collider_entity, ToBeDespawned(DESPAWN_TIME));
                            commands.remove_one::<Collider>(collider_entity);
                            if let Some(mut brick) =
                                brick_query.get_mut::<Brick>(collider_entity).ok()
                            {
                                brick.0 = false;
                            }
                            scoreboard.score += 1;
                        }
                    }

                    let color = materials.get(&material_handle).unwrap().color;
                    ball.collided = Some((collision, *collider, color));
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
                let ball_material = materials.get_mut(&ball_material_handle).unwrap();
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
                                    materials.get_mut(&paddle_material_handle).unwrap();
                                paddle_material.color = old_color.into();
                            }
                        }
                    }
                }
            }
        }
    }
}

fn despawn_system(
    mut commands: Commands,
    time: Res<Time>,
    game_state: Res<GameState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut despawn_query: Query<(Entity, &mut ToBeDespawned, &Handle<ColorMaterial>)>,
) {
    let rgb = Vec4::new(1.0, 1.0, 1.0, 0.0);
    if *game_state != GameState::Paused {
        for (entity, mut despawn_time, material_handle) in &mut despawn_query.iter() {
            if despawn_time.0 == DESPAWN_TIME {
                let material = materials.get_mut(&material_handle).unwrap();
                material.color = Color::WHITE;
            }
            despawn_time.0 -= time.delta_seconds;
            if despawn_time.0 > 0.0 {
                let material = materials.get_mut(&material_handle).unwrap();
                let color = color_to_vec4(material.color);
                material.color =
                    (color * rgb + Vec4::new(0.0, 0.0, 0.0, despawn_time.0 / DESPAWN_TIME)).into();
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

fn check_game_state_system(mut game_state: ResMut<GameState>, mut brick_query: Query<&Brick>) {
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
