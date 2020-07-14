// use ggez::event::{KeyCode, KeyMods};
use ggez::{event, graphics, nalgebra as na, Context, GameResult};
use ggez::nalgebra::{Point2};



struct Ball {
    pos: Point2<f32>,
}

impl Ball {
    fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Point2::new(x, y)
        }
    }

    fn update_pos(&mut self, x: f32, y: f32) {
        self.pos = Point2::new(x, y)
    }

    fn get_cords(& self) -> (f32,f32) {
        if self.pos.len() != 2 {
            panic!("bad things")
        }
        unsafe {
            let x = self.pos.get_unchecked(0) + 0.0;
            let y = self.pos.get_unchecked(1) + 0.0;
            (x,y)
        }
    }
}

struct MainState {
    ball: Ball,
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let s = MainState { ball: Ball::new(0.0, 300.0) };
        Ok(s)
    }
}

// enum Direction {
//     Up,
//     Down,
//     Left,
//     Right,
// }

// impl Direction {
//     pub fn from_keycode(key: KeyCode) -> Option<Direction> {
//         match key {
//             KeyCode::Up => Some(Direction::Up),
//             KeyCode::Down => Some(Direction::Down),
//             KeyCode::Left => Some(Direction::Left),
//             KeyCode::Right => Some(Direction::Right),
//             _ => None,
//         }
//     }
// }


impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        let cords =  self.ball.get_cords();
        let new_x =  cords.0 % 800.0 + 1.0;
        self.ball.update_pos(new_x, cords.1.clone());
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            na::Point2::new(0.0, 0.0),
            100.0,
            2.0,
            graphics::WHITE,
        )?;
        // graphics::draw(ctx, &circle, (na::Point2::new(self.pos_x, 380.0),))?;
        graphics::draw(ctx, &circle, (self.ball.pos,))?;


        graphics::present(ctx)?;
        Ok(())
    }

    // fn key_down_event(
    //     &mut self,
    //     _ctx: &mut Context,
    //     keycode: KeyCode,
    //     _keymod: KeyMods,
    //     _repeat: bool,
    // ) {
    //     if let Some(dir) = Direction::from_keycode(keycode) {
    //         // If it succeeds, we check if a new direction has already been set
    //         // and make sure the new direction is different then `snake.dir`
    //         if self.snake.dir != self.snake.last_update_dir && dir.inverse() != self.snake.dir {
    //             self.snake.next_dir = Some(dir);
    //         } else if dir.inverse() != self.snake.last_update_dir {
    //             // If no new direction has been set and the direction is not the inverse
    //             // of the `last_update_dir`, then set the snake's new direction to be the
    //             // direction the user pressed.
    //             self.snake.dir = dir;
    //         }
    //     }
    // }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new()?;
    event::run(ctx, event_loop, state)
}
