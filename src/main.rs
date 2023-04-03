// Don't show console window on release mode on Windows
// (or, technically, when debug assertions are disabled)
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]

mod prelude;
use prelude::*;

#[macro_use]
extern crate log;
extern crate chrono;
extern crate glium;
extern crate simplelog;

use chrono::Local;
use glium::{
    glutin::{
        self,
        dpi::{LogicalPosition, LogicalSize},
        event::VirtualKeyCode,
    },
    Surface,
};
use simplelog::{
    format_description, CombinedLogger, ConfigBuilder, WriteLogger
};
#[cfg(debug_assertions)]
use simplelog::{
    ColorChoice, TermLogger, TerminalMode
};
use std::{
    fs::{self, File},
    process::exit,
};
use time::UtcOffset;

fn main() {
    init_log();

    info!("Creating window");
    let event_loop = glutin::event_loop::EventLoop::new();

    // Calculate center position for window
    let window_size = LogicalSize::new(1280, 600);
    let monitor_result = event_loop.primary_monitor();
    match monitor_result {
        Some(_) => (),
        None => {
            error!("No monitor available! Launch cannot proceed.");
            exit(-1);
        }
    }
    let monitor = monitor_result.unwrap();
    let monitor_size: LogicalSize<u32> = monitor.size().to_logical(monitor.scale_factor());
    let window_position = LogicalPosition::new(
        monitor_size.width / 2 - window_size.width / 2,
        monitor_size.height / 2 - window_size.height / 2,
    );

    // Create window
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Rust In Peace")
        .with_inner_size(window_size)
        .with_position(window_position);
    let cb = glutin::ContextBuilder::new();
    let display_result = glium::Display::new(wb, cb, &event_loop);
    match display_result {
        Ok(_) => (),
        Err(error) => {
            error!("Failed to create window! Error: {error}");
            exit(-1);
        }
    }
    let display = display_result.unwrap();
    info!("Successfully created window");

    event_loop.run(move |ev, _, control_flow| {
        let max_fps: u64 = 60;
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(1_000_000_000 / max_fps);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    info!("Close event received, shutting down");
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                glutin::event::WindowEvent::KeyboardInput {
                    device_id: _,
                    input,
                    is_synthetic: _,
                } => match input.virtual_keycode {
                    Some(keycode) => match keycode {
                        VirtualKeyCode::Escape => {
                            info!("Escape pressed, shutting down");
                            *control_flow = glutin::event_loop::ControlFlow::Exit;
                            return;
                        }
                        _ => (),
                    },
                    None => (),
                },
                _ => return,
            },
            glutin::event::Event::MainEventsCleared => {
                // Update and draw here
                let mut target = display.draw();
                target.clear_color(0.2, 0.3, 1.0, 1.0);
                target.finish().unwrap();
            }
            _ => (),
        }
    });
}

fn init_log() {
    // Init log
    // First, name and create the file. Ensure directory exists.
    match fs::create_dir_all("Logs/") {
        Ok(_) => (),
        Err(_) => println!("Failed to create logs directory! The file creation may error."),
    }
    let time_now = Local::now();
    let log_file_name =
        "Logs/".to_owned() + &time_now.format("rip_%Y-%m-%d_%H-%M-%S").to_string() + ".log";
    let log_file = File::create(log_file_name);

    // If file was created, create our loggers. If not, print error and exit.
    let log_config = ConfigBuilder::new()
        .set_time_format_custom(format_description!(
            "[hour]:[minute]:[second].[subsecond digits:3]"
        ))
        .set_time_offset(
            UtcOffset::from_whole_seconds(time_now.offset().local_minus_utc())
                .expect("UTC offset should be valid"),
        )
        .build();
    match log_file {
        Ok(file) => {
            CombinedLogger::init(vec![
                #[cfg(debug_assertions)] // Only create terminal logger on debug builds
                TermLogger::new(
                    LevelFilter::Info,
                    log_config.clone(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(LevelFilter::Trace, log_config, file),
            ])
            .unwrap();
        }
        Err(error) => {
            println!("Failed to create log file! The application will now exit. Error: {error}");
            exit(-1);
        }
    }

    info!(
        "Rust In Peace - Session started on {} at {}",
        time_now.format("%Y/%m/%d"),
        time_now.format("%H:%M:%S")
    );
}
