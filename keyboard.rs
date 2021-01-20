//! Test program for keyboard input.
//!
//! This program will print out markdown tables of sequences of key events.
//!
//! It will automatically terminate a table when it can see that all buttons have been released.
//! You can press the middle mouse button to terminate the table manually.
//! When the current table is empty, the middle mouse button can be used to switch between manual
//! and automatic mode. Manual mode is indicated in the title bar.

use simple_logger::SimpleLogger;
use winit::{
    event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, NativeKeyCode},
    window::WindowBuilder,
};

const EXPECT_UNIDENTIFIED: bool = true;
const PKL: usize = if EXPECT_UNIDENTIFIED { 37 } else { 20 };
const LKL: usize = if EXPECT_UNIDENTIFIED { 42 } else { 25 };

macro_rules! print_table_line {
    (
        $event_number:expr,
        $kind:expr,
        $is_synthetic:expr,
        $state:expr,
        $physical_key:expr,
        $logical_key:expr,
        $location:expr,
        $text:expr,
        $modifiers:expr
        $(,)?
    ) => {
        println!(
            "| {:<6} | {:<6} | {:<5} | {:<8} | {:<pkl$} | {:<lkl$} | {:<8} | {:<16} | {:<26} |",
            $event_number,
            $kind,
            $is_synthetic,
            $state,
            $physical_key,
            $logical_key,
            $location,
            $text,
            $modifiers,
            pkl = PKL,
            lkl = LKL,
        );
    };
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let event_loop = EventLoop::new();

    let base_window_title = "A fantastic window!";
    let window = WindowBuilder::new()
        .with_title(base_window_title)
        .build(&event_loop)
        .unwrap();

    let mut focused = true;

    let mut event_number = 0u16;

    // let mut table_header_printed = false;
    let mut pressed_count = 0i32;

    let mut manual_mode = false;

    begin_new_table();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Focused(focus),
                ..
            } => {
                focused = focus;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event,
                        is_synthetic,
                        ..
                    },
                ..
            } => {
                if !event.repeat {
                    print_table_line!(
                        event_number,
                        "Window",
                        is_synthetic,
                        format!("{:?}", event.state),
                        key_code_to_string(event.physical_key),
                        key_to_string(event.logical_key),
                        format!("{:?}", event.location),
                        format!("{:?}", event.text),
                        "",
                    );

                    event_number += 1;

                    match event.state {
                        ElementState::Pressed => pressed_count += 1,
                        ElementState::Released => pressed_count -= 1,
                    }
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::Key(event),
                ..
            } => {
                if focused || pressed_count > 0 {
                    print_table_line!(
                        event_number,
                        "Device",
                        "",
                        format!("{:?}", event.state),
                        key_code_to_string(event.physical_key),
                        "",
                        "",
                        "",
                        "",
                    );

                    event_number += 1;

                    match event.state {
                        ElementState::Pressed => pressed_count += 1,
                        ElementState::Released => pressed_count -= 1,
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(modifiers),
                ..
            } => {
                print_table_line!(
                    event_number,
                    "ModC",
                    "",
                    "",
                    "",
                    "",
                    "",
                    "",
                    format!("{:?}", modifiers).replace("|", ""),
                );
                event_number += 1;
            }
            Event::WindowEvent {
                event: WindowEvent::ReceivedImeText(text),
                ..
            } => {
                print_table_line!(
                    event_number,
                    "IME",
                    "",
                    "",
                    "",
                    "",
                    "",
                    format!("{:?}", text),
                    "",
                );

                event_number += 1;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button: MouseButton::Middle,
                        ..
                    },
                ..
            } => {
                if manual_mode {
                    if event_number == 0 {
                        manual_mode = false;
                    } else {
                        begin_new_table();
                        event_number = 0;
                        // table_header_printed = true;
                        pressed_count = 0;
                    }
                } else {
                    if event_number == 0 {
                        manual_mode = true;
                    } else {
                        pressed_count = 0;
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                if !manual_mode {
                    *control_flow = ControlFlow::Exit
                }
            }
            Event::MainEventsCleared => {
                if manual_mode {
                    window.set_title(&format!("{} - Manual Mode", base_window_title));
                } else {
                    window.set_title(base_window_title);
                }
            }
            _ => (),
        }

        if !manual_mode {
            if pressed_count == 0 {
                if event_number != 0 {
                    begin_new_table();
                    event_number = 0;
                }
            }
        }
    });
}

fn begin_new_table() {
    println!();
    println!(
        "| Number | Kind   | Synth | State    | KeyCode{: <pkl$} | Key{: <lkl$} | Location | Text             | Modifiers                  |",
        "",
        "",
        pkl = PKL - 7,
        lkl = LKL - 3,
    );
    println!(
        "| ------ | ------ | ----- | -------- | {:-<pkl$} | {:-<lkl$} | -------- | ---------------- | -------------------------- |",
        "-",
        "-",
        pkl = PKL,
        lkl = LKL,
    );
}

fn key_to_string(key: Key) -> String {
    match key {
        Key::Unidentified(native_key_code) => format!(
            "Unidentified({})",
            native_key_code_to_string(native_key_code)
        ),
        _ => format!("{:?}", key),
    }
}

fn key_code_to_string(code: KeyCode) -> String {
    match code {
        KeyCode::Unidentified(native_key_code) => format!(
            "Unidentified({})",
            native_key_code_to_string(native_key_code)
        ),
        _ => format!("{:?}", code),
    }
}

fn native_key_code_to_string(native_key_code: NativeKeyCode) -> String {
    match native_key_code {
        winit::keyboard::NativeKeyCode::Windows(scancode) => {
            format!("Windows({:#X})", scancode as u32)
        }
        winit::keyboard::NativeKeyCode::MacOS(keycode) => {
            format!("MacOS({:#X})", keycode)
        }
        winit::keyboard::NativeKeyCode::XKB(keycode) => {
            format!("XKB({:#X})", keycode)
        }
        _ => unimplemented!(),
    }
}
