extern crate core;

mod op_codes_parser;
mod cartridge;
mod cpu;
mod mmu;
mod ppu;
mod interrupt;
mod gamepad;

#[cfg(test)]
mod tests;

use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::{env, fs, thread};
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use image;
use image::RgbaImage;
use image::ColorType::{Rgb8, Rgba8};
use piston_window::{Button, ButtonState, Context, Event, image as draw_image, Input, Key, PistonWindow, Texture, TextureContext, TextureSettings, WindowSettings};
use crate::cartridge::cartridge::{Cartridge, read_cartridge};
use crate::cpu::CPU::CPU;
use crate::gamepad::gamepad::ColumnType::{Action, Direction};
use crate::mmu::mmu::MMU;
use crate::ppu::ppu::{dump_current_screen_tiles, dump_tile_map, PPU, PPU_mode, tile_set_to_rgba_image};

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom_name = args.last().unwrap().clone();

    let (cpu_sender, _) : (Sender<&Vec<u8>>, Receiver<&Vec<u8>>) = mpsc::channel();
    let (window_sender, cpu_receiver) : (Sender<(Key, ButtonState)>, Receiver<(Key, ButtonState)>) = mpsc::channel();

    let image_buffer = Arc::new(Mutex::new(RgbaImage::new(160, 144)));
    let image_buffer_reference = image_buffer.clone();

    let cpu_thread = thread::spawn(move|| run_cpu(cpu_sender, cpu_receiver, image_buffer_reference, rom_name));

    let mut window: PistonWindow = WindowSettings::new("Pog!", [160, 144]).exit_on_esc(true).build().unwrap();

    let (mut texture, mut texture_context) = {
        let mut texture_context = TextureContext {
            factory: window.factory.clone(),
            encoder: window.factory.create_command_buffer().into()
        };
        let image_buffer = image::ImageBuffer::new(160, 144);
        let texture = Texture::from_image(
            &mut texture_context,
            &image_buffer,
            &TextureSettings::new()
        ).unwrap();
        (texture, texture_context)
    };

    while let Some(event) = window.next() {
        match event {
            Event::Input(input, _) => {
                match input {
                    Input::Button(ButtonArgs) => {
                        match ButtonArgs.button {
                            Button::Keyboard(key) => {
                                window_sender.send((key, ButtonArgs.state)).unwrap();
                            }
                            _ => {}
                        }
                    }
                    Input::Close(_) => {
                        window_sender.send((Key::Escape, ButtonState::Press)).unwrap();
                    }
                    _ => {}
                }
            }
            Event::Loop(_) => {

                window.draw_2d(&event, |c: Context, g, device| {

                    texture.update(&mut texture_context, &*image_buffer.lock().unwrap()).unwrap();
                    draw_image(&texture, c.transform, g);
                    texture_context.encoder.flush(device);
                });
            }
            _ => {}
        }

    }

    let _ = cpu_thread.join();

}

fn run_cpu(_: Sender<&Vec<u8>>, cpu_receiver: Receiver<(Key, ButtonState)>, image_buffer_reference: Arc<Mutex<RgbaImage>>, rom_name: String) {
    let cartridge: Cartridge = read_cartridge(&rom_name);

    let mut ppu: PPU = PPU::new();
    let mmu: MMU = MMU::new(Some(cartridge), &mut ppu);
    let mut cpu: CPU = CPU::new(mmu);

    'main: loop {
        let clock = cpu.step();
        let current_cpu_mode = cpu.MMU.PPU.step(clock);

        if current_cpu_mode == PPU_mode::HBlank {
            let mut image_buffer = image_buffer_reference.lock().unwrap();
            (*image_buffer) = cpu.MMU.PPU.image_buffer.clone();
            //let tile_set_dump = tile_set_to_rgba_image(cpu.MMU.PPU.tile_set);
            //(*image_buffer) = tile_set_dump;
        }

        let received = cpu_receiver.try_recv();
        if received.is_ok() {
            let (key, state) = received.unwrap();
            match state {
                ButtonState::Press => {
                    match key {
                        Key::Escape => {
                            break 'main
                        },
                        Key::L => {
                            //toggle cpu logging
                            cpu.logging = !cpu.logging;
                        },
                        Key::T => {
                            //toggle tileset area
                            if cpu.MMU.read_byte(0xFF40) == 0x91 {
                                cpu.MMU.write_byte(0xFF40, 0x81);
                            }else{
                                cpu.MMU.write_byte(0xFF40, 0x91);
                            }
                        },
                        Key::D => {
                            println!("{}", cpu);

                            //dump current instruction
                            cpu.MMU.disassemble((cpu.Registers.get_item("PC") - 10) as i32, 20, cpu.Registers.get_item("PC") as i32);

                            //dump current tileset
                            let tile_set_dump: RgbaImage = tile_set_to_rgba_image(cpu.MMU.PPU.tile_set);
                            image::save_buffer(&Path::new("last_tile_set.png"), &*tile_set_dump.into_vec(), 20 * 8, 20 * 8, Rgba8).expect("TODO: panic message");

                            //dump lcdc status
                            cpu.MMU.PPU.print_lcdc_status();

                            //todo dump tile maps (at 0x9800 and 0x9C00)
                            let first_tile_map = dump_tile_map(cpu.MMU.PPU.video_ram, 0x1800);
                            fs::write("tm1.txt", first_tile_map).expect("Unable to write file");

                            let second_tile_map = dump_tile_map(cpu.MMU.PPU.video_ram, 0x1C00);
                            fs::write("tm2.txt", second_tile_map).expect("Unable to write file");

                            let current_screen_tiles = format!("{:?}", dump_current_screen_tiles(cpu.MMU.PPU));
                            fs::write("current_screen_tiles.txt", current_screen_tiles).expect("Unable to write file");
                        },
                        Key::Down | Key::Up | Key::Left | Key::Right | Key::Space | Key::Comma | Key::X | Key::Z => {
                            cpu.MMU.gamepad.key_pressed(key)
                        },
                        _ => {}
                    }
                }
                ButtonState::Release => {
                    match key {
                        Key::Down | Key::Up | Key::Left | Key::Right | Key::Space | Key::Comma | Key::X | Key::Z => {
                            cpu.MMU.gamepad.key_released(key)
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}
