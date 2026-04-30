#![allow(unused_variables)]

use std::f32::consts::{PI, TAU};

use enemies::*;
use include_dir::Dir;
use level::*;
use macroquad::prelude::*;
use player::*;
use utils::*;

use crate::{
    assets::ASSETS,
    background::Background,
    bosses::Boss,
    particles::{Particle, update_particles},
    projectiles::Projectile,
    utils::{AnimationMethods, HP_MATERIAL, create_camera},
};
mod assets;
mod background;
mod bosses;
mod enemies;
mod level;
mod particles;
mod player;
mod projectiles;

mod utils;
const SCREEN_SIZE: (f32, f32) = (384., 216.);
struct CameraHolder {
    camera: Camera2D,
    pos: Vec2,
    desired_y: f32,
}
impl CameraHolder {
    pub fn is_obj_in_view(&self, obj: Vec2) -> bool {
        return !(obj.x < self.pos.x - SCREEN_SIZE.0 / 2.
            || obj.x > self.pos.x + SCREEN_SIZE.0 / 2.);
    }
    fn update(&mut self, player: &Player, level: &Level) {
        let mut pos = player.pos;
        let mut c = 0;
        if player.grounded {
            while c < 10
                && let Some(tile) = level.get_tile(pos)
                && !tile.ground
            {
                c += 1;
                pos = pos
                    + vec2(
                        TILE_SIZE * if player.previous_flipped { -1.0 } else { 1.0 },
                        TILE_SIZE,
                    );
            }
            if let Some((tile)) = level.get_tile(pos) {
                self.desired_y = player.pos.y + (pos.y - player.pos.y) * 0.2;
                draw_rectangle(pos.x, pos.y, TILE_SIZE, TILE_SIZE, RED);
            }
        }
        const Y_SPEED: f32 = 0.5;
        const Y_BELOW_THRESHOLD: f32 = 30.0;
        if player.pos.y + SCREEN_SIZE.1 / 2.0 + Y_BELOW_THRESHOLD > self.pos.y + SCREEN_SIZE.1 {
            self.pos.y = player.pos.y;
            self.desired_y = self.pos.y
        }

        if (self.pos.y - self.desired_y).abs() > 5.0 {
            self.pos.y += (self.desired_y - self.pos.y).signum() * Y_SPEED;
        }
        const FORESIGHT: f32 = 7.5;
        const MAX_FORESIGHT: f32 = FORESIGHT * 3.5;
        const X_SPEED: f32 = 1.2;

        let desired_x = player.pos.x + player.velocity.x * FORESIGHT;
        if (desired_x - player.pos.x).abs() > (self.pos.x - player.pos.x).abs() {
            let speed = X_SPEED
                * if (desired_x - player.pos.x).signum() != (self.pos.x - player.pos.x).signum() {
                    1.5
                } else if desired_x.abs() < self.pos.x.abs() {
                    0.7
                } else {
                    1.0
                };
            if (self.pos.x - desired_x).abs() > 10.0 {
                let update = (desired_x - self.pos.x).signum() * speed;
                self.pos.x = (self.pos.x + update)
                    .clamp(player.pos.x - MAX_FORESIGHT, player.pos.x + MAX_FORESIGHT);
            }
        }
        //self.pos = self.pos.round();
    }
    fn calculate_y_up(&mut self, player: &Player) {
        // self.desired_y = player.pos.y - 17.5;
    }
}

pub struct Game {
    win: bool,
    die: bool,
    scale_factor: f32,
    map: Level,
    backgrounds: Background,
    player: Player,
    camera_holder: CameraHolder,
    enemies: Vec<NewEnemy>,
    boss: Option<Box<dyn Boss>>,
    pickups: Vec<Pickup>,
    projectiles: Vec<Projectile>,
    particles: Vec<Particle>,
    map_animations: Vec<MapAnimation>,
    bullets: Vec<Bullet>,
    gun_animation: f32,
    ammo_hud_camera: Camera2D,
}
impl Game {
    fn draw_hud(&mut self, frame_time: f32) {
        set_default_camera();
        self.draw_ammo(frame_time);

        for x in 0..5 {
            let x = x as f32;
            let black_x = (self.player.hp as f32 - x * 20.0) / 20.0;

            HP_MATERIAL.set_uniform("black_x", black_x);
            gl_use_material(&HP_MATERIAL);
            draw_texture_ex(
                &ASSETS.lemon,
                (7.0 + x * (4.0 + ASSETS.lemon.width())) * self.scale_factor,
                10.0 * self.scale_factor,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(ASSETS.lemon.size() * self.scale_factor),
                    ..Default::default()
                },
            );
        }
        gl_use_default_material();
        set_camera(&self.camera_holder.camera);
    }

    fn new(level_data: (Level, SpecialData)) -> Self {
        println!("wa");

        let (map, special_data) = level_data;
        let mut enemies = Vec::new();
        for (preset, pos) in special_data.enemies.iter() {
            enemies.push(NewEnemy::new(*preset, *pos, &map))
        }

        let render_target = render_target(64, 64);
        render_target.texture.set_filter(FilterMode::Nearest);

        let mut render_target_cam = Camera2D::from_display_rect(Rect::new(0., 0., 64., 64.));
        render_target_cam.render_target = Some(render_target);

        let wahoo = Self {
            ammo_hud_camera: render_target_cam,
            gun_animation: -0.0,
            bullets: Vec::new(),
            boss: if let Some((boss, tile)) = special_data.boss {
                Some(boss.to_boss(tile, &map))
            } else {
                None
            },
            die: false,
            win: false,
            pickups: special_data.pickups,
            backgrounds: Background::new(
                vec2(
                    map.chunks.iter().map(|f| f.pos.0).max().unwrap() as f32
                        - map.chunks.iter().map(|f| f.pos.0).min().unwrap() as f32,
                    map.chunks.iter().map(|f| f.pos.1).max().unwrap() as f32
                        - map.chunks.iter().map(|f| f.pos.1).min().unwrap() as f32,
                ) * TILE_SIZE,
            ),
            scale_factor: 1.0,
            particles: Vec::new(),
            projectiles: Vec::new(),
            map_animations: special_data.map_animations,
            enemies,
            map,
            player: Player::new(special_data.spawn_location),
            camera_holder: CameraHolder {
                desired_y: special_data.spawn_location.y,
                camera: create_camera(vec2(SCREEN_SIZE.0, SCREEN_SIZE.1)),
                pos: special_data.spawn_location,
            },
        };
        return wahoo;
    }
    fn draw_camera(&mut self) {
        set_default_camera();
        clear_background(BLACK);

        draw_texture_ex(
            &self
                .camera_holder
                .camera
                .render_target
                .as_ref()
                .unwrap()
                .texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    SCREEN_SIZE.0 * self.scale_factor,
                    SCREEN_SIZE.1 * self.scale_factor,
                )),
                ..Default::default()
            },
        );
    }
    fn death(&mut self) {
        if let Some(death) = self.player.death {
            draw_rectangle(
                0.0,
                0.0,
                2000.0,
                2000.0,
                Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.2,
                },
            );
            let animation = ASSETS.death_animations.get(match death.0 {
                DeathCause::Acid => "acid",
                DeathCause::Energy => "energy",
                DeathCause::Default => "default",
                DeathCause::Explode => "explode",
            });
            if death.1 > animation.get_duration() {
                self.die = true;
            } else {
                animation.play_with_clock(
                    self.player.pos - ASSETS.death_animations.size() / 2.0 + self.player.size / 2.0,
                    death.1,
                    Some(DrawTextureParams {
                        flip_x: self.player.previous_flipped,
                        ..Default::default()
                    }),
                );
            }
        }
    }
    fn draw_ammo(&mut self, frame_time: f32) {
        self.gun_animation -= frame_time;

        set_camera(&self.ammo_hud_camera);
        let texture = &ASSETS.gun_inside;

        draw_texture(texture, 0., 0., WHITE);
        let center_gun = vec2(texture.width() / 2., texture.height() / 2.);
        const MAX_AMMO: u8 = 6;
        for i in 0..MAX_AMMO {
            let angle = 2. * PI / MAX_AMMO as f32 * i as f32;
            let y = 22. * angle.sin();
            let x = 22. * angle.cos();
            let bullet_center = vec2(center_gun.x + x, center_gun.y + y);
            if i < self.player.ammo {
                draw_texture_ex(
                    &ASSETS.bullet_in_gun,
                    bullet_center.x - ASSETS.bullet_in_gun.width() / 2.,
                    bullet_center.y - ASSETS.bullet_in_gun.height() / 2.,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(ASSETS.bullet_in_gun.size()),
                        ..Default::default()
                    },
                );
            } else {
                draw_circle(bullet_center.x, bullet_center.y, 5., BLACK);
            }
        }
        set_default_camera();
        const ROT_OFFSET: f32 = PI / 3.;
        let mut rotation =
            2. * PI / MAX_AMMO as f32 * self.player.ammo as f32 - PI / 2. - ROT_OFFSET;
        if self.gun_animation.is_sign_positive() {
            rotation = (2. * PI / MAX_AMMO as f32 * self.player.ammo as f32 - PI / 2. - ROT_OFFSET)
                .lerp(
                    2. * PI / MAX_AMMO as f32 * (self.player.ammo as f32 + 1.)
                        - PI / 2.
                        - ROT_OFFSET,
                    self.gun_animation / GUN_ANIMATION_LENGHT,
                );
        }
        draw_texture_ex(
            &self.ammo_hud_camera.render_target.as_ref().unwrap().texture,
            0.,
            screen_height() - ASSETS.gun_inside.height() * self.scale_factor,
            WHITE,
            DrawTextureParams {
                dest_size: Some(ASSETS.gun_inside.size() * self.scale_factor),
                rotation,
                ..Default::default()
            },
        );
    }
    async fn update(&mut self, frame_time: f32) {
        clear_background(BLACK);
        self.backgrounds.update(frame_time);
        if let Some(boss) = &mut self.boss {
            boss.update(
                &self.map,
                &mut self.enemies,
                &mut self.projectiles,
                frame_time,
                &self.player,
                &mut self.particles,
            );
        }
        self.map.draw_level();
        self.bullets.retain_mut(|bullet| {
            let should_live = bullet.update(frame_time, &self.camera_holder, &self.map);
            if !should_live {
                self.particles.push(Particle::preset(
                    particles::Particles::BulletSpark,
                    bullet.pos,
                ));
            }
            return should_live;
        });

        self.projectiles.retain_mut(|projectile| {
            projectile.update(&mut self.player, frame_time, &self.map, &mut self.particles)
        });
        for animation in self.map_animations.iter_mut() {
            animation.update(frame_time);
        }
        if !self.win && !self.die {
            self.player.update(
                &mut self.map,
                &mut self.projectiles,
                &mut self.enemies,
                &mut self.particles,
                frame_time,
                &mut self.camera_holder,
                &mut self.bullets,
                &mut self.gun_animation,
            );
            self.camera_holder.update(&self.player, &self.map);
        }
        #[cfg(debug_assertions)]
        {
            if is_key_down(KeyCode::B) {
                dbg!(self.player.pos);
            }
            if is_key_down(KeyCode::Minus) {
                dbg!(self.camera_holder.pos);
            }
        }
        self.camera_holder.camera.target = self.camera_holder.pos;

        update_pickups(self);
        update_enemies(
            &mut self.enemies,
            &mut self.player,
            &mut self.map,
            &mut self.projectiles,
            frame_time,
            &mut self.bullets,
            &mut self.particles,
        );
        // update_particle_generators(&mut self.map.tiles, &mut self.particles, frame_time);
        update_particles(&mut self.particles, frame_time);
        self.death();
    }
}

enum GameState {
    Normal(Game),
    MainMenu,
}
struct GameManger {
    gamestate: GameState,
    level_index: usize,
    levels: Vec<(Level, SpecialData)>,
    clock: f32,
    new_game: bool,
    next_level: bool,
}
impl GameManger {
    fn new_game(&mut self) {
        dbg!(self.level_index, self.levels.len());
        self.gamestate = GameState::Normal(Game::new(self.levels[self.level_index].clone()));
        dbg!("WAAAAAAAAAAAAAA");
    }
    fn new() -> Self {
        let levels_dir = include_dir::include_dir!("./assets/maps");
        let mut levels = Vec::new();
        for level in levels_dir.entries().iter() {
            let contents = level.as_file().unwrap().contents_utf8().unwrap();
            dbg!(level.path());
            levels.push(load_tilemap(
                contents,
                include_str!("../assets/tileset.tsx"),
            ));
        }
        Self {
            new_game: false,
            next_level: false,
            clock: 0.0,
            gamestate: GameState::MainMenu,
            levels,
            level_index: 0,
        }
    }
    fn die_screen(&mut self) {
        set_default_camera();
        let button = &ASSETS.play_again;
        let size = button.size() / 2.0
            * (screen_width() / button.width()).min(screen_height() / button.height());
        let pos = vec2(
            (screen_width() - size.x) / 2.0,
            (screen_height() - size.y) / 2.0,
        );
        draw_texture_ex(
            button,
            pos.x,
            pos.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
        if is_mouse_button_pressed(MouseButton::Left)
            && mouse_position().0 > pos.x
            && mouse_position().0 < pos.x + size.x
            && mouse_position().1 > pos.y
            && mouse_position().1 < pos.y + size.y
        {
            todo!("");
        }
    }
    async fn update(&mut self) {
        let mut frame_time = get_frame_time().min(1. / 60.0);

        if self.next_level {
            self.level_index += 1;
            if self.level_index <= self.levels.len() - 1 {
                self.new_game();
                self.new_game = false;
                self.next_level = false;
            } else {
                todo!("win fr")
            }
        } else if self.new_game {
            self.new_game();
            self.new_game = false;
            self.next_level = false;
        }
        match &mut self.gamestate {
            GameState::MainMenu => {
                let scale_factor = (screen_width() / ASSETS.main_menu.width())
                    .min(screen_height() / ASSETS.main_menu.height());
                ASSETS.main_menu.draw(
                    Vec2::ZERO,
                    Some(DrawTextureParams {
                        dest_size: Some(ASSETS.main_menu.size() * scale_factor),
                        ..Default::default()
                    }),
                );

                let buttons = [
                    (vec2(1750., 700.), vec2(2385., 900.)),
                    (vec2(1750., 1130.), vec2(2385., 1410.)),
                ];
                if is_mouse_button_pressed(MouseButton::Left) {
                    dbg!(mouse_pos() / scale_factor);
                    for (index, (top_corner, lower_corner)) in buttons.iter().enumerate() {
                        let pressed_button = check_collision_rectangle_collision(
                            (
                                vec2(mouse_position().0, mouse_position().1) / scale_factor,
                                Vec2::ZERO,
                            ),
                            (*top_corner, *lower_corner - *top_corner),
                        );
                        dbg!(*top_corner * scale_factor);
                        if pressed_button {
                            if index == 0 {
                                self.new_game = true;
                            } else {
                                panic!()
                            }
                        }
                    }
                }
            }
            GameState::Normal(game) => {
                let scale_factor = (screen_width() / SCREEN_SIZE.0)
                    .min(screen_height() / SCREEN_SIZE.1)
                    .floor();
                game.scale_factor = scale_factor;
                if game.win {
                    let mut black_bars: bool = false;

                    let transition_length = 2.0;

                    let pos = vec2(
                        game.player.pos.x
                            - if game.player.previous_flipped {
                                (ASSETS.jetpacker.get("idle").size().x
                                    + ASSETS.win_animation.size().x)
                                    / 2.0
                            } else {
                                0.0
                            },
                        game.player.pos.y
                            - (ASSETS.win_animation.size().y
                                - ASSETS.jetpacker.get("idle").size().y),
                    );
                    self.clock += frame_time;
                    if self.clock > ASSETS.win_animation.get_duration() + transition_length {
                        self.next_level = true;
                    } else if self.clock > ASSETS.win_animation.get_duration() {
                        draw_texture_ex(
                            &ASSETS.win_animation.0.last().unwrap().0,
                            pos.x,
                            pos.y,
                            WHITE,
                            DrawTextureParams {
                                flip_x: game.player.previous_flipped,
                                ..Default::default()
                            },
                        );
                        black_bars = true;
                    } else {
                        ASSETS.win_animation.play_with_clock(
                            vec2(pos.x, pos.y),
                            self.clock,
                            Some(DrawTextureParams {
                                flip_x: game.player.previous_flipped,
                                ..Default::default()
                            }),
                        );
                    }
                    if black_bars {
                        draw_rectangle(
                            0.0,
                            0.0,
                            screen_width(),
                            (self.clock - ASSETS.win_animation.get_duration()) / transition_length
                                * screen_height()
                                / 2.0,
                            BLACK,
                        );
                        draw_rectangle(
                            0.0,
                            screen_height(),
                            screen_width(),
                            -(self.clock - ASSETS.win_animation.get_duration()) / transition_length
                                * screen_height()
                                / 2.0,
                            BLACK,
                        );
                    }
                }

                game.draw_camera();
                game.draw_ammo(frame_time);

                game.draw_hud(frame_time);
                set_camera(&game.camera_holder.camera);

                if let Some(speed) = DEBUG_FLAGS.speed {
                    frame_time *= speed;
                }
                game.update(frame_time).await;
            }
        }
    }
}
#[macroquad::main("krusbar")]
async fn main() {
    let mut game = GameManger::new();
    rand::srand((get_time() * 1000.0) as u64);
    loop {
        game.update().await;
        dbg!("da");
        next_frame().await;
        dbg!("ga");
    }
}
