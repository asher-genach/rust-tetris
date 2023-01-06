// 20 x 40
// logical place (20,40)
// physical place - Need to create a function that maps logical place to physical place.

//TODO:

#[macro_use]
extern crate serde_derive;

extern crate find_folder;
extern crate piston_window;
extern crate rand;
extern crate serde;
extern crate serde_json;

use piston_window::*;
use rand::prelude::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::{thread, time};
use std::fmt;

const SIZE: u32 = 10;
const STEP: u32 = 10;
const SCREEN_WIDTH: u32 = 410;
const SCREEN_HEIGHT: u32 = 670;
const BRICK_SIZE: i32 = 25;
const BRICKS_PAD: i32 = 1;
const BOARD_BRICK_WIDTH: usize = 15;
const BOARD_BRICK_HEIGHT: usize = 25;

//Colors

const COLOR_BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const COLOR_RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const COLOR_GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const COLOR_BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const COLOR_ORANGE: [f32; 4] = [1.0, 140.0 / 255.0, 0.0, 1.0];
const COLOR_MAGENTA: [f32; 4] = [139.0 / 255.0, 0.0 / 255.0, 139.0 / 255.0, 1.0];
const COLOR_YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
const COLOR_RND: [f32; 4] = [0.6, 0.4, 0.2, 1.0];

const NUM_LIVES: u8 = 7;

/*
Grid 20 x 40
    */

/*struct Score {
    hi_score: u64,
    score: u64,
    inc_val: u64,
    num_lives: u8,
    level: u8,
    level_ended: bool,
    glyphs: Option<Glyphs>,
    env_args: Vec<String>,
}

impl Score {
    fn new(env_args: Vec<String>) -> Self {
        Score {
            hi_score: 0,
            score: 0,
            inc_val: 1,
            num_lives: NUM_LIVES,
            level: 1,
            level_ended: false,
            glyphs: None,
            env_args,
        }
    }
} */

struct Background {}

impl Background {
    fn new() -> Self {
        Background {}
    }

    fn draw(&mut self, context: &Context, graphics: &mut G2d) {
        line(
            COLOR_RED,
            1.0,
            [10.0, 10.0, (SCREEN_WIDTH - 10) as f64, 10.0],
            context.transform,
            graphics,
        );

        line(
            COLOR_RED,
            1.0,
            [
                (SCREEN_WIDTH - 10) as f64,
                10.0,
                (SCREEN_WIDTH - 10) as f64,
                (SCREEN_HEIGHT - 10) as f64,
            ],
            context.transform,
            graphics,
        );

        line(
            COLOR_RED,
            1.0,
            [
                (SCREEN_WIDTH - 10) as f64,
                (SCREEN_HEIGHT - 10) as f64,
                10.0,
                (SCREEN_HEIGHT - 10) as f64,
            ],
            context.transform,
            graphics,
        );

        line(
            COLOR_RED,
            1.0,
            [10.0, (SCREEN_HEIGHT - 10) as f64, 10.0, 10.0],
            context.transform,
            graphics,
        );
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Orientation {
    num_states: usize,
    cur_state: usize,
}

impl Orientation {
    fn new(num_states: usize) -> Self {
        Orientation {
            num_states,
            cur_state: 0,
        }
    }

    fn rotate(&mut self) {
        if self.cur_state + 1 < self.num_states {
            self.cur_state += 1;
        } else {
            self.cur_state = 0;
        }
    }
}

#[derive(Clone)]
struct Brick {
    brick_pos_x: i32,
    brick_pos_y: i32,
}

impl Brick {
    fn new(pos_x: i32, pos_y: i32) -> Self {
        Brick {
            brick_pos_x: pos_x,
            brick_pos_y: pos_y,
        }
    }

    fn draw(
        &mut self,
        orig_x: i32,
        orig_y: i32,
        color: [f32; 4],
        context: &Context,
        graphics: &mut G2d,
    ) {
        let x = 10 + (orig_x + self.brick_pos_x) * (BRICK_SIZE + BRICKS_PAD);
        let y = 10 + (orig_y + self.brick_pos_y) * (BRICK_SIZE + BRICKS_PAD);

        rectangle(
            color,
            [x as f64, y as f64, BRICK_SIZE as f64, BRICK_SIZE as f64],
            context.transform,
            graphics,
        );
    }
}

/*
    TypeA, - 4 rotation states
         ++
          ++
    TypeB, - 2 rotation states
     ++++
    TypeC, - 4 rotation states
         +
        +++
    TypeD, - 4 rotation states
        +++
          +
    TypeE - 4 rotation states
        +++
        +
    TypeF - 1 rotation state - No rotation
        ++
        ++

    TypeG - 4 rotation states
        ++
       ++
*/

#[derive(Clone)]
struct BrickGroup {
    bg_pos_x: i32,
    bg_pos_y: i32,
    orientation: Orientation,
    bg_bricks: Vec<Vec<Brick>>,
    color: [f32; 4],
}

impl BrickGroup {
    fn new(pos_x: i32, pos_y: i32, num_states:usize, color: [f32; 4]) -> Self {
        let mut bg_bricks = vec![];
        
        for _ in 0..num_states {
            bg_bricks.push(vec![]);
        }

        BrickGroup {
            bg_pos_x: pos_x,
            bg_pos_y: pos_y,
            orientation: Orientation::new(num_states),
            bg_bricks,
            color,
        }
    }

    fn add_brick(&mut self, pos_x: i32, pos_y: i32, orient:usize) {
        self.bg_bricks[orient].push(Brick::new(pos_x, pos_y));
    }

    fn move_right_one(&mut self) {
        self.bg_pos_x += 1;
    }

    fn move_left_one(&mut self) {
        self.bg_pos_x -= 1;
    }

    fn move_down_one(&mut self) {
        self.bg_pos_y += 1;
    }

    fn draw(&mut self, context: &Context, graphics: &mut G2d) {
        let orient = self.orientation.cur_state;

        for brick in self.bg_bricks[orient].iter_mut() {
            brick.draw(self.bg_pos_x, self.bg_pos_y, self.color,
                       context, graphics);
        }
    }
}

impl fmt::Display for BrickGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       let orient      = self.orientation.cur_state;
       let bg_pos_x    = self.bg_pos_x;
       let bg_pos_y    = self.bg_pos_y;

       write!(f, "brick :");
       
       for brick in self.bg_bricks[orient].iter() {
          write!(f, "({},{}) ", bg_pos_x + brick.brick_pos_x,
                                bg_pos_y + brick.brick_pos_y);
       }
      
       write!(f, "\n")
    }
}

// Looking at Testris_rotation_Nintendo.
// The (0,0) is the brick on the lower left of the 3x3 or 4x4 square.
// array of brick group. Each dimension represents the bricks placement
// in a specific orientation.

trait BGType {
    fn init(&mut self);
    fn get_brick_group(&mut self) -> &mut BrickGroup;
}

/* ++
    ++  Line 6 in Nintendo*/
struct BGTypeA {
    brick_group: BrickGroup,
}

impl BGType for BGTypeA {
    fn get_brick_group(&mut self) -> &mut BrickGroup {
        &mut self.brick_group
    }

    fn init(&mut self) {
        self.brick_group.add_brick(0, 1, 0);
        self.brick_group.add_brick(1, 1, 0);
        self.brick_group.add_brick(1, 2, 0);
        self.brick_group.add_brick(2, 2, 0);
        
        self.brick_group.add_brick(2, 0, 1);
        self.brick_group.add_brick(1, 1, 1);
        self.brick_group.add_brick(2, 1, 1);
        self.brick_group.add_brick(1, 2, 1);
        
        self.brick_group.add_brick(0, 1, 2);
        self.brick_group.add_brick(1, 1, 2);
        self.brick_group.add_brick(1, 2, 2);
        self.brick_group.add_brick(2, 2, 2);
        
        self.brick_group.add_brick(2, 0, 3);
        self.brick_group.add_brick(1, 1, 3);
        self.brick_group.add_brick(2, 1, 3);
        self.brick_group.add_brick(1, 2, 3);
    }
}

impl BGTypeA {
    fn new(pos_x: i32, pos_y: i32) -> Self {
        let mut brick_group = BrickGroup::new(pos_x, pos_y, 4, COLOR_BLACK); 

        BGTypeA { brick_group }
    }
}

/* ++++  - Line 1 in the drawing*/
struct BGTypeB {
    brick_group: BrickGroup,
}

impl BGType for BGTypeB {
    fn get_brick_group(&mut self) -> &mut BrickGroup {
        &mut self.brick_group
    }

    fn init(&mut self) {
        self.brick_group.add_brick(0, 1, 0);
        self.brick_group.add_brick(1, 1, 0);
        self.brick_group.add_brick(2, 1, 0);
        self.brick_group.add_brick(3, 1, 0);
        
        self.brick_group.add_brick(2, 0, 1);
        self.brick_group.add_brick(2, 1, 1);
        self.brick_group.add_brick(2, 2, 1);
        self.brick_group.add_brick(2, 3, 1);
    }
}

impl BGTypeB {
    fn new(pos_x: i32, pos_y: i32) -> Self {
        let mut brick_group = BrickGroup::new(pos_x, pos_y, 2, COLOR_RED); 

        BGTypeB { brick_group }
    }
}

/* +
  +++

Line 5- Nintendo
*/
struct BGTypeC {
    brick_group: BrickGroup,
}

impl BGType for BGTypeC {
    fn get_brick_group(&mut self) -> &mut BrickGroup {
        &mut self.brick_group
    }

    fn init(&mut self) {
        self.brick_group.add_brick(1, 2, 0);
        self.brick_group.add_brick(0, 1, 0);
        self.brick_group.add_brick(1, 1, 0);
        self.brick_group.add_brick(2, 1, 0);
        
        self.brick_group.add_brick(0, 1, 1);
        self.brick_group.add_brick(1, 0, 1);
        self.brick_group.add_brick(1, 1, 1);
        self.brick_group.add_brick(1, 2, 1);
        
        self.brick_group.add_brick(1, 0, 2);
        self.brick_group.add_brick(0, 1, 2);
        self.brick_group.add_brick(1, 1, 2);
        self.brick_group.add_brick(2, 1, 2);
        
        self.brick_group.add_brick(2, 1, 3);
        self.brick_group.add_brick(1, 0, 3);
        self.brick_group.add_brick(1, 1, 3);
        self.brick_group.add_brick(1, 2, 3);
    }
}

impl BGTypeC {
    fn new(pos_x: i32, pos_y: i32) -> Self {
        let mut brick_group = BrickGroup::new(pos_x, pos_y, 4, COLOR_GREEN); 

        BGTypeC { brick_group }
    }
}

/* +++
     + 

Line 2 - Nintendo
*/
struct BGTypeD {
    brick_group: BrickGroup,
}

impl BGType for BGTypeD {
    fn get_brick_group(&mut self) -> &mut BrickGroup {
        &mut self.brick_group
    }

    fn init(&mut self) {
        self.brick_group.add_brick(2, 2, 0);
        self.brick_group.add_brick(0, 1, 0);
        self.brick_group.add_brick(1, 1, 0);
        self.brick_group.add_brick(2, 1, 0);
        
        self.brick_group.add_brick(0, 2, 1);
        self.brick_group.add_brick(1, 0, 1);
        self.brick_group.add_brick(1, 1, 1);
        self.brick_group.add_brick(1, 2, 1);
        
        self.brick_group.add_brick(0, 1, 2);
        self.brick_group.add_brick(1, 1, 2);
        self.brick_group.add_brick(2, 1, 2);
        self.brick_group.add_brick(0, 0, 2);
        
        self.brick_group.add_brick(1, 0, 3);
        self.brick_group.add_brick(1, 1, 3);
        self.brick_group.add_brick(1, 2, 3);
        self.brick_group.add_brick(2, 0, 3);
    }
}


impl BGTypeD {
    fn new(pos_x: i32, pos_y: i32) -> Self {
        let mut brick_group = BrickGroup::new(pos_x, pos_y, 4, COLOR_BLUE); 

        BGTypeD { brick_group }
    }
}

/* +++
   +

Line 3 - Nintendo
*/
struct BGTypeE {
    brick_group: BrickGroup,
}

impl BGType for BGTypeE {
    fn get_brick_group(&mut self) -> &mut BrickGroup {
        &mut self.brick_group
    }

    fn init(&mut self) {
        self.brick_group.add_brick(0, 2, 0);
        self.brick_group.add_brick(0, 1, 0);
        self.brick_group.add_brick(1, 1, 0);
        self.brick_group.add_brick(2, 1, 0);
        
        self.brick_group.add_brick(0, 0, 1);
        self.brick_group.add_brick(1, 0, 1);
        self.brick_group.add_brick(1, 1, 1);
        self.brick_group.add_brick(1, 2, 1);
        
        self.brick_group.add_brick(2, 0, 2);
        self.brick_group.add_brick(0, 1, 2);
        self.brick_group.add_brick(1, 1, 2);
        self.brick_group.add_brick(2, 1, 2);
        
        self.brick_group.add_brick(2, 2, 3);
        self.brick_group.add_brick(1, 0, 3);
        self.brick_group.add_brick(1, 1, 3);
        self.brick_group.add_brick(1, 2, 3);
    }
}


impl BGTypeE {
    fn new(pos_x: i32, pos_y: i32) -> Self {
        let mut brick_group = BrickGroup::new(pos_x, pos_y, 4, COLOR_ORANGE); 

        BGTypeE { brick_group }
    }
}

/* ++
   ++
Line 1- Nintendo
*/
struct BGTypeF {
    brick_group: BrickGroup,
}

impl BGType for BGTypeF {
    fn get_brick_group(&mut self) -> &mut BrickGroup {
        &mut self.brick_group
    }

    fn init(&mut self) {
        self.brick_group.add_brick(1, 1, 0);
        self.brick_group.add_brick(2, 1, 0);
        self.brick_group.add_brick(1, 2, 0);
        self.brick_group.add_brick(2, 2, 0);
    }
}


impl BGTypeF {
    fn new(pos_x: i32, pos_y: i32) -> Self {
        let mut brick_group = BrickGroup::new(pos_x, pos_y, 1, COLOR_MAGENTA); 

        BGTypeF { brick_group }
    }
}

/*  ++
   ++
Line 1- Nintendo
*/
struct BGTypeG {
    brick_group: BrickGroup,
}

impl BGType for BGTypeG {
    fn get_brick_group(&mut self) -> &mut BrickGroup {
        &mut self.brick_group
    }

    fn init(&mut self) {
        self.brick_group.add_brick(0, 2, 0);
        self.brick_group.add_brick(1, 2, 0);
        self.brick_group.add_brick(1, 1, 0);
        self.brick_group.add_brick(2, 1, 0);
        
        self.brick_group.add_brick(1, 0, 1);
        self.brick_group.add_brick(1, 1, 1);
        self.brick_group.add_brick(2, 1, 1);
        self.brick_group.add_brick(2, 2, 1);

        self.brick_group.add_brick(0, 2, 2);
        self.brick_group.add_brick(1, 2, 2);
        self.brick_group.add_brick(1, 1, 2);
        self.brick_group.add_brick(2, 1, 2);
        
        self.brick_group.add_brick(1, 0, 3);
        self.brick_group.add_brick(1, 1, 3);
        self.brick_group.add_brick(2, 1, 3);
        self.brick_group.add_brick(2, 2, 3);

    }
}


impl BGTypeG {
    fn new(pos_x: i32, pos_y: i32) -> Self {
        let mut brick_group = BrickGroup::new(pos_x, pos_y, 4, COLOR_YELLOW); 

        BGTypeG { brick_group }
    }
}

#[derive(Copy, Debug, Clone, PartialEq)]
enum GameStatus {
    ACTIVE,
    SHAPE_PLACED,
    PAUSED,
    QUIT,
}

fn get_rand_shape(x_pos:i32, y_pos:i32) -> Box<dyn BGType> {
    let mut rng = rand::thread_rng();
    
    let rand_num:u8 = rng.gen::<u8>() % 7;

    match rand_num {
        0 => {
            Box::new(BGTypeA::new(x_pos, y_pos))
        },
        1 => {
            Box::new(BGTypeB::new(x_pos, y_pos))
        },
        2 => {
            Box::new(BGTypeC::new(x_pos, y_pos))
        },
        3 => {
            Box::new(BGTypeD::new(x_pos, y_pos))
        },
        4 => {
            Box::new(BGTypeE::new(x_pos, y_pos))
        },
        5 => {
            Box::new(BGTypeF::new(x_pos, y_pos))
        },
        6 => {
            Box::new(BGTypeG::new(x_pos, y_pos))
        },
        _ => {
            Box::new(BGTypeA::new(x_pos, y_pos)) // will never happen !!!
        },
    }
  } 

struct Graphics {
    window: PistonWindow,
    bg: Background,
}

impl Graphics {
  fn new() -> Self {
    let window = WindowSettings::new("Tetris", [SCREEN_WIDTH, SCREEN_HEIGHT])
            .exit_on_esc(true)
            .build()
            .unwrap();

    Graphics { window, bg: Background::new() }
  }
}

struct Board {
  board:[[bool;BOARD_BRICK_HEIGHT];BOARD_BRICK_WIDTH],
  brick_group_arr:Vec<BrickGroup>,
}

impl Board {
  fn new() -> Self
  {
    let board:[[bool;BOARD_BRICK_HEIGHT];BOARD_BRICK_WIDTH] = 
                    [[false;BOARD_BRICK_HEIGHT];BOARD_BRICK_WIDTH];
  
    Board { board, brick_group_arr: vec![] }
  }

  fn collide_w_backgroud(&mut self, brick_group:&mut BrickGroup) -> bool {
    let mut collide = false;
    let orient      = brick_group.orientation.cur_state;
    let bg_pos_y    = brick_group.bg_pos_y;

    for brick in brick_group.bg_bricks[orient].iter() {
      if ((bg_pos_y + brick.brick_pos_y) as usize) + 1 == BOARD_BRICK_HEIGHT {
        collide = true;
        break;
      }
    }

    collide
  }

  fn board_update(&mut self, brick_group:&BrickGroup) {
     let orient      = brick_group.orientation.cur_state;
     let bg_pos_x    = brick_group.bg_pos_x;
     let bg_pos_y    = brick_group.bg_pos_y;

     for brick in brick_group.bg_bricks[orient].iter() {
        self.board[(bg_pos_x + brick.brick_pos_x) as usize]
                  [(bg_pos_y + brick.brick_pos_y) as usize] = true;
     }
    
     self.brick_group_arr.push(brick_group.clone());

     //println!("{}", brick_group);
  }

  fn update_down_collide(&mut self, brick_group:&mut BrickGroup) -> bool {
    // TODO: Implement this
    let mut collide = false;

    collide = self.collide_w_backgroud(brick_group);

    if collide {
      self.board_update( brick_group );
    }

    collide
  }

  fn draw(&mut self, context: &Context, graphics: &mut G2d) {
        println!("In");
        for brick_group in self.brick_group_arr.iter_mut() {
          brick_group.draw(context, graphics);
        }

  }
}

struct Game {
    board:Board,
    graphics: Graphics,
    gb: Box<dyn BGType>,
    clock_counter: u128,
}
impl Game {
    fn new() -> Self {
        
        Game {
            board:Board::new(),
            graphics:Graphics::new(),
            gb:Box::new(BGTypeA::new(0,0)), 
            clock_counter: 0,
        }
    }

    fn game_move_down_one(&mut self) -> bool {
      let mut collide     = false;
      let     brick_group = self.gb.get_brick_group();

      collide = self.board.update_down_collide(brick_group);

      if !collide {
        self.gb.get_brick_group().move_down_one();
      } 

      collide
    }

    fn clock_delay(&mut self) -> bool {
        let mut collide = false;

        if self.clock_counter < 1000000
        {
            self.clock_counter += 1;
        }
        else
        {
            self.clock_counter = 0;
        }
        
        if self.clock_counter % 500 == 0 {
          collide = self.game_move_down_one();
        }

        collide
    }

    fn play(&mut self) -> GameStatus {
        let mut game_status  = GameStatus::ACTIVE;
        
        self.gb = get_rand_shape(0,0);

        while game_status == GameStatus::ACTIVE {
            game_status = self.shape_play();
        }

        game_status
    }

    fn shape_play(&mut self) -> GameStatus {
        let mut game_status = GameStatus::ACTIVE;

        self.gb.init();

        while let Some(event) = self.graphics.window.next() {
            // Catch Window CloseEvent
            //
            
            if self.clock_delay() {
                return GameStatus::SHAPE_PLACED;
            }

            if let Some(_) = event.close_args() {
                game_status = GameStatus::QUIT;
                break;
            }

            if let Some(button) = event.press_args() {
                match button {
                    Button::Keyboard(Key::Left) => {
                        self.gb.get_brick_group().move_left_one();
                    }
                    Button::Keyboard(Key::Up) => {
                        self.gb.get_brick_group().orientation.rotate();
                    }
                    Button::Keyboard(Key::Down) => {
                        if self.game_move_down_one() {
                          return GameStatus::SHAPE_PLACED;
                        }
                    }
                    Button::Keyboard(Key::Right) => {
                        self.gb.get_brick_group().move_right_one();
                    }
                    Button::Keyboard(Key::Escape) => {
                        game_status = GameStatus::QUIT;
                        break;
                    }
                    Button::Keyboard(Key::P) => {}
                    Button::Keyboard(Key::Space) => {}
                    _ => {
                    }
                }
            }

            self.draw(&event);
        }
        game_status
    }

    fn draw(&mut self, event: &Event) {
        let     bg     = &mut self.graphics.bg;
        let     board  = &mut self.board;
        let     gb     = &mut self.gb;
        let mut pos_x  = 10;
        let mut pos_y  = 10;

        self.graphics.window.draw_2d(event, |context, graphics| {
            clear([1.0, 1.0, 1.0, 0.0], graphics);

            bg.draw(&context, graphics);

            let x = 10 + 4 * (BRICK_SIZE + BRICKS_PAD);
            let y = 10 + 5 * (BRICK_SIZE + BRICKS_PAD);

            gb.get_brick_group().draw(&context, graphics);

           board.draw(&context, graphics);
        });

    }
}

struct FullGame 
{}

impl FullGame {
    fn new() -> Self {
        FullGame {}
    }

    fn full_game_play(&mut self) {
        loop {
            let mut game = Game::new();

            match game.play() {
                GameStatus::QUIT => {
                    break;
                }
                _ => {}
            }
        }
    }
}

fn main() {
    FullGame::new().full_game_play();
}
