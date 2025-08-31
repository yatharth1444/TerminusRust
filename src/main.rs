use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder};
use glutin::display::{Display, DisplayApiPreference};
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use nix::pty::{forkpty, ForkptyResult, Winsize};
use nix::unistd::execvp;
use std::ffi::CString;
use std::io::{Read, Write};
use std::num::NonZeroU32;
use std::os::fd::FromRawFd;
use std::os::unix::io::RawFd;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    // Setup event loop and window
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Rust Terminal MVP");
    let window = wb.build(&event_loop).unwrap();

    // Setup GL display and context
    let raw_display = unsafe {
        glutin::platform::unix::XlibDisplay::new(window.xlib_display().unwrap())
            .expect("Failed to create raw display")
    };
    let display = Display::new(raw_display, DisplayApiPreference::Egl).unwrap();

    // Choose config
    let config_template = ConfigTemplateBuilder::new().build();
    let config = unsafe { display.find_configs(config_template) }
        .unwrap()
        .next()
        .expect("No GL configs");

    // Create GL context
    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(None))
        .build(Some(window.xlib_window().unwrap()));

    let not_current_context =
        unsafe { display.create_context(&config, &context_attributes) }.unwrap();

    // Obtain window size and create surface attributes
    let window_size = window.inner_size();
    let width = NonZeroU32::new(window_size.width.max(1)).unwrap();
    let height = NonZeroU32::new(window_size.height.max(1)).unwrap();

    let surface_attributes =
        SurfaceAttributesBuilder::<WindowSurface>::new()
            .build(window.xlib_window().unwrap(), width, height);

    let surface = unsafe { display.create_window_surface(&config, &surface_attributes) }.unwrap();

    let context = not_current_context.make_current(&surface).unwrap();

    let (master_fd, _shell_pid) = spawn_shell();

    let buffer = Arc::new(Mutex::new(Vec::new()));
    let buffer_clone = buffer.clone();

    thread::spawn(move || read_pty_output(master_fd, buffer_clone));

    // Main event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        if input.state == ElementState::Pressed {
                            handle_keyboard_key(key, master_fd);
                        }
                    }
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                render(&buffer.lock().unwrap());
                surface.swap_buffers(&context).unwrap();
            }
            _ => (),
        }

        window.request_redraw();
    });
}

fn spawn_shell() -> (RawFd, nix::unistd::Pid) {
    let winsize = Winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let ForkptyResult { master, fork_result } = unsafe { forkpty(Some(&winsize), None).unwrap() };

    match fork_result {
        nix::unistd::ForkResult::Parent { child } => (master, child),
        nix::unistd::ForkResult::Child => {
            let shell = CString::new("/bin/bash").unwrap();
            execvp(&shell, &[shell.clone()]).unwrap();
            unreachable!();
        }
    }
}

fn read_pty_output(master_fd: RawFd, buffer: Arc<Mutex<Vec<u8>>>) {
    let mut file = unsafe { std::fs::File::from_raw_fd(master_fd) };
    let mut buf = [0u8; 1024];
    loop {
        match file.read(&mut buf) {
            Ok(size) if size > 0 => {
                if let Ok(mut b) = buffer.lock() {
                    b.extend_from_slice(&buf[..size]);
                }
            }
            _ => thread::sleep(Duration::from_millis(10)),
        }
    }
}

fn handle_keyboard_key(key: VirtualKeyCode, master_fd: RawFd) {
    let mut file = unsafe { std::fs::File::from_raw_fd(master_fd) };
    let c = match key {
        VirtualKeyCode::Return => b"\n",
        VirtualKeyCode::Back => b"\x08",
        VirtualKeyCode::Escape => b"\x1b",
        VirtualKeyCode::Space => b" ",
        _ => return,
    };
    let _ = file.write(c);
}

fn render(buffer: &Vec<u8>) {
    println!("{}", String::from_utf8_lossy(buffer));
}
