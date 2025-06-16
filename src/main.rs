use std::{env, fs, io, process::ExitCode};

use constants::{SCALE, SCREEN_HEIGHT, SCREEN_WIDTH, WINDOW_HEIGHT, WINDOW_WIDTH};
use emu::Emu;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render};

mod constants;
mod emu;

fn main() -> ExitCode {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return ExitCode::FAILURE;
    }

    // TODO: more robust error handling

    let sdl_context = sdl2::init().unwrap();
    let video_subsytem = sdl_context.video().unwrap();

    let window = video_subsytem
        .window("CHIP-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut emu = match create_and_load_emulator(&args[1]) {
        Ok(emu) => emu,
        Err(_) => {
            eprintln!("Unable to load emulator file!");
            return ExitCode::FAILURE;
        }
    };

    'gameloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    break 'gameloop;
                }
                _ => (),
            }
        }

        emu.tick();
        draw_screen(&emu, &mut canvas);
    }

    ExitCode::SUCCESS
}

fn draw_screen(emu: &Emu, canvas: &mut render::Canvas<sdl2::video::Window>) {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    let screen_buf = emu.get_display();

    // Clear to white and draw
    canvas.set_draw_color(Color::WHITE);

    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }

    canvas.present();
}

fn create_and_load_emulator(file: &str) -> io::Result<Emu> {
    let data = fs::read(file)?;

    let mut emu = Emu::new();

    emu.load(&data);

    Ok(emu)
}
