use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::rwops::RWops;
use sdl2::ttf::Sdl2TtfContext;
use std::collections::LinkedList;
use std::time::{Duration, Instant};
use rand::Rng;

const GRID_SIZE: u32 = 20;
const GRID_WIDTH: u32 = 30;
const GRID_HEIGHT: u32 = 20;
const WINDOW_WIDTH: u32 = GRID_SIZE * GRID_WIDTH;
const WINDOW_HEIGHT: u32 = GRID_SIZE * GRID_HEIGHT;

// Embed the font file into the binary
const FONT_DATA: &[u8] = include_bytes!("arial.ttf");

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Snake {
    body: LinkedList<Rect>,
    direction: Direction,
}

impl Snake {
    fn new() -> Self {
        let mut body = LinkedList::new();
        let start_x = (GRID_WIDTH / 2) as i32;
        let start_y = (GRID_HEIGHT / 2) as i32;

        body.push_back(Rect::new(
            start_x * GRID_SIZE as i32,
            start_y * GRID_SIZE as i32,
            GRID_SIZE,
            GRID_SIZE,
        ));

        Snake {
            body,
            direction: Direction::Right,
        }
    }

    fn move_forward(&mut self, food: &mut Food) -> bool {
        let head = self.body.front().unwrap();
        let (x, y) = match self.direction {
            Direction::Up => (head.x(), head.y() - GRID_SIZE as i32),
            Direction::Down => (head.x(), head.y() + GRID_SIZE as i32),
            Direction::Left => (head.x() - GRID_SIZE as i32, head.y()),
            Direction::Right => (head.x() + GRID_SIZE as i32, head.y()),
        };

        // Check boundaries
        if x < 0 || x >= WINDOW_WIDTH as i32 || y < 0 || y >= WINDOW_HEIGHT as i32 {
            return false;
        }

        // Check self-collision
        let new_head = Rect::new(x, y, GRID_SIZE, GRID_SIZE);
        if self.body.iter().any(|segment| segment.has_intersection(new_head)) {
            return false;
        }

        self.body.push_front(new_head);

        // Check food collision
        if new_head.has_intersection(food.rect) {
            food.spawn(&self.body);
        } else {
            self.body.pop_back();
        }

        true
    }
}

struct Food {
    rect: Rect,
}

impl Food {
    fn new(snake_body: &LinkedList<Rect>) -> Self {
        let mut rng = rand::thread_rng();
        let mut rect = Rect::new(0, 0, GRID_SIZE, GRID_SIZE);
        self::spawn_food(&mut rect, snake_body, &mut rng);
        Food { rect }
    }

    fn spawn(&mut self, snake_body: &LinkedList<Rect>) {
        let mut rng = rand::thread_rng();
        self::spawn_food(&mut self.rect, snake_body, &mut rng);
    }
}

fn spawn_food(rect: &mut Rect, snake_body: &LinkedList<Rect>, rng: &mut impl Rng) {
    loop {
        let x = rng.gen_range(0..GRID_WIDTH) as i32 * GRID_SIZE as i32;
        let y = rng.gen_range(0..GRID_HEIGHT) as i32 * GRID_SIZE as i32;
        *rect = Rect::new(x, y, GRID_SIZE, GRID_SIZE);

        if !snake_body.iter().any(|segment| segment.has_intersection(*rect)) {
            break;
        }
    }
}

struct Game {
    snake: Snake,
    food: Food,
    score: u32,
    game_over: bool,
}

impl Game {
    fn new() -> Self {
        let snake = Snake::new();
        let food = Food::new(&snake.body);
        Game {
            snake,
            food,
            score: 0,
            game_over: false,
        }
    }

    fn update(&mut self) {
        if !self.game_over {
            self.game_over = !self.snake.move_forward(&mut self.food);
            if self.game_over {
                return;
            }

            if self.snake.body.front().unwrap().has_intersection(self.food.rect) {
                self.score += 10;
            }
        }
    }

    fn render(&self, canvas: &mut WindowCanvas, font: &sdl2::ttf::Font) -> Result<(), String> {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // Draw snake
        canvas.set_draw_color(Color::RGB(0, 255, 0));
        for segment in &self.snake.body {
            canvas.fill_rect(*segment)?;
        }

        // Draw food
        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.fill_rect(self.food.rect)?;

        // Draw score
        let surface = font
            .render(&format!("Score: {}", self.score))
            .blended(Color::RGB(255, 255, 255))
            .map_err(|e| e.to_string())?;

        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        let target_rect = Rect::new(10, 10, 100, 24);
        canvas.copy(&texture, None, Some(target_rect))?;

        if self.game_over {
            let surface = font
                .render("Game Over! Press Q to quit")
                .blended(Color::RGB(255, 255, 255))
                .map_err(|e| e.to_string())?;

            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;

            let target_rect = Rect::new(
                (WINDOW_WIDTH / 2 - 150) as i32,
                (WINDOW_HEIGHT / 2 - 12) as i32,
                300,
                24,
            );
            canvas.copy(&texture, None, Some(target_rect))?;
        }

        canvas.present();
        Ok(())
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    // Load the font from the embedded byte array
    let rwops = RWops::from_bytes(FONT_DATA).map_err(|e| e.to_string())?;
    let font = ttf_context.load_font_from_rwops(rwops, 24)?;

    let window = video_subsystem
        .window("Snake Game", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut game = Game::new();
    let mut last_update = Instant::now();
    let frame_duration = Duration::from_millis(100);

    'running: loop {
        // Handle input
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if game.game_over && key == Keycode::Q {
                        break 'running;
                    }

                    if !game.game_over {
                        match key {
                            Keycode::Up if game.snake.direction != Direction::Down => {
                                game.snake.direction = Direction::Up
                            }
                            Keycode::Down if game.snake.direction != Direction::Up => {
                                game.snake.direction = Direction::Down
                            }
                            Keycode::Left if game.snake.direction != Direction::Right => {
                                game.snake.direction = Direction::Left
                            }
                            Keycode::Right if game.snake.direction != Direction::Left => {
                                game.snake.direction = Direction::Right
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        // Update game state
        if !game.game_over && last_update.elapsed() >= frame_duration {
            game.update();
            last_update = Instant::now();
        }

        // Render
        game.render(&mut canvas, &font)?;
    }

    Ok(())
}
