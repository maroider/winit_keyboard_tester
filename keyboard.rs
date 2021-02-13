//! Test program for keyboard input.
//!
//! This program will print out markdown tables of sequences of key events.
//!
//! It will automatically terminate a table when it can see that all buttons have been released.
//! You can press the middle mouse button to terminate the table manually.
//! When the current table is empty, the middle mouse button can be used to switch between manual
//! and automatic mode. Manual mode is indicated in the title bar.

use std::collections::HashMap;

use simple_logger::SimpleLogger;
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, NativeKeyCode},
    window::WindowBuilder,
};

mod column {
    pub const NUMBER: &str = "Number";
    pub const KIND: &str = "Kind";
    pub const SYNTH: &str = "Synth";
    pub const STATE: &str = "State";
    pub const KEY_CODE: &str = "KeyCode";
    pub const KEY: &str = "Key";
    pub const LOCATION: &str = "Location";
    pub const TEXT: &str = "Text";
    pub const MODIFIERS: &str = "Modifiers";
    pub const KEY_NO_MOD: &str = "Key (no modifiers)";
    pub const TEXT_ALL_MODS: &str = "Text (all modifiers)";
    pub const SCAN_CODE: &str = "Scancode";
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let event_loop = EventLoop::new();

    let base_window_title = "A fantastic window!";
    let window = WindowBuilder::new()
        .with_title(base_window_title)
        .build(&event_loop)
        .unwrap();

    #[rustfmt::skip]
    let table = {
        let mut table = Table::new();
        table.add_column(TableColumn {
            header: column::NUMBER.to_string()       , normal_width: 0 , extended_width: 0 , use_extended_width: false, enabled: true,
        });
        table.add_column(TableColumn {
            header: column::KIND.to_string()         , normal_width: 6 , extended_width: 0 , use_extended_width: false, enabled: true,
        });
        table.add_column(TableColumn {
            header: column::SYNTH.to_string()        , normal_width: 5 , extended_width: 0 , use_extended_width: false, enabled: true,
        });
        table.add_column(TableColumn {
            header: column::STATE.to_string()        , normal_width: 8 , extended_width: 0 , use_extended_width: false, enabled: true,
        });
        table.add_column(TableColumn {
            header: column::KEY_CODE.to_string()     , normal_width: 20, extended_width: 37, use_extended_width: false, enabled: true,
        });
        table.add_column(TableColumn {
            header: column::KEY.to_string()          , normal_width: 25, extended_width: 42, use_extended_width: true , enabled: true,
        });
        table.add_column(TableColumn {
            header: column::LOCATION.to_string()     , normal_width: 0 , extended_width: 0 , use_extended_width: false, enabled: true,
        });
        table.add_column(TableColumn {
            header: column::TEXT.to_string()         , normal_width: 12, extended_width: 0 , use_extended_width: false, enabled: true,
        });
        table.add_column(TableColumn {
            header: column::MODIFIERS.to_string()    , normal_width: 0 , extended_width: 0 , use_extended_width: false, enabled: true,
        });
        table.add_column(TableColumn {
            header: column::KEY_NO_MOD.to_string()   , normal_width: 25, extended_width: 42, use_extended_width: false, enabled: true,
        });
        table.add_column(TableColumn {
            header: column::TEXT_ALL_MODS.to_string(), normal_width: 0 , extended_width: 0 , use_extended_width: false, enabled: true,
        });
        table.add_column(TableColumn {
            header: column::SCAN_CODE.to_string()    , normal_width: 0 , extended_width: 0 , use_extended_width: false, enabled: false,
        });
        table
    };

    let mut focused = true;
    let mut event_number = 0u16;
    let mut pressed_count = 0i32;
    let mut manual_mode = false;

    table.print_headers();

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
                    table
                        .print_table_line()
                        .column(column::NUMBER, event_number)
                        .column(column::KIND, "Window")
                        .column(column::SYNTH, is_synthetic)
                        .column_with(column::STATE, || format!("{:?}", event.state))
                        .column_with(column::KEY_CODE, || key_code_to_string(event.physical_key))
                        .column_with(column::KEY, || key_to_string(event.logical_key))
                        .column_with(column::LOCATION, || format!("{:?}", event.location))
                        .column_with(column::TEXT, || {
                            event.text.map(nice_text).unwrap_or_else(|| "".to_string())
                        })
                        .column_with(column::KEY_NO_MOD, || key_without_modifiers(&event))
                        .column_with(column::TEXT_ALL_MODS, || text_with_all_modifiers(&event))
                        .print();

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
                if !event.repeat && (focused || pressed_count > 0) {
                    table
                        .print_table_line()
                        .column(column::NUMBER, event_number)
                        .column(column::KIND, "Device")
                        .column_with(column::STATE, || format!("{:?}", event.state))
                        .column_with(column::KEY_CODE, || key_code_to_string(event.physical_key))
                        .print();

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
                if event_number != 0 {
                    table
                        .print_table_line()
                        .column(column::NUMBER, event_number)
                        .column(column::KIND, "ModC")
                        .column_with(column::MODIFIERS, || {
                            format!("{:?}", modifiers).replace("|", "")
                        })
                        .print();

                    event_number += 1;
                }
            }
            Event::WindowEvent {
                event: WindowEvent::ReceivedImeText(text),
                ..
            } => {
                table
                    .print_table_line()
                    .column(column::NUMBER, event_number)
                    .column(column::KIND, "IME")
                    .column_with(column::TEXT, || format!("{:?}", text))
                    .print();

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
                        table.print_headers();
                        event_number = 0;
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
                    table.print_headers();
                    event_number = 0;
                }
            }
        }
    });
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

#[cfg(not(arch = "wasm32"))]
fn key_without_modifiers(event: &KeyEvent) -> String {
    use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
    format!("{:?}", event.key_without_modifiers())
}

#[cfg(arch = "wasm32")]
fn key_without_modifiers(event: &KeyEvent) -> &'static str {
    ""
}

#[cfg(not(arch = "wasm32"))]
fn text_with_all_modifiers(event: &KeyEvent) -> String {
    use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
    event
        .text_with_all_modifers()
        .map(nice_text)
        .unwrap_or_else(String::new)
}

#[cfg(arch = "wasm32")]
fn text_with_all_modifiers(event: &KeyEvent) -> &'static str {
    ""
}

#[cfg(arch = "wasm32")]
fn text_with_all_modifiers(event: &KeyEvent) -> &'static str {
    ""
}

fn nice_text(text: &str) -> String {
    if text.chars().any(|c| c.is_control()) {
        format!("{:?}", text)
    } else {
        text.to_string()
    }
}

struct TableLinePrinter<'a> {
    table: &'a Table,
    column_values: HashMap<String, String>,
}

impl<'a> TableLinePrinter<'a> {
    fn new(table: &'a Table) -> Self {
        Self {
            table,
            column_values: HashMap::new(),
        }
    }

    fn column<T>(mut self, column: &str, value: T) -> Self
    where
        T: ToString,
    {
        if let Some(col) = self.table.columns.iter().find(|col| col.header == column) {
            if col.enabled {
                self.column_values
                    .insert(column.to_string(), value.to_string());
            }
        }
        self
    }

    fn column_with<F: FnOnce() -> T, T>(mut self, column: &str, f: F) -> Self
    where
        T: ToString,
    {
        if let Some(col) = self.table.columns.iter().find(|col| col.header == column) {
            if col.enabled {
                self.column_values
                    .insert(column.to_string(), f().to_string());
            }
        }
        self
    }

    fn print(self) {
        use std::io::{self, Write as _};
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        for column in self.table.columns.iter() {
            if !column.enabled {
                continue;
            }
            write!(
                &mut stdout,
                "| {:<length$} ",
                self.column_values
                    .get(&column.header)
                    .map(AsRef::as_ref)
                    .unwrap_or(""),
                length = column.width(),
            )
            .unwrap();
        }
        writeln!(&mut stdout, "|").unwrap();

        stdout.flush().unwrap();
    }
}

struct Table {
    columns: Vec<TableColumn>,
}

impl Table {
    fn new() -> Self {
        Self {
            columns: Vec::new(),
        }
    }

    fn add_column(&mut self, column: TableColumn) {
        self.columns.push(column);
    }

    fn print_headers(&self) {
        use std::io::{self, Write as _};
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        writeln!(&mut stdout).unwrap();

        for column in self.columns.iter() {
            if !column.enabled {
                continue;
            }

            write!(
                &mut stdout,
                "| {:<length$} ",
                column.header,
                length = column.width(),
            )
            .unwrap();
        }

        writeln!(&mut stdout, "|").unwrap();

        for column in self.columns.iter() {
            if !column.enabled {
                continue;
            }

            let mut buf = String::new();
            for _ in 0..column.width() {
                buf.push('-');
            }
            write!(&mut stdout, "| {} ", buf).unwrap();
        }

        writeln!(&mut stdout, "|").unwrap();

        stdout.flush().unwrap();
    }

    fn print_table_line<'a>(&'a self) -> TableLinePrinter<'a> {
        TableLinePrinter::new(self)
    }
}

struct TableColumn {
    header: String,
    normal_width: usize,
    extended_width: usize,
    use_extended_width: bool,
    enabled: bool,
}

impl TableColumn {
    fn width(&self) -> usize {
        if self.use_extended_width {
            self.extended_width
        } else {
            self.normal_width
        }
        .max(self.header.len())
    }
}
