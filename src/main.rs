use std::{env, fs, io, process::ExitCode};

use constants::{SCALE, SCREEN_WIDTH, TICKS_PER_FRAME, WINDOW_HEIGHT, WINDOW_WIDTH};
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
                Event::Quit { .. } => {
                    break 'gameloop;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'gameloop;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key_to_button(key) {
                        emu.keypress(k as usize, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key_to_button(key) {
                        emu.keypress(k as usize, false);
                    }
                }
                _ => (),
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            emu.tick();
        }

        emu.tick_timers();
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

fn key_to_button(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
