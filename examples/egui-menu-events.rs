#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_os = "linux"))]
use std::{cell::RefCell, rc::Rc};

use eframe::egui;
use tray_icon::{menu::Menu, menu::MenuEvent, TrayIconBuilder, TrayIconEvent};

fn main() -> Result<(), eframe::Error> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/icon.png");
    let icon = load_icon(std::path::Path::new(path));

    // Since egui uses winit under the hood and doesn't use gtk on Linux, and we need gtk for
    // the tray icon to show up, we need to spawn a thread
    // where we initialize gtk and create the tray_icon
    #[cfg(target_os = "linux")]
    std::thread::spawn(|| {
        let tray_menu = Menu::new();
        let menu_item2 = MenuItem::new("Menu item #2", false, None);
        let submenu = Submenu::with_items(
            "Submenu Outer",
            true,
            &[
                &MenuItem::new(
                    "Menu item #1",
                    true,
                    Some(Accelerator::new(Some(Modifiers::ALT), Code::KeyD)),
                ),
                &PredefinedMenuItem::separator(),
                &menu_item2,
                &MenuItem::new("Menu item #3", true, None),
                &PredefinedMenuItem::separator(),
                &Submenu::with_items(
                    "Submenu Inner",
                    true,
                    &[
                        &MenuItem::new("Submenu item #1", true, None),
                        &PredefinedMenuItem::separator(),
                        &menu_item2,
                    ],
                ),
            ],
        );

        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_menu(Box::new(menu_item2))
            .with_menu(Box::new(sub_menu))
            .with_tooltip("system-tray - tray icon library!")
            .with_icon(icon)
            .build()
            .unwrap();
        gtk::init().unwrap();
        #[cfg(target_os = "windows")]
        unsafe {
            _tray_icon.init_for_hwnd(window.hwnd() as isize)
        };
        #[cfg(target_os = "linux")]
        _tray_icon.init_for_gtk_window(&gtk_window, Some(&vertical_gtk_box));
        #[cfg(target_os = "macos")]
        _tray_icon.init_for_nsapp();

        //let _tray_icon = TrayIconBuilder::new()
        //    .with_menu(Box::new(Menu::new()))
        //    .with_icon(icon)
        //    .build()
        //    .unwrap();

        //gtk::main();
    });

    #[cfg(not(target_os = "linux"))]
    let mut _tray_icon = Rc::new(RefCell::new(None));
    #[cfg(not(target_os = "linux"))]
    let tray_c = _tray_icon.clone();

    eframe::run_native(
        "My egui App",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| {
            #[cfg(not(target_os = "linux"))]
            {
                tray_c
                    .borrow_mut()
                    .replace(TrayIconBuilder::new().with_icon(icon).build().unwrap());
            }
            Box::<MyApp>::default()
        }),
    )
}

struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use tray_icon::TrayIconEvent;

        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            println!("tray event: {event:?}");
        }
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            println!("menu event: {:?}", event);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
