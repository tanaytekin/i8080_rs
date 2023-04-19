use std::time::{Duration, Instant};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
mod renderer;
use renderer::Renderer;

// const CPU_FREQ: f32 = 800.0;
// const TIMER_FREQ: f32 = 60.0;
//
// const CPU_DURATION: Duration = Duration::from_micros((1.0 / CPU_FREQ * 1_000_000.0) as u64);
// const TIMER_DURATION: Duration = Duration::from_micros((1.0 / TIMER_FREQ * 1_000_000.0) as u64);

fn foo (_: &i8080::I8080,  _: u8,  _: u8) {
}
fn main() {
    env_logger::init();
    // let path = std::env::args().nth(1).expect("No ROM path is provided.");
    let opcodes = std::env::args().nth(1).expect("No ROM path is provided.");
    let opcodes: u64 = opcodes.parse().unwrap();
    // let mut chip8 = chip8::Chip8::new();
    let mut i8080 = i8080::I8080::new(64*1024, 8, foo);
    i8080.load("space-invaders.rom", 0, 0xF000);
    while i8080.opcode_count < opcodes {
        i8080.emulate();
    }
        i8080.print();
    // chip8.load(path).unwrap();

    let size = winit::dpi::PhysicalSize::<u32>::new((i8080::DISPLAY_WIDTH * 2) as u32, (i8080::DISPLAY_HEIGHT * 2) as u32);
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_inner_size(size).build(&event_loop).unwrap();
    let mut renderer = Renderer::new(&window).unwrap();

    let start_time = Instant::now();
    let mut cpu_timer = start_time;
    let mut timer = start_time;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { window_id, event } => {
            if window_id == window.id() {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(new_size) => {
                        renderer.resize(Some(new_size));
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        renderer.resize(Some(*new_inner_size));
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    } => match state {
                        ElementState::Pressed => match keycode {
                            VirtualKeyCode::Return => {
                                let cur = i8080.opcode_count;
                                while cur == i8080.opcode_count {
                                    i8080.emulate();
                                }
                                i8080.print();
                            },

                            // VirtualKeyCode::Key1 => chip8.keys[0x1] = true,
                            // VirtualKeyCode::Key2 => chip8.keys[0x2] = true,
                            // VirtualKeyCode::Key3 => chip8.keys[0x3] = true,
                            // VirtualKeyCode::Key4 => chip8.keys[0xC] = true,
                            // VirtualKeyCode::Q => chip8.keys[0x4] = true,
                            // VirtualKeyCode::W => chip8.keys[0x5] = true,
                            // VirtualKeyCode::E => chip8.keys[0x6] = true,
                            // VirtualKeyCode::R => chip8.keys[0xD] = true,
                            // VirtualKeyCode::A => chip8.keys[0x7] = true,
                            // VirtualKeyCode::S => chip8.keys[0x8] = true,
                            // VirtualKeyCode::D => chip8.keys[0x9] = true,
                            // VirtualKeyCode::F => chip8.keys[0xE] = true,
                            // VirtualKeyCode::Z => chip8.keys[0xA] = true,
                            // VirtualKeyCode::X => chip8.keys[0x0] = true,
                            // VirtualKeyCode::C => chip8.keys[0xB] = true,
                            // VirtualKeyCode::V => chip8.keys[0xF] = true,
                            _ => {}
                        },
                        ElementState::Released => match keycode {
                            // VirtualKeyCode::Key1 => chip8.keys[0x1] = false,
                            // VirtualKeyCode::Key2 => chip8.keys[0x2] = false,
                            // VirtualKeyCode::Key3 => chip8.keys[0x3] = false,
                            // VirtualKeyCode::Key4 => chip8.keys[0xC] = false,
                            // VirtualKeyCode::Q => chip8.keys[0x4] = false,
                            // VirtualKeyCode::W => chip8.keys[0x5] = false,
                            // VirtualKeyCode::E => chip8.keys[0x6] = false,
                            // VirtualKeyCode::R => chip8.keys[0xD] = false,
                            // VirtualKeyCode::A => chip8.keys[0x7] = false,
                            // VirtualKeyCode::S => chip8.keys[0x8] = false,
                            // VirtualKeyCode::D => chip8.keys[0x9] = false,
                            // VirtualKeyCode::F => chip8.keys[0xE] = false,
                            // VirtualKeyCode::Z => chip8.keys[0xA] = false,
                            // VirtualKeyCode::X => chip8.keys[0x0] = false,
                            // VirtualKeyCode::C => chip8.keys[0xB] = false,
                            // VirtualKeyCode::V => chip8.keys[0xF] = false,
                            _ => {}
                        },
                    },
                    _ => {}
                }
            }
        }
        Event::MainEventsCleared => {
            // let current_time = Instant::now();
            // if current_time.duration_since(cpu_timer) >= CPU_DURATION {
            //     cpu_timer = current_time;
            //     chip8.cycle();
            // }
            //
            // if current_time.duration_since(timer) >= TIMER_DURATION {
            //     log::trace!(
            //         "FPS: {}",
            //         1.0 / (current_time.duration_since(timer).as_secs_f64())
            //     );
            //     timer = current_time;
            //     chip8.timer();
            //     match renderer.render(&chip8.display, 0xFF00FF00, 0) {
            //         Ok(_) => {}
            //         Err(wgpu::SurfaceError::Lost) => renderer.resize(None),
            //         Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            //         Err(e) => eprintln!("{:?}", e),
            //     }
            // }
            // std::thread::sleep(Duration::from_nanos(1_300_000));
            // i8080.emulate();
            // i8080.print();
            match renderer.render(&i8080, 0xFF00FF00, 0) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => renderer.resize(None),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
            }
        }
        _ => {}
    });
}
