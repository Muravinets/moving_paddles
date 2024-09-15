// First we'll import the crates we need for our game;
// in this case that is just `ggez` and `oorandom` (and `getrandom`
// to seed the RNG.)
use oorandom::Rand32;

// Next we need to actually `use` the pieces of ggez that we are going
// to need frequently.
use ggez::{
    event, graphics,
    input::keyboard::{KeyCode, KeyInput},
    Context, GameResult,
};

// We'll bring in some things from `std` to help us in the future.
use std::collections::VecDeque;

// The first thing we want to do is set up some constants that will help us out later.

// Here we define the size of our game board in terms of how many grid
// cells it will take up. We choose to make a 30 x 20 game board.
const GRID_SIZE: (i16, i16) = (30, 20);
// Now we define the pixel size of each tile, which we make 32x32 pixels.
const GRID_CELL_SIZE: (i16, i16) = (32, 32);

// Next we define how large we want our actual window to be by multiplying
// the components of our grid size by its corresponding pixel size.
const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32,
);

// Here we're defining how often we want our game to update. This will be
// important later so that we don't have our snake fly across the screen because
// it's moving a full tile every frame.
const DESIRED_FPS: u32 = 23;


/// Now we define a struct that will hold an entity's position on our game board
/// or grid which we defined above. We'll use signed integers because we only want
/// to store whole numbers, and we need them to be signed so that they work properly
/// with our modulus arithmetic later.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct GridPosition {
    x: i16,
    y: i16,
}

impl GridPosition {
    /// We make a standard helper function so that we can create a new `GridPosition`
    /// more easily.
    pub fn new(x: i16, y: i16) -> Self {
        GridPosition { x, y }
    }

    /// As well as a helper function that will give us a random `GridPosition` from
    /// `(0, 0)` to `(max_x, max_y)`
    pub fn random(rng: &mut Rand32, max_x: i16, max_y: i16) -> Self {
        // We can use `.into()` to convert from `(i16, i16)` to a `GridPosition` since
        // we implement `From<(i16, i16)>` for `GridPosition` below.
        (
            rng.rand_range(0..(max_x as u32)) as i16,
            rng.rand_range(0..(max_y as u32)) as i16,
        )
            .into()
    }

    /// We'll make another helper function that takes one grid position and returns a new one after
    /// making one move in the direction of `dir`.
    /// We use the [`rem_euclid()`](https://doc.rust-lang.org/std/primitive.i16.html#method.rem_euclid)
    /// API when crossing the top/left limits, as the standard remainder function (`%`) returns a
    /// negative value when the left operand is negative.
    /// Only the Up/Left cases require rem_euclid(); for consistency, it's used for all of them.
    pub fn new_from_move(pos: GridPosition, dir: Direction) -> Self {
        match dir {
            Direction::None => pos,
            Direction::Up => GridPosition::new(pos.x, (pos.y - 1).rem_euclid(GRID_SIZE.1)),
            Direction::Down => GridPosition::new(pos.x, (pos.y + 1).rem_euclid(GRID_SIZE.1)),
            Direction::Left => GridPosition::new((pos.x - 1).rem_euclid(GRID_SIZE.0), pos.y),
            Direction::Right => GridPosition::new((pos.x + 1).rem_euclid(GRID_SIZE.0), pos.y),
        }
    }
}

/// We implement the `From` trait, which in this case allows us to convert easily between
/// a `GridPosition` and a ggez `graphics::Rect` which fills that grid cell.
/// Now we can just call `.into()` on a `GridPosition` where we want a
/// `Rect` that represents that grid cell.
impl From<GridPosition> for graphics::Rect {
    fn from(pos: GridPosition) -> Self {
        graphics::Rect::new_i32(
            pos.x as i32 * GRID_CELL_SIZE.0 as i32,
            pos.y as i32 * GRID_CELL_SIZE.1 as i32,
            GRID_CELL_SIZE.0 as i32,
            GRID_CELL_SIZE.1 as i32,
        )
    }
}

/// And here we implement `From` again to allow us to easily convert between
/// `(i16, i16)` and a `GridPosition`.
impl From<(i16, i16)> for GridPosition {
    fn from(pos: (i16, i16)) -> Self {
        GridPosition { x: pos.0, y: pos.1 }
    }
}

/// Next we create an enum that will represent all the possible
/// directions that our snake could move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    None,
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// We create a helper function that will allow us to easily get the inverse
    /// of a `Direction` which we can use later to check if the player should be
    /// able to move the snake in a certain direction.
    pub fn inverse(self) -> Self {
        match self {
            Direction::None => Direction::None,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    /// We also create a helper function that will let us convert between a
    /// `ggez` `Keycode` and the `Direction` that it represents. Of course,
    /// not every keycode represents a direction, so we return `None` if this
    /// is the case.
    pub fn from_keycode(key: KeyCode) -> Option<Direction> {
        match key {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::W => Some(Direction::Up),
            KeyCode::S => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None,
        }
    }

    /// We also create a helper function that will let us convert between a
    /// `ggez` `Keycode` and the `Direction` that it represents. Of course,
    /// not every keycode represents a direction, so we return `None` if this
    /// is the case.
    pub fn from_keycode_player_number(key: KeyCode) -> Option<u8> {
        match key {
            KeyCode::Up => Some(2),
            KeyCode::Down => Some(2),
            KeyCode::Left => Some(2),
            KeyCode::Right => Some(2),
            KeyCode::W => Some(1),
            KeyCode::S => Some(1),
            _ => None,
        }
    }
}



/// This is mostly just a semantic abstraction over a `GridPosition` to represent
/// a segment of the snake. It could be useful to, say, have each segment contain its
/// own color or something similar. This is an exercise left up to the reader ;)
#[derive(Clone, Copy, Debug)]
struct Segment {
    pos: GridPosition,
}

impl Segment {
    pub fn new(pos: GridPosition) -> Self {
        Segment { pos }
    }
}

/// This is again an abstraction over a `GridPosition` that represents
/// a ball the paddle can beat. It can draw itself.
struct Ball {
    pos: GridPosition,
    /// Then we have the current direction the ball is moving. This is
    /// the direction it will move when `update` is called on it.
    dir: Direction,
}

impl Ball {
    pub fn new(pos: GridPosition) -> Self {
        Ball { 
            pos,
            dir: Direction::Left,
        }
    }

    /// The main update function for our ball which gets called every time
    /// we want to update the game state.
    fn update(&mut self, padle1: &Padle, padle2: &Padle) {
        self.pos = GridPosition::new_from_move(self.pos, self.dir);

        if padle1.meats_ball(self) {
            self.dir = self.dir.inverse();
            self.pos = GridPosition::new_from_move(self.pos, self.dir);
        }
        if padle2.meats_ball(self) {
            self.dir = self.dir.inverse();
            self.pos = GridPosition::new_from_move(self.pos, self.dir);
        }
    }

    /// Here is the first time we see what drawing looks like with ggez.
    /// We have a function that takes in a `&mut ggez::graphics::Canvas` which we use
    /// to do drawing.
    ///
    /// Note: this method of drawing does not scale. If you need to render
    /// a large number of shapes, use an `InstanceArray`. This approach is fine for
    /// this example since there are a fairly limited number of calls.
    fn draw(&self, canvas: &mut graphics::Canvas) {
        // First we set the color to draw with, in this case all food will be
        // colored blue.
        let color = [0.0, 0.0, 1.0, 1.0];
        // Then we draw a rectangle with the Fill draw mode, and we convert the
        // Food's position into a `ggez::Rect` using `.into()` which we can do
        // since we implemented `From<GridPosition>` for `Rect` earlier.
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest_rect(self.pos.into())
                .color(color),
        );
    }
}

struct Padle {
    /// Next we have the body, which we choose to represent as a `VecDeque`
    /// of `Segment`s.
    body: VecDeque<Segment>,
    /// Then we have the current direction the padle is moving. This is
    /// the direction it will move when `update` is called on it.
    dir: Direction,
}

impl Padle {
    pub fn new(pos: GridPosition) -> Self {
        let length = 5;
        let mut body = VecDeque::new();

        for seg_number in 0..length {
            body.push_back(Segment::new((pos.x, pos.y - seg_number).into()));
        }

        Padle {
            body,
            dir: Direction::None,
        }
    }

    /// The main update function for our padle which gets called every time
    /// we want to update the game state.
    fn update(&mut self) {
        if self.dir == Direction::Up {
            if let Some(back) = self.body.back() {
                let new_back_pos = GridPosition::new_from_move(back.pos, self.dir);
                let new_back = Segment::new(new_back_pos);
                self.body.push_back(new_back);
                self.body.pop_front();
                self.dir = Direction::None;
            }
        }
        if self.dir == Direction::Down {
            if let Some(front) = self.body.front() {
                let new_front_pos = GridPosition::new_from_move(front.pos, self.dir);
                let new_front = Segment::new(new_front_pos);
                self.body.push_front(new_front);
                self.body.pop_back();
                self.dir = Direction::None;
            }
        }
    }

    /// Here we have the Padle draw itself. 
    ///
    /// Again, note that this approach to drawing is fine for the limited scope of this
    /// example, but larger scale games will likely need a more optimized render path
    /// using `InstanceArray` or something similar that batches draw calls.
    fn draw(&self, canvas: &mut graphics::Canvas) {
        // We first iterate through the body segments and draw them.
        for seg in &self.body {
            // Again we set the color (in this case an orangey color)
            // and then draw the Rect that we convert that Segment's position into
            canvas.draw(
                &graphics::Quad,
                graphics::DrawParam::new()
                    .dest_rect(seg.pos.into())
                    .color([1.0, 1.0, 1.0, 1.0]),
            );
        }
    }

    // A helper function that determines whether
    // the ball meats a given padle based on its current position
    pub fn meats_ball(&self, ball: &Ball) -> bool {
        for seg in &self.body {
            if seg.pos == ball.pos {
                return true;
            }
        }
        false
    }
}

/// Now we have the heart of our game, the `GameState`. This struct
/// will implement ggez's `EventHandler` trait and will therefore drive
/// everything else that happens in our game.
struct GameState {
    /// First we need a Padles
    padle1: Padle,
    padle2: Padle,
    /// The ball
    ball: Ball,
    /// Whether the game is over or not
    gameover: bool,
    /// Our RNG state
    rng: Rand32,
}

impl GameState {
    /// Our new function will set up the initial state of our game.
    pub fn new() -> Self {
        // And we seed our RNG with the system RNG.
        let mut seed: [u8; 8] = [0; 8];
        // oorandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
        let mut rng = Rand32::new(u64::from_ne_bytes(seed));
        // Then we choose a random place to put our ball using the helper we made
        // earlier.
        let ball_pos = GridPosition::random(&mut rng, GRID_SIZE.0, GRID_SIZE.1);

        GameState {
            padle1: Padle::new((0, GRID_SIZE.1 / 2).into()),
            padle2: Padle::new((GRID_SIZE.0 - 1, GRID_SIZE.1 / 2).into()),
            ball: Ball::new(ball_pos),
            gameover: false,
            rng,
        }
    }
}

/// Now we implement `EventHandler` for `GameState`. This provides an interface
/// that ggez will call automatically when different events happen.
impl event::EventHandler<ggez::GameError> for GameState {
    /// Update will happen on every frame before it is drawn. This is where we update
    /// our game state to react to whatever is happening in the game world.
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Rely on ggez's built-in timer for deciding when to update the game, and how many times.
        // If the update is early, there will be no cycles, otherwises, the logic will run once for each
        // frame fitting in the time since the last update.
        while ctx.time.check_update_time(DESIRED_FPS) {
            // We check to see if the game is over. If not, we'll update. If so, we'll just do nothing.
            if !self.gameover {
                // Here we do the actual updating of our game world. 

                // First we tell the padles and ball to update itself,
                self.padle1.update();
                self.padle2.update();
                self.ball.update(&self.padle1, &self.padle2);

                // Next we check if the snake ate anything as it updated.
                // if let Some(ate) = self.snake.ate {
                //     // If it did, we want to know what it ate.
                //     match ate {
                //         // If it ate a piece of food, we randomly select a new position for our piece of food
                //         // and move it to this new position.
                //         Ate::Food => {
                //             let new_food_pos =
                //                 GridPosition::random(&mut self.rng, GRID_SIZE.0, GRID_SIZE.1);
                //             self.food.pos = new_food_pos;
                //         }
                //         // If it ate itself, we set our gameover state to true.
                //         Ate::Itself => {
                //             self.gameover = true;
                //         }
                //     }
                // }
            }
        }

        Ok(())
    }

    /// draw is where we should actually render the game's current state.
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // First we create a canvas that renders to the frame, and clear it to a (sort of) green color
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.0, 0.0, 0.0, 1.0]));

        // Then we tell the padles to draw themselves
        self.padle1.draw(&mut canvas);
        self.padle2.draw(&mut canvas);

        // Then we tell the ballto draw themselves
        self.ball.draw(&mut canvas);

        // Finally, we "flush" the draw commands.
        // Since we rendered to the frame, we don't need to tell ggez to present anything else,
        // as ggez will automatically present the frame image unless told otherwise.
        canvas.finish(ctx)?;

        // We yield the current thread until the next update
        ggez::timer::yield_now();
        // And return success.
        Ok(())
    }

    /// `key_down_event` gets fired when a key gets pressed.
    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        // Here we attempt to convert the Keycode into a Direction using the helper
        // we defined earlier.
        if let Some(player_number) = input.keycode.and_then(Direction::from_keycode_player_number) {
            if let Some(dir) = input.keycode.and_then(Direction::from_keycode) {
                if player_number == 1 {
                    self.padle1.dir = dir;
                }
                if player_number == 2 {
                    self.padle2.dir = dir;
                }
            }
        }
        
        // If it succeeds, we check if a new direction has already been set
        // and make sure the new direction is different then `snake.dir`
        // if self.paddle.dir != self.paddle.last_update_dir && dir.inverse() != self.snake.dir {
        //     self.snake.next_dir = Some(dir);
        // } else if dir.inverse() != self.snake.last_update_dir {
            // If no new direction has been set and the direction is not the inverse
            // of the `last_update_dir`, then set the snake's new direction to be the
            // direction the user pressed.
        // }

        Ok(())
    }
}

fn main() -> GameResult {
    // Here we use a ContextBuilder to setup metadata about our game. First the title and author
    let (ctx, events_loop) = ggez::ContextBuilder::new("snake", "Game World")
        // Next we set up the window. This title will be displayed in the title bar of the window.
        .window_setup(ggez::conf::WindowSetup::default().title("Movinig Paddles!"))
        // Now we get to set the size of the window, which we use our SCREEN_SIZE constant from earlier to help with
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        // And finally we attempt to build the context and create the window. If it fails, we panic with the message
        // "Failed to build ggez context"
        .build()?;

    // Next we create a new instance of our GameState struct, which implements EventHandler
    let state = GameState::new();
    // And finally we actually run our game, passing in our context and state.
    event::run(ctx, events_loop, state)
}