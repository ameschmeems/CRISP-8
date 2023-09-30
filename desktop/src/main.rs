extern crate sdl2;

use crisp8_core::*;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::env;
use std::fs::File;
use std::io::Read;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

fn main()
{
	let args: Vec<_> = env::args().collect();
	if args.len() != 2
	{
		println!("Usage: cargo run rom_path_here");
		return ;
	}

	// SDL setup
	let sdl = sdl2::init().unwrap();
	let video_subsystem = sdl.video().unwrap();
	let window = video_subsystem
		.window("CRISP-8", WINDOW_WIDTH, WINDOW_HEIGHT)
		.position_centered()
		.opengl()
		.build()
		.unwrap();

	let mut canvas = window
		.into_canvas()
		.present_vsync()
		.build()
		.unwrap();
	canvas.clear();
	canvas.present();
	
	let mut event_pump = sdl.event_pump().unwrap();

	// Interpreter setup + loading program
	let mut crisp8 = Emu::new();

	let mut rom = File::open(&args[1]).expect("Unable to open file");
	let mut buff = Vec::new();
	rom.read_to_end(&mut buff).unwrap();
	crisp8.load(&buff);
	'main: loop
	{
		for event in event_pump.poll_iter()
		{
			match event
			{
				Event::Quit { .. } => break 'main,
				_ => {}
			}
		}

		crisp8.tick();
		draw_screen(&crisp8, &mut canvas);
	}
}

fn draw_screen(emu: &Emu, canvas: &mut Canvas<Window>)
{
	canvas.set_draw_color(Color::RGB(0, 0, 0));
	canvas.clear();

	let screen_buff = emu.get_screen();
	
	canvas.set_draw_color(Color::RGB(255, 255, 255));
	for (index, pixel) in screen_buff.iter().enumerate()
	{
		if *pixel
		{
			let x = (index % SCREEN_WIDTH) as u32;
			let y = (index / SCREEN_WIDTH) as u32;
	
			let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
			canvas.fill_rect(rect).unwrap();
		}
	}
	canvas.present();
}
