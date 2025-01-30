use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use std::collections::LinkedList;

use termion::{event::Key, input::TermRead};
use termion::cursor::Hide;
use termion::screen::IntoAlternateScreen;
use rand::Rng;

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Game {
    width: u16,
    height: u16,
    snake: LinkedList<(u16, u16)>,
    direction: Direction,
    food: (u16, u16),
    game_over: bool,
}

impl Game {
    fn new(width: u16, height: u16) -> Self {
        let mut snake = LinkedList::new();
        snake.push_back((width / 2, height / 2));
        snake.push_back((width / 2 - 1, height / 2));
        snake.push_back((width / 2 - 2, height / 2));

        let mut rng = rand::thread_rng();
        let food = (
            rng.gen_range(2..width - 2),
            rng.gen_range(2..height - 2),
        );

        Game {
            width,
            height,
            snake,
            direction: Direction::Right,
            food,
            game_over: false,
        }
    }

    fn update(&mut self) {
        let head = self.snake.front().cloned().unwrap();
        let new_head = match self.direction {
            Direction::Up => (head.0, head.1 - 1),
            Direction::Down => (head.0, head.1 + 1),
            Direction::Left => (head.0 - 1, head.1),
            Direction::Right => (head.0 + 1, head.1),
        };

        // Check collision with walls
        if new_head.0 == 0 || new_head.0 >= self.width - 1 ||
           new_head.1 == 0 || new_head.1 >= self.height - 1 {
            self.game_over = true;
            return;
        }

        // Check collision with self
        if self.snake.contains(&new_head) {
            self.game_over = true;
            return;
        }

        self.snake.push_front(new_head);

        if new_head == self.food {
            let mut rng = rand::thread_rng();
            self.food = (
                rng.gen_range(1..self.width - 1),
                rng.gen_range(1..self.height - 1),
            );
        } else {
            self.snake.pop_back();
        }
    }

    fn draw(&self, screen: &mut impl Write) {
        write!(screen, "{}", termion::clear::All).unwrap();

        // Draw borders
        for x in 0..self.width {
            write!(screen, "{}#", termion::cursor::Goto(x + 1, 1)).unwrap();
            write!(screen, "{}#", termion::cursor::Goto(x + 1, self.height)).unwrap();
        }
        for y in 2..self.height {
            write!(screen, "{}#", termion::cursor::Goto(1, y)).unwrap();
            write!(screen, "{}#", termion::cursor::Goto(self.width, y)).unwrap();
        }

        // Draw snake
        for &(x, y) in &self.snake {
            write!(screen, "{}■", termion::cursor::Goto(x + 1, y + 1)).unwrap();
        }

        // Draw food
        write!(screen, "{}●", termion::cursor::Goto(self.food.0 + 1, self.food.1 + 1)).unwrap();

        screen.flush().unwrap();
    }
}

fn main() {
    // Initialize terminal
    let stdout = io::stdout()
        .into_raw_mode()
        .unwrap()
        .into_alternate_screen()
        .unwrap();

    let mut screen = stdout;
    write!(screen, "{}", Hide).unwrap();

    // Initialize game
    let (width, height) = termion::terminal_size().unwrap();
    let mut game = Game::new(width, height);

    let stdin = io::stdin();
    let mut keys = stdin.keys();

    // Game loop
    while !game.game_over {
        game.draw(&mut screen);

        let timeout = Duration::from_millis(100);
        let input = keys.next();

        if let Some(Ok(key)) = input {
            match key {
                Key::Up if game.direction != Direction::Down => game.direction = Direction::Up,
                Key::Down if game.direction != Direction::Up => game.direction = Direction::Down,
                Key::Left if game.direction != Direction::Right => game.direction = Direction::Left,
                Key::Right if game.direction != Direction::Left => game.direction = Direction::Right,
                Key::Char('q') => break,
                _ => {}
            }
        }

        game.update();
        thread::sleep(timeout);
    }

    // Game over screen
    write!(screen, "{}Game Over! Press 'q' to quit", termion::cursor::Goto(width / 2 - 5, height / 2)).unwrap();
    screen.flush().unwrap();

    // Wait for quit
    for key in &mut keys {
        if let Ok(Key::Char('q')) = key {
            break;
        }
    }
}
