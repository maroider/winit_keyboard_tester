use std::{
    collections::HashMap,
    iter,
    time::{Duration, Instant},
};

use softbuffer::GraphicsContext;
use unicode_width::UnicodeWidthStr;
use winit::{
    event::{DeviceEvent, ElementState, Event, Ime, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, ModifiersState},
    window::WindowBuilder,
};

#[allow(dead_code)]
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

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn wasm_main() {
        console_log::init_with_level(log::Level::Debug).unwrap();

        super::main();
    }
}

const TABLE_TIMEOUT: Duration = Duration::from_secs(5);

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    simple_logger::SimpleLogger::new().init().unwrap();
    let event_loop = EventLoop::new();

    let base_window_title = "A fantastic window!";
    let window = WindowBuilder::new()
        .with_title(base_window_title)
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();
    let mut graphics_context = unsafe { GraphicsContext::new(&window, &window) }.unwrap();

    #[rustfmt::skip]
    let table = {
        let mut table = Table::new();
        table.add_column(TableColumn { header: column::NUMBER       , normal_width: 0 , extended_width: 0 , use_extended_width: false, enabled: true , });
        table.add_column(TableColumn { header: column::KIND         , normal_width: 6 , extended_width: 0 , use_extended_width: false, enabled: true , });
        table.add_column(TableColumn { header: column::SYNTH        , normal_width: 5 , extended_width: 0 , use_extended_width: false, enabled: true , });
        table.add_column(TableColumn { header: column::STATE        , normal_width: 8 , extended_width: 0 , use_extended_width: false, enabled: true , });
        table.add_column(TableColumn { header: column::KEY_CODE     , normal_width: 20, extended_width: 37, use_extended_width: false, enabled: true , });
        table.add_column(TableColumn { header: column::KEY          , normal_width: 25, extended_width: 42, use_extended_width: true , enabled: true , });
        table.add_column(TableColumn { header: column::LOCATION     , normal_width: 0 , extended_width: 0 , use_extended_width: false, enabled: true , });
        table.add_column(TableColumn { header: column::TEXT         , normal_width: 12, extended_width: 0 , use_extended_width: false, enabled: true , });
        table.add_column(TableColumn { header: column::MODIFIERS    , normal_width: 11, extended_width: 11, use_extended_width: false, enabled: true , });
        #[cfg(not(target_arch = "wasm32"))]
        {
        table.add_column(TableColumn { header: column::KEY_NO_MOD   , normal_width: 25, extended_width: 42, use_extended_width: false, enabled: true , });
        table.add_column(TableColumn { header: column::TEXT_ALL_MODS, normal_width: 0 , extended_width: 0 , use_extended_width: false, enabled: true , });
        table.add_column(TableColumn { header: column::SCAN_CODE    , normal_width: 0 , extended_width: 0 , use_extended_width: false, enabled: false, });
        }
        table
    };

    #[cfg(target_arch = "wasm32")]
    let mut table_printer = {
        use winit::platform::web::WindowExtWebSys;

        let canvas = optional_gl.window().canvas();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        body.append_child(&canvas)
            .expect("Append canvas to HTML body");

        HtmlTablePrinter::new(document, &body, &table)
    };

    #[cfg(not(target_arch = "wasm32"))]
    let mut table_printer = StdoutTablePrinter::new();

    let mut raw_keys_pressed = HashMap::new();
    let mut repeated_keys = HashMap::new();

    let mut focused = true;
    let mut event_number = 0u16;
    let mut pressed_count = 0i32;
    let mut modifiers = Default::default();
    let mut manual_mode = false;

    table_printer.begin_new_table(&table);

    let mut last_change = Instant::now();
    let mut skip_timeout = false;
    let mut size = window.inner_size();
    let mut screen_buf: Vec<u32> = iter::repeat(u32::MAX)
        .take(size.width as usize * size.height as usize)
        .collect();
    window.set_resizable(true);

    event_loop.run(move |event, _, control_flow| {
        let now = Instant::now();
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Focused(focus),
                ..
            } => {
                if event_number > 0 {
                    table
                        .print_table_line()
                        .column(column::NUMBER, event_number)
                        .column(column::KIND, "Focus")
                        .column(column::STATE, if focus { "Received" } else { "Lost" })
                        .print(&mut table_printer);
                    event_number += 1;
                }
                focused = focus;
                last_change = now;
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
                let table = table
                    .print_table_line()
                    .column(column::NUMBER, event_number)
                    .column(column::KIND, "Window")
                    .column(column::SYNTH, is_synthetic)
                    .column_with(column::KEY_CODE, || key_code_to_string(&event.physical_key))
                    .column_with(column::KEY, || key_to_string(&event.logical_key))
                    .column_with(column::LOCATION, || format!("{:?}", event.location))
                    .column_with(column::TEXT, || {
                        event
                            .text
                            .as_ref()
                            .map(nice_text)
                            .unwrap_or_else(|| "".to_string())
                    })
                    .column_with(column::KEY_NO_MOD, || key_without_modifiers(&event))
                    .column_with(column::TEXT_ALL_MODS, || text_with_all_modifiers(&event));

                if !event.repeat {
                    table
                        .column_with(column::STATE, || format!("{:?}", event.state))
                        .print(&mut table_printer);

                    event_number += 1;

                    match event.state {
                        ElementState::Pressed => pressed_count += 1,
                        ElementState::Released => {
                            repeated_keys.remove(&event.physical_key);
                            pressed_count -= 1
                        }
                    }
                } else {
                    let repeat_count = { repeated_keys.entry(event.physical_key).or_insert(1) };
                    if *repeat_count == 1 {
                        event_number += 1;
                    }
                    table
                        .column_with(column::STATE, || format!("Rpt {:>4}", repeat_count))
                        .update(&mut table_printer);
                    *repeat_count += 1;
                }

                last_change = now;
            }
            Event::DeviceEvent {
                event: DeviceEvent::Key(event),
                ..
            } => {
                if focused || pressed_count > 0 {
                    let repeat_count = match event.state {
                        ElementState::Pressed => Some(
                            raw_keys_pressed
                                .entry(event.physical_key.clone())
                                .or_insert_with(|| {
                                    pressed_count += 1;
                                    0
                                }),
                        ),
                        ElementState::Released => {
                            if raw_keys_pressed.remove(&event.physical_key).is_some() {
                                pressed_count -= 1;
                            }
                            None
                        }
                    };

                    let table = table
                        .print_table_line()
                        .column(column::NUMBER, event_number)
                        .column(column::KIND, "Device")
                        .column_with(column::KEY_CODE, || key_code_to_string(&event.physical_key));

                    let print_normal = if let Some(repeat_count) = repeat_count {
                        if *repeat_count > 0 {
                            table
                                .clone() // TODO: Get rid of this clone
                                .column_with(column::STATE, || format!("Rpt {:>4}", repeat_count))
                                .update(&mut table_printer);
                            false
                        } else {
                            true
                        }
                    } else {
                        true
                    };
                    if print_normal {
                        table
                            .column_with(column::STATE, || format!("{:?}", event.state))
                            .print(&mut table_printer);
                        event_number += 1;
                    }
                }

                last_change = now;
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(new_modifiers),
                ..
            } => {
                modifiers = new_modifiers;
                if !modifiers.is_empty() || event_number != 0 {
                    table
                        .print_table_line()
                        .column(column::NUMBER, event_number)
                        .column(column::KIND, "ModC")
                        .column_with(column::MODIFIERS, || format_modifiers(modifiers))
                        .print(&mut table_printer);

                    event_number += 1;
                }

                last_change = now;
            }
            Event::WindowEvent {
                event: WindowEvent::Ime(ime),
                ..
            } => {
                // TODO: Print this in a better way...
                let mut row = table
                    .print_table_line()
                    .column(column::NUMBER, event_number)
                    .column(column::KIND, "IME")
                    .column(
                        column::STATE,
                        match ime {
                            Ime::Enabled => "Enabled",
                            Ime::Preedit(_, _) => "Preedit",
                            Ime::Commit(_) => "Commit",
                            Ime::Disabled => "Disabled",
                        },
                    );
                match ime {
                    // TODO: Print preedit position?
                    Ime::Preedit(text, _) | Ime::Commit(text) => {
                        row = row.column_with(column::TEXT, || format!("{:?}", text));
                    }
                    Ime::Enabled | Ime::Disabled => {}
                }
                row.print(&mut table_printer);

                event_number += 1;

                last_change = now;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button,
                        ..
                    },
                ..
            } => match button {
                MouseButton::Middle => {
                    if manual_mode {
                        if event_number == 0 {
                            manual_mode = false;
                            // TODO: Move this elsewhere?
                            window.set_title(base_window_title);
                        } else {
                            table_printer.begin_new_table(&table);
                            event_number = 0;
                            pressed_count = 0;
                            raw_keys_pressed.clear();
                            repeated_keys.clear();
                            modifiers = Default::default();
                        }
                    } else {
                        if event_number == 0 {
                            manual_mode = true;
                            // TODO: Move this elsewhere?
                            window.set_title(&format!("{} - Manual Mode", base_window_title));
                        } else {
                            pressed_count = 0;
                            skip_timeout = true;
                            modifiers = Default::default();
                        }
                    }

                    last_change = now;
                }
                MouseButton::Right => {
                    window.reset_dead_keys();
                    table
                        .print_table_line()
                        .column(column::NUMBER, event_number)
                        .column(column::KIND, "DeadRST")
                        .print(&mut table_printer);
                    event_number += 1;

                    last_change = now;
                }
                _ => {}
            },
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                let new_area = new_size.width as usize * new_size.height as usize;
                screen_buf.resize_with(new_area, || u32::MAX);
                size = new_size;
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                if !manual_mode {
                    *control_flow = ControlFlow::Exit
                }
            }
            Event::RedrawRequested(_) => {
                graphics_context.set_buffer(
                    &screen_buf,
                    size.width.min(u16::MAX.into()) as u16,
                    size.height.min(u16::MAX.into()) as u16,
                );
            }
            _ => (),
        }

        if !manual_mode {
            if pressed_count == 0 && modifiers.is_empty() {
                if event_number != 0 {
                    if last_change + TABLE_TIMEOUT <= now || skip_timeout {
                        print!("\r{:30}", "");
                        table_printer.begin_new_table(&table);
                        event_number = 0;
                        skip_timeout = false;
                        *control_flow = ControlFlow::Wait;
                    } else {
                        print!(
                            "\rTable finishes in {}s",
                            (TABLE_TIMEOUT - now.duration_since(last_change)).as_secs()
                        );
                    }
                }
            }
        }
    });
}

fn key_to_string(key: &Key) -> String {
    match key {
        Key::Unidentified(native_key) => format!("Unidentified({:?})", native_key),
        _ => format!("{:?}", key),
    }
}

fn key_code_to_string(code: &KeyCode) -> String {
    match code {
        KeyCode::Unidentified(native_key_code) => format!("Unidentified({:?})", native_key_code),
        _ => format!("{:?}", code),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn key_without_modifiers(event: &KeyEvent) -> String {
    use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
    format!("{:?}", event.key_without_modifiers())
}

#[cfg(target_arch = "wasm32")]
fn key_without_modifiers(_: &KeyEvent) -> &'static str {
    ""
}

#[cfg(not(target_arch = "wasm32"))]
fn text_with_all_modifiers(event: &KeyEvent) -> String {
    use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
    event
        .text_with_all_modifiers()
        .map(nice_text)
        .unwrap_or_else(String::new)
}

#[cfg(target_arch = "wasm32")]
fn text_with_all_modifiers(_: &KeyEvent) -> &'static str {
    ""
}

fn nice_text<S>(text: S) -> String
where
    S: AsRef<str>,
{
    let text = text.as_ref();
    if text.chars().any(|c| c.is_control() || c.is_whitespace()) {
        format!("{:?}", text)
    } else {
        text.to_string()
    }
}

fn format_modifiers(modifiers: ModifiersState) -> String {
    let mut string = String::with_capacity(modifiers.bits().count_ones() as usize * 3);

    if modifiers.contains(ModifiersState::ALT) {
        string.push_str("AL");
    }
    if modifiers.contains(ModifiersState::CONTROL) {
        if !string.is_empty() {
            string.push('|');
        }
        string.push_str("CO");
    }
    if modifiers.contains(ModifiersState::SHIFT) {
        if !string.is_empty() {
            string.push('|');
        }
        string.push_str("SH");
    }
    if modifiers.contains(ModifiersState::SUPER) {
        if !string.is_empty() {
            string.push('|');
        }
        string.push_str("SU");
    }

    string
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

    fn print_table_line(&self) -> RowBuilder<'_> {
        RowBuilder::new(self)
    }
}

struct TableColumn {
    header: &'static str,
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

#[derive(Clone)]
struct RowBuilder<'a> {
    table: &'a Table,
    column_values: HashMap<String, String>,
}

impl<'a> RowBuilder<'a> {
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

    fn print<P: TablePrinter>(self, printer: &mut P) {
        printer.print_row(self)
    }

    fn update<P: TablePrinter>(self, printer: &mut P) {
        printer.update_row(self)
    }
}

trait TablePrinter {
    fn begin_new_table(&mut self, table: &Table);

    fn print_row(&mut self, row: RowBuilder<'_>);

    fn update_row(&mut self, row: RowBuilder<'_>);
}

#[cfg(not(target_arch = "wasm32"))]
struct StdoutTablePrinter {
    updating: bool,
    ioprinter: IoWriteTablePrinter,
}

#[cfg(not(target_arch = "wasm32"))]
impl StdoutTablePrinter {
    fn new() -> Self {
        Self {
            updating: false,
            ioprinter: IoWriteTablePrinter::new(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl TablePrinter for StdoutTablePrinter {
    fn begin_new_table(&mut self, table: &Table) {
        use std::io::{self, Write as _};
        let stdout = io::stdout();
        let mut out = stdout.lock();

        writeln!(out).unwrap();

        self.ioprinter.begin_new_table(table, &mut out);
    }

    fn print_row(&mut self, row: RowBuilder<'_>) {
        use std::io::{self, Write};
        let stdout = io::stdout();
        let mut out = stdout.lock();

        if self.updating {
            write!(out, "\n").unwrap();
            self.updating = false;
        }

        write!(out, "\r").unwrap();

        self.ioprinter.print_row(row, &mut out);

        write!(out, "\n").unwrap();
    }

    fn update_row(&mut self, row: RowBuilder<'_>) {
        use std::io::{self, Write};
        let stdout = io::stdout();
        let mut out = stdout.lock();

        self.updating = true;

        write!(out, "\r").unwrap();
        self.ioprinter.print_row(row, &mut out);
    }
}

#[cfg(target_arch = "wasm32")]
struct HtmlTablePrinter {
    document: web_sys::Document,
    table_container: web_sys::Element,
    table_element: web_sys::Element,
    tbody: web_sys::Element,
    last_table: Option<web_sys::Element>,
    ioprinter: IoWriteTablePrinter,
    markdown_table_buffer: Vec<u8>,
    updating: bool,
}

#[cfg(target_arch = "wasm32")]
impl HtmlTablePrinter {
    fn new(document: web_sys::Document, body: &web_sys::HtmlElement, table: &Table) -> Self {
        let (table_element, tbody) = Self::create_new_table(&document, table);
        let table_container = document.create_element("div").unwrap();
        table_container.set_id("table-container");
        table_container.append_child(&table_element).unwrap();
        body.append_child(&table_container).unwrap();

        Self {
            document,
            table_container,
            table_element,
            tbody,
            last_table: None,
            ioprinter: IoWriteTablePrinter::new(),
            markdown_table_buffer: Vec::new(),
            updating: false,
        }
    }

    fn create_new_table(
        document: &web_sys::Document,
        table: &Table,
    ) -> (web_sys::Element, web_sys::Element) {
        let table_element = document.create_element("table").unwrap();
        let thead = document.create_element("thead").unwrap();
        let header_row = document.create_element("tr").unwrap();
        for column in table.columns.iter() {
            let header = document.create_element("th").unwrap();
            header.set_text_content(Some(&column.header));
            header_row.append_child(&header).unwrap();
        }
        thead.append_child(&header_row).unwrap();
        table_element.append_child(&thead).unwrap();

        let tbody = document.create_element("tbody").unwrap();

        table_element.append_child(&tbody).unwrap();

        (table_element, tbody)
    }
}

#[cfg(target_arch = "wasm32")]
impl TablePrinter for HtmlTablePrinter {
    fn begin_new_table(&mut self, table: &Table) {
        let mardown_table = std::str::from_utf8(&self.markdown_table_buffer)
            .unwrap()
            .to_string();
        self.markdown_table_buffer.clear();
        self.ioprinter
            .begin_new_table(table, &mut self.markdown_table_buffer);

        // TODO: Don't require this hack, maybe.
        if self.tbody.child_element_count() == 0 {
            return;
        }

        let (new_table, new_tbody) = Self::create_new_table(&self.document, table);
        self.table_container
            .replace_child(&new_table, &self.table_element)
            .unwrap();

        let details = self.document.create_element("details").unwrap();
        details.set_attribute("open", "").unwrap();
        let summary = self.document.create_element("summary").unwrap();
        summary.set_text_content(Some("Event table"));
        let button = self.document.create_element("button").unwrap();
        button
            .set_attribute(
                "onclick",
                &format!(r#"navigator.clipboard.writeText(`{}`)"#, mardown_table),
            )
            .unwrap();
        button.set_class_name("copy-to-clipboard");
        button.set_text_content(Some("Copy to clipboard"));
        summary.append_child(&button).unwrap();
        details.append_child(&summary).unwrap();
        details.append_child(&self.table_element).unwrap();
        self.table_container
            .insert_before(&details, self.last_table.as_deref())
            .unwrap();

        self.table_element = new_table;
        self.tbody = new_tbody;
        self.last_table = Some(details);
    }

    fn print_row(&mut self, row: RowBuilder<'_>) {
        let tr = self.document.create_element("tr").unwrap();
        self.fill_row(&tr, row);
        self.tbody.append_child(&tr).unwrap();
        self.updating = false;
    }

    fn update_row(&mut self, row: RowBuilder<'_>) {
        if self.updating {
            if let Some(tr) = self.tbody.last_element_child() {
                while let Some(last_child) = tr.last_element_child() {
                    tr.remove_child(last_child.as_ref()).unwrap();
                }
                self.fill_row(&tr, row)
            }
        } else {
            self.print_row(row);
            self.updating = true;
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl HtmlTablePrinter {
    fn fill_row(&mut self, tr: &web_sys::Element, row: RowBuilder<'_>) {
        for column in row.table.columns.iter() {
            if !column.enabled {
                continue;
            }
            let td = self.document.create_element("td").unwrap();
            if let Some(value) = row.column_values.get(&column.header) {
                td.set_text_content(Some(value));
            }
            tr.append_child(&td).unwrap();
        }

        self.ioprinter
            .print_row(row, &mut self.markdown_table_buffer);
    }
}

struct IoWriteTablePrinter {}

impl IoWriteTablePrinter {
    fn new() -> Self {
        Self {}
    }
}

impl IoWriteTablePrinter {
    fn begin_new_table<W>(&mut self, table: &Table, out: &mut W)
    where
        W: std::io::Write,
    {
        for column in table.columns.iter() {
            if !column.enabled {
                continue;
            }

            write!(
                out,
                "| {:<length$} ",
                column.header,
                length = column.width(),
            )
            .unwrap();
        }

        writeln!(out, "|").unwrap();

        for column in table.columns.iter() {
            if !column.enabled {
                continue;
            }

            let mut buf = String::new();
            for _ in 0..column.width() {
                buf.push('-');
            }
            write!(out, "| {} ", buf).unwrap();
        }

        writeln!(out, "|").unwrap();

        out.flush().unwrap();
    }

    fn print_row<W>(&mut self, row: RowBuilder<'_>, out: &mut W)
    where
        W: std::io::Write,
    {
        for column in row.table.columns.iter() {
            if !column.enabled {
                continue;
            }
            let content = row
                .column_values
                .get(column.header)
                .map(AsRef::as_ref)
                .unwrap_or("");
            let content_width = content.width();
            let padding = column.width().saturating_sub(content_width);
            write!(out, "| {content}{:padding$} ", "").unwrap();
        }
        write!(out, "|").unwrap();

        out.flush().unwrap();
    }
}
