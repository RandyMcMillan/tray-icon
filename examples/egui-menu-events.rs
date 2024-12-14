#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_os = "linux"))]
use std::{cell::RefCell, rc::Rc};

use muda::{
    accelerator::{Accelerator, Code, Modifiers},
    dpi::Position,
    AboutMetadata, CheckMenuItem, ContextMenu, IconMenuItem, Menu, MenuEvent, MenuItem,
    PredefinedMenuItem, Submenu,
};
#[cfg(target_os = "macos")]
use tao::platform::macos::WindowExtMacOS;
#[cfg(target_os = "linux")]
use tao::platform::unix::WindowExtUnix;
#[cfg(target_os = "windows")]
use tao::platform::windows::{EventLoopBuilderExtWindows, WindowExtWindows};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Window, WindowBuilder},
};
#[cfg(target_os = "linux")]
use wry::WebViewBuilderExtUnix;
use wry::{http::Request, WebViewBuilder};

use eframe::egui;
use tray_icon::{TrayIconBuilder, TrayIconEvent};

use std::error::Error;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

const CUSTOM_PORT: usize = 8000;

enum UserEvent {
    MenuEvent(muda::MenuEvent),
}

fn prepend<T>(v: Vec<T>, s: &[T]) -> Vec<T>
where
    T: Clone,
{
    let mut tmp: Vec<_> = s.to_owned();
    tmp.extend(v);
    tmp
}

fn main() -> wry::Result<()> {
    let wry_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .thread_name("wry-pool")
        .enable_all()
        .build()?;

    // Create another tokio runtime whose job is only to write the response bytes to the outgoing TCP message.
    let tray_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("tray-pool")
        .enable_all()
        .build()?;

    // this channel is used to pass the TcpStream from acceptor_runtime task to
    // to echo_runtime task where the request handling is done.
    let (tx, mut rx) = mpsc::channel::<TcpStream>(CUSTOM_PORT.into());

    // The receiver part of the channel is moved inside a echo_runtime task.
    // This task simply writes the echo response to the TcpStreams coming through the
    // channel receiver.
    wry_runtime.spawn(async move {
        //println!("echo_runtime.spawn");
        while let Some(mut sock) = rx.recv().await {
            //println!("rx.recv().await");
            //prepended bytes are lost
            //103, 110, 111, 115, 116, 114
            let mut buf = prepend(vec![0u8; 512], &[b'g', b'n', b'o', b's', b't', b'r']);
            //println!("pre:buf.push:\n{:?}", &buf);
            //gnostr bytes
            //114, 116, 115, 111, 110, 103
            buf.push(b'r'); //last element 103
            buf.push(b't'); //last element 110
            buf.push(b's'); //last element 111
            buf.push(b'o'); //last element 115
            buf.push(b'n'); //last element 116
            buf.push(b'g'); //last element 114
                            //println!("post:buf.push:\n{:?}", &buf);

            tokio::spawn(async move {
                /*loop {*/
                //println!("pre:\n{:?}", &buf);
                loop {
                    let bytes_read = sock.read(&mut buf).await.expect("failed to read request");

                    if bytes_read == 0 {
                        //println!("99:>>>>>>>>-----> bytes_read = {}", bytes_read);
                        return;
                    }
                    //println!("102:>>>>>>>>-----> bytes_read = {}", bytes_read);
                    let mut new_buf = prepend(vec![0u8; 512], &buf);

                    new_buf.push(b'g'); //last element 32
                    new_buf.push(b'n'); //last element 32
                    new_buf.push(b'o'); //last element 32
                    new_buf.push(b's'); //last element 32
                    new_buf.push(b't'); //last element 32
                    new_buf.push(b'r'); //last element 32
                    sock.write_all(&new_buf[0..bytes_read + 3])
                        .await
                        .expect("failed to write response");
                    //println!("post:\n{:?}", new_buf);
                    let utf8_string = String::from_utf8(new_buf)
                        .map_err(|non_utf8| {
                            String::from_utf8_lossy(non_utf8.as_bytes()).into_owned()
                        })
                        .unwrap();

                    let mut words = utf8_string.split(' ');
                    let first = words.next().unwrap();
                    let second = words.next().unwrap();

                    //println!("first={}", first);
                    println!("second={}", second);

                    //buf.push(b'\n');
                }
                /*}*/
            });
        }
    });

    // acceptor_runtime task is run in a blocking manner, so that our server
    // starts accepting new TCP connections. This task just accepts the
    // incoming TcpStreams and are sent to the sender half of the channel.
    tray_runtime.block_on(async move {
        //println!("acceptor_runtime is started");
        let listener = match TcpListener::bind(format!("127.0.0.1:{}", CUSTOM_PORT)).await {
            Ok(l) => l,
            Err(e) => panic!("error binding TCP listener: {}", e),
        };

        loop {
            //println!(
            //    "{}",
            //    format!("acceptor_runtime loop:listener:{}", CUSTOM_PORT)
            //);
            let sock = match accept_conn(&listener).await {
                Ok(stream) => stream,
                Err(e) => panic!("error reading TCP stream: {}", e),
            };
            let _ = tx.send(sock).await;
        }
    });

    Ok(())

    //wry()
    //tray()?;
}

async fn accept_conn(listener: &TcpListener) -> Result<TcpStream, Box<dyn Error>> {
    //loop {
    /*return*/
    //println!("accept_conn");
    match listener.accept().await {
        Ok((sock, _)) => Ok(sock),
        Err(e) => panic!("error accepting connection: {}", e),
    } /*;*/
    //}
}

async fn wry() -> wry::Result<()> {
    let mut event_loop_builder = EventLoopBuilder::<UserEvent>::with_user_event();

    let menu_bar = Menu::new();

    // setup accelerator handler on Windows
    #[cfg(target_os = "windows")]
    {
        let menu_bar = menu_bar.clone();
        event_loop_builder.with_msg_hook(move |msg| {
            use windows_sys::Win32::UI::WindowsAndMessaging::{TranslateAcceleratorW, MSG};
            unsafe {
                let msg = msg as *const MSG;
                let translated = TranslateAcceleratorW((*msg).hwnd, menu_bar.haccel() as _, msg);
                translated == 1
            }
        });
    }

    let event_loop = event_loop_builder.build();

    // set a menu event handler that wakes up the event loop
    let proxy = event_loop.create_proxy();
    muda::MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let window = WindowBuilder::new()
        .with_title("Window 1")
        .build(&event_loop)
        .unwrap();

    let window2 = WindowBuilder::new()
        .with_title("Window 2")
        .build(&event_loop)
        .unwrap();

    #[cfg(target_os = "macos")]
    {
        let app_m = Submenu::new("App", true);
        menu_bar.append(&app_m).unwrap();
        app_m
            .append_items(&[
                &PredefinedMenuItem::about(None, None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::services(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::hide(None),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(None),
            ])
            .unwrap();
    }

    let file_m = Submenu::new("&File", true);
    let edit_m = Submenu::new("&Edit", true);
    let window_m = Submenu::new("&Window", true);

    menu_bar
        .append_items(&[&file_m, &edit_m, &window_m])
        .unwrap();

    let custom_i_1 = MenuItem::new(
        "C&ustom 1",
        true,
        Some(Accelerator::new(Some(Modifiers::ALT), Code::KeyC)),
    );

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/icon.png");
    let icon = muda_load_icon(std::path::Path::new(path));
    let image_item = IconMenuItem::new(
        "Image custom 1",
        true,
        Some(icon),
        Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyC)),
    );

    let check_custom_i_1 = CheckMenuItem::new("Check Custom 1", true, true, None);
    let check_custom_i_2 = CheckMenuItem::new("Check Custom 2", false, true, None);
    let check_custom_i_3 = CheckMenuItem::new(
        "Check Custom 3",
        true,
        true,
        Some(Accelerator::new(Some(Modifiers::SHIFT), Code::KeyD)),
    );

    let copy_i = PredefinedMenuItem::copy(None);
    let cut_i = PredefinedMenuItem::cut(None);
    let paste_i = PredefinedMenuItem::paste(None);

    file_m
        .append_items(&[
            &custom_i_1,
            &image_item,
            &window_m,
            &PredefinedMenuItem::separator(),
            &check_custom_i_1,
            &check_custom_i_2,
        ])
        .unwrap();

    window_m
        .append_items(&[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(None),
            &PredefinedMenuItem::close_window(Some("Close")),
            &PredefinedMenuItem::fullscreen(None),
            &PredefinedMenuItem::bring_all_to_front(None),
            &PredefinedMenuItem::about(
                None,
                Some(AboutMetadata {
                    name: Some("tao".to_string()),
                    version: Some("1.2.3".to_string()),
                    copyright: Some("Copyright tao".to_string()),
                    ..Default::default()
                }),
            ),
            &check_custom_i_3,
            &image_item,
            &custom_i_1,
        ])
        .unwrap();

    edit_m
        .append_items(&[
            &copy_i,
            &PredefinedMenuItem::separator(),
            &cut_i,
            &PredefinedMenuItem::separator(),
            &paste_i,
        ])
        .unwrap();

    #[cfg(target_os = "windows")]
    unsafe {
        menu_bar.init_for_hwnd(window.hwnd() as _).unwrap();
        menu_bar.init_for_hwnd(window2.hwnd() as _).unwrap();
    }
    #[cfg(target_os = "linux")]
    {
        menu_bar
            .init_for_gtk_window(window.gtk_window(), window.default_vbox())
            .unwrap();
        menu_bar
            .init_for_gtk_window(window2.gtk_window(), window2.default_vbox())
            .unwrap();
    }
    #[cfg(target_os = "macos")]
    {
        menu_bar.init_for_nsapp();
        window_m.set_as_windows_menu_for_nsapp();
    }

    #[cfg(windows)]
    let condition = "e.button !== 2";
    #[cfg(not(windows))]
    let condition = "e.button == 2 && e.buttons === 0";
    let html: String = format!(
        r#"
    <html>
    <body>
        <style>
            * {{
                padding: 0;
                margin: 0;
                box-sizing: border-box;
                font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
            }}
            main {{
                    width: 100vw;
                    height: 100vh;
            }}
            @media (prefers-color-scheme: dark) {{
                main {{
                    color: #fff;
                    background: #2f2f2f;
                }}
            }}
        </style>
        <main>
            <h4> WRYYYYYYYYYYYYYYYYYYYYYY! </h4>
        </main>
        <script>
            window.addEventListener('contextmenu', (e) => {{
                e.preventDefault();
                console.log(e)
                // contextmenu was requested from keyboard
                if ({condition}) {{
                    window.ipc.postMessage(`showContextMenuPos:${{e.clientX}},${{e.clientY}}`);
                }}
            }})
            let x = true;
            window.addEventListener('mouseup', (e) => {{
                if (e.button === 2) {{
                    if (x) {{
                        window.ipc.postMessage(`showContextMenuPos:${{e.clientX}},${{e.clientY}}`);
                    }} else {{
                        window.ipc.postMessage(`showContextMenu`);
                    }}
                    x = !x;
                }}
            }})
        </script>
    </body>
    </html>
  "#,
    );

    let window = Rc::new(window);
    let window2 = Rc::new(window2);

    let create_ipc_handler = |window: &Rc<Window>| {
        let window = window.clone();
        let file_m_c = file_m.clone();
        let menu_bar = menu_bar.clone();
        move |req: Request<String>| {
            let req = req.body();
            if req == "showContextMenu" {
                show_context_menu(&window, &file_m_c, None)
            } else if let Some(rest) = req.strip_prefix("showContextMenuPos:") {
                let (x, mut y) = rest
                    .split_once(',')
                    .map(|(x, y)| (x.parse::<i32>().unwrap(), y.parse::<i32>().unwrap()))
                    .unwrap();

                #[cfg(target_os = "linux")]
                if let Some(menu_bar) = menu_bar
                    .clone()
                    .gtk_menubar_for_gtk_window(window.gtk_window())
                {
                    use gtk::prelude::*;
                    y += menu_bar.allocated_height();
                }

                show_context_menu(&window, &file_m_c, Some(Position::Logical((x, y).into())))
            }
        }
    };

    fn create_webview(window: &Rc<Window>) -> WebViewBuilder<'_> {
        #[cfg(not(target_os = "linux"))]
        return WebViewBuilder::new(window);
        #[cfg(target_os = "linux")]
        WebViewBuilder::new_gtk(window.default_vbox().unwrap())
    };

    let webview = create_webview(&window)
        .with_html(&html)
        .with_ipc_handler(create_ipc_handler(&window))
        .build()?;
    let webview2 = create_webview(&window2)
        .with_html(html)
        .with_ipc_handler(create_ipc_handler(&window2))
        .build()?;

    let menu_channel = MenuEvent::receiver();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            Event::UserEvent(UserEvent::MenuEvent(event)) => {
                if event.id == custom_i_1.id() {
                    let _ = file_m.insert(&MenuItem::new("New Menu Item", true, None), 2);
                }
                println!("{event:?}");
            }
            _ => {}
        }
    })
}

async fn tray() -> Result<(), eframe::Error> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/icon.png");
    let icon = tray_icon_load_icon(std::path::Path::new(path));

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

fn show_context_menu(window: &Window, menu: &dyn ContextMenu, position: Option<Position>) {
    println!("Show context menu at position {position:?}");
    #[cfg(target_os = "windows")]
    unsafe {
        menu.show_context_menu_for_hwnd(window.hwnd() as _, position);
    }
    #[cfg(target_os = "linux")]
    menu.show_context_menu_for_gtk_window(window.gtk_window().as_ref(), position);
    #[cfg(target_os = "macos")]
    unsafe {
        menu.show_context_menu_for_nsview(window.ns_view() as _, position);
    }
}

fn muda_load_icon(path: &std::path::Path) -> muda::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    muda::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

fn tray_icon_load_icon(path: &std::path::Path) -> tray_icon::Icon {
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
