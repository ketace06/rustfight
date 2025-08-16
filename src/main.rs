use bevy::animation::AnimationPlayer;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;
use std::path::Path;
use std::time::Duration;

#[derive(Component)]
struct GameEntity;

#[derive(Component)]
struct HitboxVisual {
    owner: Entity,
}

#[derive(Component, Clone)]
struct Player {
    id: usize,
}

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec3);

#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

#[derive(Component)]
struct AttackCooldowns {
    punch: Timer,
    kick: Timer,
    jump_kick: Timer,
}

#[derive(Component)]
struct Hitbox {
    owner: Entity,
    damage: f32,
}

#[derive(Component)]
struct Grounded(bool);

#[derive(Component)]
struct HealthBar;

#[derive(Component)]
struct UiHealthBar;

#[derive(Component, Deref, DerefMut)]
struct Lifetime(Timer);

#[derive(Component)]
struct PlayerHealthBar {
    player_id: usize,
}

#[derive(Resource)]
struct Players {
    player1: Entity,
    player2: Entity,
}

#[derive(Component)]
struct MainCamera;

const ARENA_WIDTH: f32 = 800.;
const ARENA_DEPTH: f32 = 400.;
const PLAYER_SPEED: f32 = 300.;
const GRAVITY: f32 = -1200.;
const JUMP_VEL: f32 = 320.;
const PUNCH_RANGE: f32 = 80.;
const PUNCH_DAMAGE: f32 = 8.;
const PUNCH_COOLDOWN: f32 = 0.45;
const KICK_RANGE: f32 = 100.;
const KICK_DAMAGE: f32 = 14.;
const KICK_COOLDOWN: f32 = 0.8;
const HITBOX_DURATION: f32 = 0.12;

#[derive(Component, Default)]
struct SlideState {
    sliding: bool,
    direction: Vec3,
    timer: Timer,
}

#[derive(Resource, Default)]
struct PlayerInputMemory {
    last_press: std::collections::HashMap<(usize, KeyCode), f32>,
}

const SLIDE_THRESHOLD: f32 = 0.25;
const SLIDE_DURATION: f32 = 0.25;
const SLIDE_SPEED: f32 = 500.0;

#[derive(Component, Default)]
struct AttackAnimationState {
    active: bool,
    kind: Option<AttackKind>,
    timer: Timer,
    start_pos: Vec3,
    end_pos: Vec3,
}

#[derive(Clone, Copy)]
enum AttackKind {
    Punch,
    Kick,
}

const PUNCH_ANIM_DURATION: f32 = 0.18;
const KICK_ANIM_DURATION: f32 = 0.25;
const PUNCH_OFFSET: Vec3 = Vec3::new(40.0, 60.0, 0.0);
const KICK_OFFSET: Vec3 = Vec3::new(40.0, 20.0, 0.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Mini Tekken - Bevy".to_string(),
                resolution: (ARENA_WIDTH, 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.08)))
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(
            Update,
            (
                player_input_system,
                attack_animation_system,
                slide_timer_system,
                apply_velocity_system,
                ground_and_gravity_system,
                attack_cooldowns_system,
                spawn_hitbox_system,
                face_each_other_system,
                hitbox_lifetime_system,
                hitbox_damage_system,
                update_healthbar_system,
                update_healthbar_ui_system,
                camera_follow_system,
                play_animation_system,
                draw_hitboxes_system,
            ),
        )
        .insert_resource(PlayerInputMemory::default())
        .run();
}

fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 1,
            ..default()
        },
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::None,
            ..default()
        },
        ..default()
    });

    let mut player1_style = Style::default();
    player1_style.position_type = PositionType::Absolute;
    player1_style.top = Val::Px(10.0);
    player1_style.left = Val::Px(10.0);
    player1_style.width = Val::Px(220.0);
    player1_style.height = Val::Px(28.0);

    let mut healthbar_bg_style = Style::default();
    healthbar_bg_style.width = Val::Percent(100.0);
    healthbar_bg_style.height = Val::Percent(100.0);

    let mut healthbar_fg_style = Style::default();
    healthbar_fg_style.width = Val::Percent(100.0);
    healthbar_fg_style.height = Val::Percent(100.0);

    commands
        .spawn(NodeBundle {
            style: player1_style.clone(),
            background_color: BackgroundColor(Color::NONE),
            ..default()
        })
        .insert(PlayerHealthBar { player_id: 1 })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: healthbar_bg_style.clone(),
                    background_color: BackgroundColor(Color::rgba(0.1, 0.1, 0.1, 0.6)),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: healthbar_fg_style.clone(),
                            background_color: BackgroundColor(Color::GREEN),
                            ..default()
                        })
                        .insert(UiHealthBar);
                });
        });

    let mut player2_style = Style::default();
    player2_style.position_type = PositionType::Absolute;
    player2_style.top = Val::Px(10.0);
    player2_style.right = Val::Px(10.0);
    player2_style.width = Val::Px(220.0);
    player2_style.height = Val::Px(28.0);

    commands
        .spawn(NodeBundle {
            style: player2_style,
            background_color: BackgroundColor(Color::NONE),
            ..default()
        })
        .insert(PlayerHealthBar { player_id: 2 })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: healthbar_bg_style,
                    background_color: BackgroundColor(Color::rgba(0.1, 0.1, 0.1, 0.6)),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: healthbar_fg_style,
                            background_color: BackgroundColor(Color::GREEN),
                            ..default()
                        })
                        .insert(UiHealthBar);
                });
        });
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 0,
                ..default()
            },
            transform: Transform::from_xyz(ARENA_WIDTH / 2., 300., 600.)
                .looking_at(Vec3::new(ARENA_WIDTH / 2., 0., 0.), Vec3::Y),
            ..default()
        },
        MainCamera,
        GameEntity,
    ));

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: false,
                illuminance: 20000.0,
                ..default()
            },
            transform: Transform::from_rotation(Quat::from_euler(
                EulerRot::XYZ,
                -std::f32::consts::FRAC_PI_4,
                std::f32::consts::FRAC_PI_6,
                0.0,
            )),
            ..default()
        },
        GameEntity,
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: ARENA_WIDTH,
                subdivisions: 1,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.08, 0.6, 0.2),
                perceptual_roughness: 1.0,
                ..default()
            }),
            transform: Transform::from_xyz(ARENA_WIDTH / 2., 0.0, 0.0),
            ..default()
        },
        GameEntity,
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 80.0 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.2, 0.2),
                ..default()
            }),
            transform: Transform::from_xyz(ARENA_WIDTH / 2., 40.0, 0.0),
            ..default()
        },
        GameEntity,
    ));

    let has_model = Path::new("assets/fighter.glb").exists();
    let glb_handle: Handle<Scene> = if has_model {
        asset_server.load("fighter.glb#Scene0")
    } else {
        Handle::default()
    };

    let spawn_player = |commands: &mut Commands,
                        meshes: &mut Assets<Mesh>,
                        materials: &mut Assets<StandardMaterial>,
                        id: usize,
                        x: f32| {
        let entity = if has_model {
            commands
                .spawn((
                    SceneBundle {
                        scene: glb_handle.clone(),
                        transform: Transform::from_xyz(x, 0., 0.).with_scale(Vec3::splat(80.0)),
                        ..default()
                    },
                    Player { id },
                    Velocity(Vec3::ZERO),
                    Grounded(true),
                    Health {
                        current: 100.,
                        max: 100.,
                    },
                    AttackCooldowns {
                        punch: Timer::from_seconds(0.0, TimerMode::Once),
                        kick: Timer::from_seconds(0.0, TimerMode::Once),
                        jump_kick: Timer::from_seconds(0.0, TimerMode::Once),
                    },
                    GameEntity,
                ))
                .id()
        } else {
            commands
                .spawn((
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 80.0 })),
                        material: materials.add(StandardMaterial {
                            base_color: Color::rgb(0.8, 0.2, 0.2),
                            ..default()
                        }),
                        transform: Transform::from_xyz(x, 40., 0.),
                        ..default()
                    },
                    Player { id },
                    Velocity(Vec3::ZERO),
                    Grounded(true),
                    Health {
                        current: 100.,
                        max: 100.,
                    },
                    AttackCooldowns {
                        punch: Timer::from_seconds(0.0, TimerMode::Once),
                        kick: Timer::from_seconds(0.0, TimerMode::Once),
                        jump_kick: Timer::from_seconds(0.0, TimerMode::Once),
                    },
                    GameEntity,
                ))
                .id()
        };

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::new(60.0, 6.0, 1.0))),
                    material: materials.add(StandardMaterial {
                        base_color: Color::GREEN,
                        emissive: Color::GREEN,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 160.0, 0.0),
                    ..default()
                },
                HealthBar,
                GameEntity,
            ));
        });
        entity
    };

    let player1 = spawn_player(&mut commands, &mut meshes, &mut materials, 1, 100.);
    let player2 = spawn_player(&mut commands, &mut meshes, &mut materials, 2, 700.);

    commands
        .entity(player1)
        .insert(SlideState::default())
        .insert(AttackAnimationState::default());
    commands
        .entity(player2)
        .insert(SlideState::default())
        .insert(AttackAnimationState::default());

    commands.insert_resource(Players { player1, player2 });
}

fn face_each_other_system(
    mut param_set: ParamSet<(Query<(Entity, &Player, &Transform)>, Query<&mut Transform>)>,
) {
    let players: Vec<(Entity, Player, Transform)> = {
        let query = param_set.p0();
        query
            .iter()
            .map(|(entity, player, transform)| (entity, player.clone(), transform.clone()))
            .collect()
    };
    if players.len() != 2 {
        return;
    }
    let (entity1, _p1, t1) = &players[0];
    let (entity2, _p2, t2) = &players[1];
    let mut binding = param_set.p1();
    if let Ok([mut transform1, mut transform2]) = binding.get_many_mut([*entity1, *entity2]) {
        let dir_1_to_2 = (t2.translation - t1.translation).normalize_or_zero();
        let dir_2_to_1 = -dir_1_to_2;
        transform1.rotation =
            Quat::from_rotation_arc(Vec3::X, Vec3::new(dir_1_to_2.x, 0.0, dir_1_to_2.z));
        transform2.rotation =
            Quat::from_rotation_arc(Vec3::X, Vec3::new(dir_2_to_1.x, 0.0, dir_2_to_1.z));
    }
}

fn player_input_system(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(
        &Player,
        &mut Velocity,
        &mut Grounded,
        &mut AttackCooldowns,
        &mut SlideState,
        &mut AttackAnimationState,
        &Transform,
    )>,
    camera_query: Query<&Transform, With<MainCamera>>,
    mut input_memory: ResMut<PlayerInputMemory>,
    time: Res<Time>,
) {
    let camera_transform = if let Ok(t) = camera_query.get_single() {
        t
    } else {
        return;
    };

    let now = time.elapsed_seconds();

    let right = camera_transform.rotation * Vec3::X;
    let right = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();
    let forward = camera_transform.rotation * Vec3::NEG_Z;
    let forward = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();

    for (player, mut vel, mut grounded, mut cooldowns, mut slide, mut attack_anim, transform) in
        query.iter_mut()
    {
        let mut dir = Vec3::ZERO;
        let mut slide_dir = Vec3::ZERO;
        let mut slide_key: Option<KeyCode> = None;

        let (left, right_key, up, down) = match player.id {
            1 => (KeyCode::A, KeyCode::D, KeyCode::W, KeyCode::S),
            2 => (KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down),
            _ => continue,
        };

        let mut input_dir = Vec3::ZERO;
        if keyboard.pressed(left) {
            input_dir -= right;
            slide_dir = -right;
            slide_key = Some(left);
        }
        if keyboard.pressed(right_key) {
            input_dir += right;
            slide_dir = right;
            slide_key = Some(right_key);
        }
        if keyboard.pressed(up) {
            input_dir += forward;
            slide_dir = forward;
            slide_key = Some(up);
        }
        if keyboard.pressed(down) {
            input_dir -= forward;
            slide_dir = -forward;
            slide_key = Some(down);
        }
        if input_dir.length_squared() > 0. {
            dir = input_dir.normalize();
        }

        if let Some(key) = slide_key {
            if keyboard.just_pressed(key) {
                let last = input_memory
                    .last_press
                    .get(&(player.id, key))
                    .copied()
                    .unwrap_or(-100.0);
                if now - last < SLIDE_THRESHOLD && !slide.sliding && grounded.0 {
                    slide.sliding = true;
                    slide.direction = slide_dir.normalize_or_zero();
                    slide.timer = Timer::from_seconds(SLIDE_DURATION, TimerMode::Once);
                }
                input_memory.last_press.insert((player.id, key), now);
            }
        }

        if slide.sliding {
            vel.x = slide.direction.x * SLIDE_SPEED;
            vel.z = slide.direction.z * SLIDE_SPEED;
        } else {
            vel.x = dir.x * PLAYER_SPEED;
            vel.z = dir.z * PLAYER_SPEED;
        }

        if player.id == 1 {
            if cooldowns.punch.finished() && keyboard.just_pressed(KeyCode::J) {
                cooldowns.punch.reset();
                cooldowns
                    .punch
                    .set_duration(Duration::from_secs_f32(PUNCH_COOLDOWN));
                cooldowns.punch.unpause();
                attack_anim.active = true;
                attack_anim.kind = Some(AttackKind::Punch);
                attack_anim.timer = Timer::from_seconds(PUNCH_ANIM_DURATION, TimerMode::Once);
                attack_anim.start_pos = transform.translation;
                attack_anim.end_pos = transform.translation + transform.rotation * PUNCH_OFFSET;
            }
            if cooldowns.kick.finished() && keyboard.just_pressed(KeyCode::K) {
                cooldowns.kick.reset();
                cooldowns
                    .kick
                    .set_duration(Duration::from_secs_f32(KICK_COOLDOWN));
                cooldowns.kick.unpause();
                attack_anim.active = true;
                attack_anim.kind = Some(AttackKind::Kick);
                attack_anim.timer = Timer::from_seconds(KICK_ANIM_DURATION, TimerMode::Once);
                attack_anim.start_pos = transform.translation;
                attack_anim.end_pos = transform.translation + transform.rotation * KICK_OFFSET;
            }
            if cooldowns.jump_kick.finished() && keyboard.just_pressed(KeyCode::Space) {
                cooldowns.jump_kick.reset();
                cooldowns.jump_kick.unpause();
            }
        } else if player.id == 2 {
            if cooldowns.punch.finished() && keyboard.just_pressed(KeyCode::Numpad1) {
                cooldowns.punch.reset();
                cooldowns
                    .punch
                    .set_duration(Duration::from_secs_f32(PUNCH_COOLDOWN));
                cooldowns.punch.unpause();
                attack_anim.active = true;
                attack_anim.kind = Some(AttackKind::Punch);
                attack_anim.timer = Timer::from_seconds(PUNCH_ANIM_DURATION, TimerMode::Once);
                attack_anim.start_pos = transform.translation;
                attack_anim.end_pos = transform.translation + transform.rotation * PUNCH_OFFSET;
            }
            if cooldowns.kick.finished() && keyboard.just_pressed(KeyCode::Numpad2) {
                cooldowns.kick.reset();
                cooldowns
                    .kick
                    .set_duration(Duration::from_secs_f32(KICK_COOLDOWN));
                cooldowns.kick.unpause();
                attack_anim.active = true;
                attack_anim.kind = Some(AttackKind::Kick);
                attack_anim.timer = Timer::from_seconds(KICK_ANIM_DURATION, TimerMode::Once);
                attack_anim.start_pos = transform.translation;
                attack_anim.end_pos = transform.translation + transform.rotation * KICK_OFFSET;
            }
        }
    }
}

fn attack_animation_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AttackAnimationState, &Transform, &Player)>,
    time: Res<Time>,
) {
    for (entity, mut anim, transform, player) in query.iter_mut() {
        if anim.active {
            anim.timer.tick(time.delta());
            let t = 1.0 - anim.timer.percent_left();
            let mut hitbox_pos = transform.translation;
            if let Some(kind) = anim.kind {
                match kind {
                    AttackKind::Punch => {
                        hitbox_pos = anim.start_pos.lerp(anim.end_pos, t);
                    }
                    AttackKind::Kick => {
                        hitbox_pos = anim.start_pos.lerp(anim.end_pos, t);
                    }
                }
            }
            if anim.timer.just_finished() {
                let (range, damage, duration) = match anim.kind {
                    Some(AttackKind::Punch) => (PUNCH_RANGE, PUNCH_DAMAGE, PUNCH_ANIM_DURATION),
                    Some(AttackKind::Kick) => (KICK_RANGE, KICK_DAMAGE, KICK_ANIM_DURATION),
                    _ => (0.0, 0.0, 0.0),
                };
                commands.spawn((
                    Transform::from_translation(hitbox_pos),
                    GlobalTransform::default(),
                    Velocity(Vec3::ZERO),
                    Hitbox {
                        owner: entity,
                        damage,
                    },
                    Lifetime(Timer::from_seconds(duration, TimerMode::Once)),
                ));
                anim.active = false;
                anim.kind = None;
            }
        }
    }
}

fn slide_timer_system(mut query: Query<&mut SlideState>, time: Res<Time>) {
    for mut slide in query.iter_mut() {
        if slide.sliding {
            slide.timer.tick(time.delta());
            if slide.timer.finished() {
                slide.sliding = false;
            }
        }
    }
}

fn apply_velocity_system(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += **velocity * time.delta_seconds();
        transform.translation.x = transform.translation.x.clamp(0.0, ARENA_WIDTH);
        transform.translation.z = transform
            .translation
            .z
            .clamp(-ARENA_DEPTH / 2.0, ARENA_DEPTH / 2.0);
    }
}

fn ground_and_gravity_system(
    mut query: Query<(&mut Velocity, &mut Grounded, &mut Transform)>,
    time: Res<Time>,
) {
    for (mut velocity, mut grounded, mut transform) in query.iter_mut() {
        velocity.y += GRAVITY * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
        if transform.translation.y <= 0.0 {
            grounded.0 = true;
            transform.translation.y = 0.0;
            velocity.y = 0.0;
        }
    }
}

fn attack_cooldowns_system(mut query: Query<&mut AttackCooldowns>, time: Res<Time>) {
    for mut cooldowns in query.iter_mut() {
        cooldowns.punch.tick(time.delta());
        cooldowns.kick.tick(time.delta());
        cooldowns.jump_kick.tick(time.delta());
    }
}

fn spawn_hitbox_system(
    mut commands: Commands,
    query: Query<(Entity, &Player, &Transform, &AttackCooldowns)>,
) {
    for (entity, _player, transform, cooldowns) in query.iter() {
        if !cooldowns.punch.finished() && cooldowns.punch.elapsed_secs() < PUNCH_COOLDOWN {
            spawn_hitbox(&mut commands, entity, transform, PUNCH_RANGE, PUNCH_DAMAGE);
        }
        if !cooldowns.kick.finished() && cooldowns.kick.elapsed_secs() < KICK_COOLDOWN {
            spawn_hitbox(&mut commands, entity, transform, KICK_RANGE, KICK_DAMAGE);
        }
    }
}

fn spawn_hitbox(
    commands: &mut Commands,
    owner: Entity,
    player_transform: &Transform,
    range: f32,
    damage: f32,
) {
    let forward = player_transform.rotation * Vec3::X;
    let hitbox_pos = player_transform.translation + forward * range;
    commands.spawn((
        Transform::from_translation(hitbox_pos),
        GlobalTransform::default(),
        Velocity(Vec3::ZERO),
        Hitbox { owner, damage },
        Lifetime(Timer::from_seconds(HITBOX_DURATION, TimerMode::Once)),
    ));
}

fn hitbox_lifetime_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.tick(time.delta());
        if lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn hitbox_damage_system(
    mut commands: Commands,
    hitboxes: Query<(Entity, &Hitbox, &Transform)>,
    mut players: Query<(Entity, &mut Health, &Transform, Option<&Player>)>,
) {
    for (hitbox_entity, hitbox, hitbox_transform) in hitboxes.iter() {
        for (player_entity, mut health, player_transform, _maybe_player) in players.iter_mut() {
            if player_entity == hitbox.owner {
                continue;
            }
            let distance = player_transform
                .translation
                .distance(hitbox_transform.translation);
            if distance < 60.0 {
                health.current -= hitbox.damage;
                if health.current < 0.0 {
                    health.current = 0.0;
                }
                commands.entity(hitbox_entity).despawn();
                break;
            }
        }
    }
}

fn update_healthbar_system(
    query: Query<(Entity, &Health, &Children)>,
    mut healthbars: Query<&mut Transform, With<HealthBar>>,
) {
    for (_entity, health, children) in query.iter() {
        for child in children.iter() {
            if let Ok(mut healthbar_transform) = healthbars.get_mut(*child) {
                let ratio = (health.current / health.max).clamp(0.0, 1.0);
                healthbar_transform.scale.x = ratio;
            }
        }
    }
}

fn update_healthbar_ui_system(
    players: Query<(&Player, &Health)>,
    bars: Query<(&PlayerHealthBar, &Children)>,
    mut fills: Query<&mut Style, With<UiHealthBar>>,
) {
    use std::collections::HashMap;
    let mut health_map = HashMap::new();
    for (player, health) in players.iter() {
        health_map.insert(player.id, (health.current / health.max).clamp(0.0, 1.0));
    }
    for (bar, children) in bars.iter() {
        if let Some(ratio) = health_map.get(&bar.player_id) {
            for &child in children.iter() {
                if let Ok(mut style) = fills.get_mut(child) {
                    style.width = Val::Percent(ratio * 100.0);
                }
            }
        }
    }
}

fn camera_follow_system(
    time: Res<Time>,
    mut params: ParamSet<(
        Query<&mut Transform, With<MainCamera>>,
        Query<&Transform, With<Player>>,
    )>,
) {
    let players: Vec<Vec3> = params.p1().iter().map(|t| t.translation).collect();
    if players.len() < 2 {
        return;
    }
    let p1 = players[0];
    let p2 = players[1];
    let midpoint = (p1 + p2) / 2.0;
    let mut fight_axis = p2 - p1;
    fight_axis.y = 0.0;
    if fight_axis.length_squared() < 0.001 {
        fight_axis = Vec3::X;
    } else {
        fight_axis = fight_axis.normalize();
    }
    let base_distance = 500.0;
    let zoom_factor = 15.0;
    let fixed_height = 200.0;
    let distance_between = (p2 - p1).length();
    let zoomed_distance = base_distance + zoom_factor * distance_between.sqrt();
    let cam_dir = Vec3::new(-fight_axis.z, 0.0, fight_axis.x);
    let target_pos = midpoint + cam_dir * zoomed_distance + Vec3::Y * fixed_height;
    for mut cam_transform in params.p0().iter_mut() {
        cam_transform.translation = cam_transform
            .translation
            .lerp(target_pos, 5.0 * time.delta_seconds());
        cam_transform.look_at(midpoint, Vec3::Y);
    }
}

#[derive(Component, Default)]
struct AnimationStarted(bool);

fn play_animation_system(
    mut query: Query<(&Children, &mut AnimationStarted), With<Player>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    clips: Res<Assets<AnimationClip>>,
) {
    for (children, mut started) in query.iter_mut() {
        if started.0 {
            continue;
        }
        for &child in children.iter() {
            if let Ok(mut anim_player) = animation_players.get_mut(child) {
                if let Some((handle_id, _)) = clips.iter().next() {
                    let handle = Handle::<AnimationClip>::weak(handle_id);
                    anim_player.play(handle).repeat();
                    started.0 = true;
                    break;
                }
            }
        }
    }
}

fn draw_hitboxes_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    hitboxes: Query<(Entity, &Transform), With<Hitbox>>,
    visuals: Query<(Entity, &HitboxVisual)>,
) {
    use std::collections::HashSet;
    for (vis_entity, vis) in visuals.iter() {
        if hitboxes.get(vis.owner).is_err() {
            commands.entity(vis_entity).despawn();
        }
    }
    let mut owners_with_visual = HashSet::new();
    for (_vis_entity, vis) in visuals.iter() {
        owners_with_visual.insert(vis.owner);
    }
    for (hb_entity, transform) in hitboxes.iter() {
        if owners_with_visual.contains(&hb_entity) {
            continue;
        }
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(60.0, 60.0, 20.0))),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgba(1.0, 0.0, 0.0, 0.5),
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                }),
                transform: *transform,
                ..default()
            },
            HitboxVisual { owner: hb_entity },
        ));
    }
}
