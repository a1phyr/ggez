//! An Asteroids-ish example game to show off ggez.
//! The idea is that this game is simple but still
//! non-trivial enough to be interesting.

extern crate ggez;
extern crate rand;
extern crate sdl2;

use std::path;
use sdl2::pixels::Color;
use sdl2::event::*;
use sdl2::keyboard::Keycode;

use ggez::audio;
use ggez::conf;
use ggez::game::{Game, GameState};
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::timer;
use std::time::Duration;

#[derive(Debug, Copy, Clone)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        Vec2 {
            x: x,
            y: y,
        }
    }

    /// Create a unit vector representing the
    /// given angle (in radians)
    fn from_angle(angle: f64) -> Self {
        let vx = angle.sin();
        let vy = angle.cos();
        Vec2 {
            x: vx,
            y: vy
        }
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        Self::new(0., 0.)
    }
}

#[derive(Debug)]
enum ActorType {
    Player,
    Rock,
    Shot,
}

#[derive(Debug)]
struct Actor {
    tag: ActorType,
    pos: Vec2,
    facing: f64,
    velocity: Vec2,
}

struct Assets {
    player_image: graphics::Image,
    shot_image: graphics::Image,
    rock_image: graphics::Image,
}



fn create_player() -> Actor {
    Actor {
        tag: ActorType::Player,
        pos: Vec2::default(),
        facing: 0.,
        velocity: Vec2::default(),
    }
}

fn create_rock() -> Actor {
    Actor {
        tag: ActorType::Rock,
        pos: Vec2::default(),
        facing: 0.,
        velocity: Vec2::default(),
    }
}

fn create_shot() -> Actor {
    Actor {
        tag: ActorType::Shot,
        pos: Vec2::default(),
        facing: 0.,
        velocity: Vec2::default(),
    }
}

fn update_position(actor: &mut Actor, dt: f64) {
    let dx = dt * actor.velocity.x;
    let dy = dt * actor.velocity.y;
    actor.pos.x += dx;
    actor.pos.y += dy;
}


const SHOT_SPEED: f64 = 200.0;


// Acceleration in pixels per second, more or less. 
const PLAYER_THRUST: f64 = 100.0;
// Rotation in radians per second.
const PLAYER_TURN_RATE: f64 = 3.05;
// Seconds between shots
const PLAYER_SHOT_TIME: f64 = 0.5;


fn player_handle_input(actor: &mut Actor, input: &InputState, dt: f64) {
    actor.facing += dt * PLAYER_TURN_RATE * input.xaxis;

    if input.yaxis > 0.0 {
        player_thrust(actor, dt);
    }
}

fn player_thrust(actor: &mut Actor, dt: f64) {
    let direction_vector = Vec2::from_angle(actor.facing);
    let vx = PLAYER_THRUST * direction_vector.x;
    let vy = PLAYER_THRUST * direction_vector.y;
    actor.velocity.x += dt * vx;
    actor.velocity.y += dt * vy;
}



// Translates the world coordinate system, which
// has Y pointing up and the origin at the center,
// to the screen coordinate system, which has Y
// pointing downward and the origin at the top-left,

fn world_to_screen_coords(state: &MainState, point: &Vec2) -> Vec2 {
    let width = state.screen_width as f64;
    let height = state.screen_height as f64;
    let x = point.x + width/2.0;
    let y = height - (point.y + height/2.0);
    Vec2{ x: x, y: y }
}


impl Assets {
    fn new(ctx: &mut Context) -> GameResult<Assets> {
        let player_image_path = path::Path::new("player.png");
        let player_image = try!(graphics::Image::new(ctx, player_image_path));
        let shot_image_path = path::Path::new("shot.png");
        let shot_image = try!(graphics::Image::new(ctx, shot_image_path));
        let rock_image_path = path::Path::new("rock.png");
        let rock_image = try!(graphics::Image::new(ctx, rock_image_path));
        Ok(Assets {
            player_image: player_image,
            shot_image: shot_image,
            rock_image: rock_image,
        })
    }
}

#[derive(Debug)]
struct InputState {
    xaxis: f64,
    yaxis: f64,
    fire: bool,
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            xaxis: 0.0,
            yaxis: 0.0,
            fire: false,
        }
    }
}

struct MainState {
    player: Actor,
    shots: Vec<Actor>,
    rocks: Vec<Actor>,
    score: u32,
    assets: Assets,
    screen_width: u32,
    screen_height: u32,
    input: InputState,
    player_shot_timeout: f64,
}



impl MainState {

    fn fire_player_shot(&mut self) {
        self.player_shot_timeout = PLAYER_SHOT_TIME;

        let player = &self.player;
        let mut shot = create_shot();
        shot.pos = player.pos;
        shot.facing = player.facing;
        let direction = Vec2::from_angle(shot.facing);
        shot.velocity.x = SHOT_SPEED * direction.x;
        shot.velocity.y = SHOT_SPEED * direction.y;

        self.shots.push(shot);
    }


    fn draw_actor(&self, ctx: &mut Context, actor: &Actor) -> GameResult<()>  {
        
        let pos = world_to_screen_coords(self, &actor.pos);
        let px = pos.x as i32;
        let py = pos.y as i32;
        let destrect = graphics::Rect::new(px, py, 32, 32);
        let actor_center = graphics::Point::new(16, 16);
        let image = self.actor_image(actor);
        graphics::draw_ex(
            ctx,
            image, None, Some(destrect),
            actor.facing.to_degrees(), Some(actor_center),
            false, false)

    }

    fn actor_image(&self, actor: &Actor) -> &graphics::Image {
        match actor.tag {
            ActorType::Player => &self.assets.player_image,
            ActorType::Rock   => &self.assets.rock_image,
            ActorType::Shot   => &self.assets.shot_image,
        }
    }
}

impl<'a> GameState for MainState {
    fn load(ctx: &mut Context, conf: &conf::Conf) -> GameResult<MainState> {
        ctx.print_sound_stats();
        ctx.print_resource_stats();
        graphics::set_background_color(ctx, Color::RGB(0, 0, 0));

        let assets = try!(Assets::new(ctx));
        
        let s = MainState {
            player: create_player(),
            shots: Vec::new(),
            rocks: Vec::new(),
            score: 0,
            assets: assets,
            screen_width: conf.window_width,
            screen_height: conf.window_height,
            input: InputState::default(),
            player_shot_timeout: 0.0,
        };

        Ok(s)
    }

    fn update(&mut self, _ctx: &mut Context, dt: Duration) -> GameResult<()> {
        //println!("Player: {:?}", self.player);
        let seconds = timer::duration_to_f64(dt);
        player_handle_input(&mut self.player, &mut self.input, seconds);
        self.player_shot_timeout -= seconds;
        if self.input.fire && self.player_shot_timeout < 0.0 {
            self.fire_player_shot();
            println!("Bang!");
        }
        update_position(&mut self.player, seconds);
        for s in &mut self.shots {
            update_position(s, seconds);
        }
        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        let p = &self.player;
        self.draw_actor(ctx, p);
        for s in &self.shots {
            self.draw_actor(ctx, &s);
        }

        graphics::present(ctx);
        timer::sleep_until_next_frame(ctx, 60);
        // ctx.quit() is broken :-(
        //ctx.quit();
        Ok(())
    }

    fn key_down_event(&mut self, evt: Event) {
        match evt {
            Event::KeyDown { keycode, .. } => {
                match keycode {
                    Some(Keycode::Up) => {
                        println!("Thrusting");
                        self.input.yaxis = 1.0;
                    },
                    Some(Keycode::Left) => {
                        self.input.xaxis = -1.0;
                    },
                    Some(Keycode::Right) => {
                        self.input.xaxis = 1.0;
                    },
                    Some(Keycode::Space) => {
                        self.input.fire = true;
                    }
                    _ => () // Do nothing
                }
            },
            _ => panic!("Should never happen"),
        }
    }


    fn key_up_event(&mut self, evt: Event) {
        match evt {
            Event::KeyUp { keycode, .. } => {
                match keycode {
                    Some(Keycode::Up) => {
                        println!("Thrusting stopped");
                        self.input.yaxis = 0.0;
                    },
                    Some(Keycode::Left) => {
                        self.input.xaxis = 0.0;
                    },
                    Some(Keycode::Right) => {
                        self.input.xaxis = 0.0;
                    },
                    Some(Keycode::Space) => {
                        self.input.fire = false;
                    }
                    _ => () // Do nothing
                }
            },
            _ => panic!("Should never happen"),
        }
    }
}

pub fn main() {
    let mut c = conf::Conf::new("Astroblasto!");
    c.window_title = "Astroblasto!".to_string();
    c.window_width = 640;
    c.window_height = 480;
    let game: GameResult<Game<MainState>> = Game::new(c);
    match game {
        Err(e) => {
            println!("Could not load game!");
            println!("Error: {:?}", e);
        }
        Ok(mut game) => {
            let result = game.run();
            if let Err(e) = result {
                println!("Error encountered running game: {:?}", e);
            } else {
                println!("Game exited cleanly.");
            }
        }
    }
}
