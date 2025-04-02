mod vm;

use std::fs::File;
use std::io::Read;
use sdl2::pixels::PixelFormatEnum;
use vm::{VM, DISPLAY_WIDTH, DISPLAY_HEIGHT};

pub const WINDOW_WIDTH: usize = 640;
pub const WINDOW_HEIGHT: usize = 320;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_rom>", args[0]);
        std::process::exit(1);
    }

    let mut file = File::open(&args[1]).expect("Failed to open ROM file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read ROM file");

    let sdl_context = sdl2::init().expect("Failed to initialize SDL2");
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to initialize video subsystem");
    let window = video_subsystem
        .window("CHIP-8 Emulator", WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .position_centered()
        .build()
        .expect("Failed to create window");
    let canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .expect("Failed to create canvas");

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_target(
            PixelFormatEnum::RGB332,
            DISPLAY_WIDTH as u32,
            DISPLAY_HEIGHT as u32
        )
        .expect("Failed to create texture");

    let mut vm = VM::new(sdl_context, canvas, texture);
    vm.load_program(&buffer);

    println!("Loaded {} bytes into RAM (address 0x200)", buffer.len());
    println!("Starting VM...");

    vm.mainloop();
}
