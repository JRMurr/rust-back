use ggez::{
    event,
    event::{KeyCode, KeyMods},
    graphics, nalgebra as na,
    nalgebra::{Point2, Vector2},
    Context, GameResult,
};
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn from_keycode(key: KeyCode) -> Option<Direction> {
        match key {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None,
        }
    }
}

struct Ball {
    pos: Point2<f32>,
    vel: Vector2<f32>,
}

impl Ball {
    fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Point2::new(x, y),
            vel: Vector2::new(0.0, 0.0),
        }
    }

    fn update(&mut self, ctx: &Context) {
        self.pos += self.vel;

        // keep ball in window
        let screen_cords = graphics::screen_coordinates(ctx);
        let cords = self.get_cords();
        let new_x = cords.0.rem_euclid(screen_cords.w);
        let new_y = cords.1.rem_euclid(screen_cords.h);
        println!("({}, {})", new_x, new_y);
        self.set_pos(new_x, new_y);
    }

    fn draw(&self, ctx: &mut Context) -> GameResult {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            na::Point2::new(0.0, 0.0),
            100.0,
            2.0,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &circle, (self.pos,))?;
        Ok(())
    }

    fn set_pos(&mut self, x: f32, y: f32) {
        self.pos = Point2::new(x, y)
    }

    fn apply_force(&mut self, force: Vector2<f32>) {
        // self.vel += force;
        // self.vel = self.vel.normalize()
        self.vel = force * 5.0;
    }

    fn get_cords(&self) -> (f32, f32) {
        if self.pos.len() != 2 {
            panic!("bad things")
        }
        unsafe {
            let x = self.pos.get_unchecked(0) + 0.0;
            let y = self.pos.get_unchecked(1) + 0.0;
            (x, y)
        }
    }
}

struct MainState {
    ball: Ball,
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let s = MainState {
            ball: Ball::new(0.0, 300.0),
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.ball.update(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        self.ball.draw(ctx)?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        if let Some(dir) = Direction::from_keycode(keycode) {
            let force: Vector2<f32> = match dir {
                Direction::Up => Vector2::new(0.0, -1.0),
                Direction::Down => Vector2::new(0.0, 1.0),
                Direction::Right => Vector2::new(1.0, 0.0),
                Direction::Left => Vector2::new(-1.0, 0.0),
            };
            self.ball.apply_force(force);
        }
        if keycode == KeyCode::Escape {
            event::quit(ctx);
        }
    }
}

pub fn main() -> GameResult {
    use std::{env, path};
    let mut cb = ggez::ContextBuilder::new("Rust Back", "JRMurr");
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let path = path::PathBuf::from(manifest_dir).join("resources");
        println!("Adding 'resources' path {:?}", path);
        cb = cb.add_resource_path(path);
    }
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new()?;
    event::run(ctx, event_loop, state)
}
