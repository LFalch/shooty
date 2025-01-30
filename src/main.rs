use std::f32::consts::TAU;
use std::iter;

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event;
use ggez::graphics::{self, Color, DrawParam, Image};
use ggez::input::keyboard::KeyCode;
use ggez::mint::Point2;
use ggez::{Context, GameResult};
use ggez::glam::*;

use self_compare::SliceCompareExt;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Obj {
    pos: Vec2,
    vel: Vec2,
    rot: f32,
    rot_v: f32,
}

impl Obj {
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            vel: Vec2::ZERO,
            rot: 0.,
            rot_v: 0.,
        }
    }
    pub const fn from(pos: Vec2, vel: Vec2, rot: f32) -> Self {
        Self {
            pos,
            vel,
            rot,
            rot_v: 0.,
        }
    }
    pub const fn with(x: f32, y: f32, vx: f32, vy: f32, rot: f32, rot_v: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            vel: Vec2::new(vx, vy),
            rot,
            rot_v,
        }
    }
    fn draw_param(&self) -> DrawParam {
        DrawParam::new()
            .offset(Point2::from(Vec2::new(0.5, 0.5)))
            .scale(Vec2::new(0.5, 0.5))
            .dest(self.pos)
            .rotation(self.rot)
    }

    pub const fn bullet(self, ttl: f32) -> Bullet {
        Bullet {
            obj: self,
            ttl,
        }
    }
    pub fn pushed(self, dx: f32, dy: f32, dvx: f32, dvy: f32) -> Self {
        Self {
            pos: self.pos + Vec2::new(dx, dy),
            vel: self.vel + Vec2::new(dvx, dvy),
            rot: self.rot + rand::random_range(0. .. TAU),
            rot_v: self.rot_v + rand::random_range(-3. .. 3.),
        }
    }
    fn resolve(&mut self, other: &mut Self) {
        let a = self;
        let b = other;

        const W: f32 = 32.;
        let d = a.pos - b.pos;
        let dist_sq = d.length_squared();
        if dist_sq < W * W {
            let dv = (a.vel - b.vel).dot(d) / dist_sq * d;
            a.vel -= dv;
            b.vel += dv;

            let dist = dist_sq.sqrt();
            let dp = 0.5 * (W / dist - 1.) * d;
            a.pos += dp;
            b.pos -= dp;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Bullet {
    obj: Obj,
    ttl: f32,
}

impl Bullet {
    fn draw_param(&self) -> DrawParam {
        self.obj.draw_param()
            .color(opacity(self.ttl.min(5.) * 2.))
    }
}

struct MainState {
    ship: Obj,
    bullets: Vec<Bullet>,
    crates: Vec<Obj>,
    splinters: Vec<Bullet>,

    ship_img: Image,
    crate_img: Image,
    bullet_img: Image,
    splinter_img: Image,

    crate_spawn_time: f32,

    bounce_edge: bool,
}

impl MainState {
    fn new(ctx: &Context) -> GameResult<MainState> {
        let s = MainState {
            crate_spawn_time: -CRATE_SPAWN_RATE * 20.,
            ship: Obj::new(0.5 * WIDTH, 0.5 * HEIGHT),
            bullets: Vec::new(),
            crates: Vec::new(),
            splinters: Vec::new(),
            ship_img: Image::from_path(ctx, "/ship.png").unwrap(),
            crate_img: Image::from_path(ctx, "/crate.png").unwrap(),
            bullet_img: Image::from_path(ctx, "/bullet.png").unwrap(),
            splinter_img: Image::from_path(ctx, "/splinter.png").unwrap(),
            bounce_edge: false,
        };
        Ok(s)
    }
}

const CRATE_LIMIT: usize = 200;

const ROT_SPEED: f32 = 5.53;
const ACCELERATION: f32 = 150.;
const CRATE_SPAWN_RATE: f32 = 0.65;
const BULLET_SPEED: f32 = 470.;
const CRATE_BULLET_COLLIDE_DIST: f32 = 16.+8.;

pub fn angle_to_vec(angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    Vec2::new(cos, sin)
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if self.crate_spawn_time <= 0. {
            let x = rand::random_range(0. .. WIDTH);
            let y = rand::random_range(0. .. HEIGHT);
            
            if (self.ship.pos-Vec2::new(x, y)).length_squared() >= 160.*160. {
                self.crate_spawn_time += CRATE_SPAWN_RATE;
                let obj = Obj::with(
                    x, y,
                    rand::random_range(-150. .. 150.),
                    rand::random_range(-150. .. 150.),
                    rand::random_range(0. .. TAU),
                    rand::random_range(-3. .. 3.),
                );
                self.crates.push(obj);
            }

        }

        const DELTA: f32 = 1./60.;
        if ctx.time.check_update_time(60) {
            if self.crates.len() < CRATE_LIMIT {
                self.crate_spawn_time -= DELTA;
            }

            let mut deads = Vec::new();
            for (i, bullet) in self.bullets.iter_mut().enumerate() {
                bullet.ttl -= DELTA;
                if bullet.ttl <= 0. {
                    deads.push(i);
                }
            }
            deads.drain(..).rev().for_each(|i| {self.bullets.remove(i);});
            for (i, bullet) in self.splinters.iter_mut().enumerate() {
                bullet.ttl -= DELTA;
                if bullet.ttl <= 0. {
                    deads.push(i);
                }
            }
            deads.into_iter().rev().for_each(|i| {self.splinters.remove(i);});

            if ctx.keyboard.is_key_just_pressed(KeyCode::Space) {
                let dir = angle_to_vec(self.ship.rot);
                let obj = Obj::from(self.ship.pos + dir * 20., self.ship.vel + dir * BULLET_SPEED, self.ship.rot);
                self.bullets.push(obj.bullet(rand::random_range(4.5 .. 6.2)));
            }
            if ctx.keyboard.is_key_just_pressed(KeyCode::C) {
                self.crate_spawn_time -= CRATE_SPAWN_RATE;
            }
            if ctx.keyboard.is_key_just_pressed(KeyCode::B) {
                self.bounce_edge = !self.bounce_edge;
            }

            if ctx.keyboard.is_key_pressed(KeyCode::A) {
                self.ship.rot -= ROT_SPEED * DELTA;
            }
            if ctx.keyboard.is_key_pressed(KeyCode::D) {
                self.ship.rot += ROT_SPEED * DELTA;
            }
            
            let mut wish_dir = Vec2::ZERO;
            if ctx.keyboard.is_key_pressed(KeyCode::W) {
                wish_dir.x += 1.;
            }
            if ctx.keyboard.is_key_pressed(KeyCode::S) {
                wish_dir.x -= 1.;
            }
            if ctx.keyboard.is_key_pressed(KeyCode::E) {
                wish_dir.y += 1.;
            }
            if ctx.keyboard.is_key_pressed(KeyCode::Q) {
                wish_dir.y -= 1.;
            }
            let wish_dir = wish_dir.normalize_or_zero();
            let dir = angle_to_vec(self.ship.rot);

            if ctx.keyboard.is_key_pressed(KeyCode::LShift) {
                let velocity_to_cancel = self.ship.vel - self.ship.vel.dot(dir).max(0.) * dir;
                self.ship.vel -= velocity_to_cancel.normalize_or_zero() * ACCELERATION * DELTA;
            }

            if wish_dir != Vec2::ZERO {
                let accel = dir.rotate(wish_dir) * ACCELERATION;
                self.ship.vel += accel * DELTA;
            }
        }

        let iter = iter::once(&mut self.ship)
            .chain(self.bullets.iter_mut().map(|b| &mut b.obj))
            .chain(&mut self.crates)
            .chain(self.splinters.iter_mut().map(|b| &mut b.obj));
        for obj in iter {
            obj.pos += obj.vel * DELTA;
            obj.rot += obj.rot_v * DELTA;
            if self.bounce_edge {
                const W: f32 = 16.;
                if obj.pos.x < W {
                    obj.vel.x = obj.vel.x.abs();
                } else if obj.pos.x >= (WIDTH-W) {
                    obj.vel.x = -obj.vel.x.abs();
                }
                if obj.pos.y < W {
                    obj.vel.y = obj.vel.y.abs();
                } else if obj.pos.y >= (HEIGHT-W) {
                    obj.vel.y = -obj.vel.y.abs();
                }
            } else {
                obj.pos.x = obj.pos.x.rem_euclid(WIDTH);
                obj.pos.y = obj.pos.y.rem_euclid(HEIGHT);
            }
        }

        let mut dead_bullets = Vec::new();
        for (b, bullet) in self.bullets.iter().enumerate() {
            let mut dead = None;
            for (c, crat) in self.crates.iter().enumerate() {
                let dist = bullet.obj.pos - crat.pos;
                if dist.length_squared() < CRATE_BULLET_COLLIDE_DIST * CRATE_BULLET_COLLIDE_DIST {
                    dead = Some(c);
                    break;
                }
            }
            if let Some(c) = dead {
                let mut crat = self.crates.remove(c);
                const D: f32 = 8.;
                const DV: f32 = 50.;
                crat.vel += 0.4 * bullet.obj.vel;
                self.splinters.push(crat.pushed(D, 0., DV, 0.).bullet(rand::random_range(1.6 .. 4.2)));
                self.splinters.push(crat.pushed(-D, 0., -DV, 0.).bullet(rand::random_range(1.6 .. 4.2)));
                self.splinters.push(crat.pushed(0., D, 0., DV).bullet(rand::random_range(1.6 .. 4.2)));
                self.splinters.push(crat.pushed(0., -D,0., -DV).bullet(rand::random_range(1.6 .. 4.2)));
                dead_bullets.push(b);
            }
        }
        dead_bullets.into_iter().rev().for_each(|i| {self.bullets.remove(i);});

        self.crates.compare_self_mut(Obj::resolve);
        self.crates.iter_mut().for_each(|c| self.ship.resolve(c));

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        canvas.draw(&self.ship_img, self.ship.draw_param());
        for bullet in &self.bullets {
            canvas.draw(&self.bullet_img, bullet.draw_param());
        }
        for craet in &self.crates {
            canvas.draw(&self.crate_img, craet.draw_param());
        }
        for splinter in &self.splinters {
            canvas.draw(&self.splinter_img, splinter.draw_param());
        }

        canvas.finish(ctx)?;
        Ok(())
    }
}

const fn opacity(a: f32) -> Color {
    Color {
        a,
        .. Color::WHITE
    }
}

const WIDTH: f32 = 1200.;
const HEIGHT: f32 = 900.;

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("shooty", "Falch")
        .window_setup(WindowSetup::default().title("Shooty"))
        .window_mode(WindowMode::default().dimensions(1200., 900.))
    ;
    let (ctx, event_loop) = cb.build()?;

    #[cfg(debug_assertions)]
    {
        // Add the workspace directory to the filesystem when running with cargo
        if let Ok(manifest_dir) = ::std::env::var("CARGO_MANIFEST_DIR") {
            let mut path = ::std::path::PathBuf::from(manifest_dir);
            path.push("resources");
            ctx.fs.mount(&path, true);
        }
    }

    let state = MainState::new(&ctx)?;
    event::run(ctx, event_loop, state)
}
