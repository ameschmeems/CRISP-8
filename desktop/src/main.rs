extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use std::time::Duration;

fn main()
{
	let sdl = sdl2::init().unwrap();
	let video_subsystem = sdl.video().unwrap();
	let window = video_subsystem
		.window("CRISP-8", 1280, 720)
		.position_centered()
		.build()
		.unwrap();

	let mut canvas = window.into_canvas().build().unwrap();

	canvas.set_draw_color(Color::RGB(0, 255, 255));
	canvas.clear();
	canvas.present();
	let mut event_pump = sdl.event_pump().unwrap();
	let mut i = 0;
	'main: loop
	{
		i = (i + 1) % 255;
		canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
		canvas.clear();
		for event in event_pump.poll_iter()
		{
			match event
			{
				Event::Quit { .. } => { break 'main },
				_ => {}
			}
		}
		canvas.present();
		::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
	}
}
