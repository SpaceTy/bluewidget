use gtk4::prelude::*;
use gtk4::{
    style_context_add_provider_for_display, Align, Application, ApplicationWindow, Box, Button,
    CssProvider, EventControllerFocus, GestureClick, GestureDrag, Image, Label, ListBox,
    Orientation, ScrolledWindow, Separator, Switch, STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::bluetooth::BluetoothService;
use crate::config::Config;
use crate::ui::device_row::DeviceRow;

pub struct Window {
    pub window: ApplicationWindow,
    pub list_box: ListBox,
    pub status_label: Label,
    pub toggle_switch: Switch,
    bluetooth_service: Arc<Mutex<BluetoothService>>,
    config: Config,
}

impl Window {
    pub fn new(app: &Application) -> Self {
        let overall_start = Instant::now();

        let config_start = Instant::now();
        let config = Config::load();
        println!(
            "[SPEED] Config::load() took: {:.2?}",
            config_start.elapsed()
        );

        let bt_start = Instant::now();
        let bluetooth_service = Arc::new(Mutex::new(BluetoothService::new().unwrap_or_else(|e| {
            eprintln!("Failed to initialize Bluetooth service: {}", e);
            // In a real app we might want to show an error dialog or exit
            panic!("Bluetooth service init failed");
        })));
        println!(
            "[SPEED] BluetoothService::new() took: {:.2?}",
            bt_start.elapsed()
        );

        let window_start = Instant::now();
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Bluetooth Widget")
            .default_width(config.window_width)
            .default_height(config.window_height)
            .resizable(true)
            .build();
        println!(
            "[SPEED] ApplicationWindow::builder() took: {:.2?}",
            window_start.elapsed()
        );

        let css_start = Instant::now();
        let provider = CssProvider::new();
        provider.load_from_data(
            "window { background-color: rgba(0, 0, 0, 0.85); color: white; }
             list { background-color: transparent; }
             row { background-color: transparent; }
             row:hover { background-color: rgba(255, 255, 255, 0.1); }
             .dim-label { opacity: 0.7; }",
        );

        style_context_add_provider_for_display(
            &gtk4::prelude::WidgetExt::display(&window),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        println!(
            "[SPEED] CSS provider setup took: {:.2?}",
            css_start.elapsed()
        );

        let main_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        // Header
        let header_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        let icon = Image::builder()
            .icon_name("bluetooth")
            .pixel_size(18)
            .valign(Align::Center)
            .build();
        header_box.append(&icon);

        let status_label = Label::builder()
            .use_markup(true)
            .label("<b>Bluetooth</b>")
            .valign(Align::Center)
            .build();
        header_box.append(&status_label);

        // Spacer
        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header_box.append(&spacer);

        // Refresh button
        let refresh_button = Button::builder()
            .icon_name("view-refresh")
            .tooltip_text("Refresh devices")
            .css_classes(vec!["flat"])
            .valign(Align::Center)
            .build();
        header_box.append(&refresh_button);

        // Settings button
        let settings_button = Button::builder()
            .icon_name("preferences-system")
            .tooltip_text("Bluetooth settings")
            .css_classes(vec!["flat"])
            .valign(Align::Center)
            .build();
        header_box.append(&settings_button);

        // Toggle switch
        let toggle_switch = Switch::builder().valign(Align::Center).build();

        // Set initial state
        if let Ok(service) = bluetooth_service.lock() {
            toggle_switch.set_active(service.is_powered());
        }

        header_box.append(&toggle_switch);

        // Close button
        let close_button = Button::builder()
            .icon_name("window-close")
            .tooltip_text("Close widget")
            .css_classes(vec!["flat"])
            .valign(Align::Center)
            .build();
        header_box.append(&close_button);

        main_box.append(&header_box);

        // Separator
        let separator = Separator::new(Orientation::Horizontal);
        main_box.append(&separator);

        // Device list
        let scrolled = ScrolledWindow::builder()
            .min_content_height(0)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Never)
            .build();

        let list_box = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .build();

        scrolled.set_child(Some(&list_box));
        main_box.append(&scrolled);

        window.set_child(Some(&main_box));

        let win = Self {
            window,
            list_box,
            status_label,
            toggle_switch,
            bluetooth_service,
            config,
        };

        let signals_start = Instant::now();
        win.setup_signals(refresh_button, settings_button, close_button);
        println!(
            "[SPEED] setup_signals() took: {:.2?}",
            signals_start.elapsed()
        );

        let gestures_start = Instant::now();
        win.setup_gestures();
        println!(
            "[SPEED] setup_gestures() took: {:.2?}",
            gestures_start.elapsed()
        );

        // Defer device loading until after window is presented
        let list_box_weak = win.list_box.downgrade();
        let service = win.bluetooth_service.clone();
        let bt_enabled = win.config.enable_bluetooth_functionality;
        glib::idle_add_local(move || {
            if let Some(list_box) = list_box_weak.upgrade() {
                Self::refresh_devices_deferred(&list_box, service.clone(), bt_enabled);
            }
            glib::ControlFlow::Break
        });

        println!(
            "[SPEED] Window::new() total took: {:.2?}",
            overall_start.elapsed()
        );

        win
    }

    pub fn present(&self) {
        self.window.present();
    }

    fn refresh_devices_deferred(
        list_box: &ListBox,
        bluetooth_service: Arc<Mutex<BluetoothService>>,
        bt_enabled: bool,
    ) {
        let refresh_start = Instant::now();

        // Clear list
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        let list_box = list_box.clone();
        let service_clone = bluetooth_service.clone();

        // Use channel to send devices from thread to main thread
        let (tx, rx) = mpsc::channel();

        // Spawn thread to fetch devices
        let service_for_thread = bluetooth_service.clone();
        let thread_start = Instant::now();
        thread::spawn(move || {
            let devices = if let Ok(s) = service_for_thread.lock() {
                s.get_devices()
            } else {
                vec![]
            };
            println!(
                "[SPEED] Device fetching in thread took: {:.2?}",
                thread_start.elapsed()
            );
            let _ = tx.send(devices);
        });

        // Receive devices on main thread and update UI
        let list_box_clone = list_box.clone();
        let ui_update_start = Instant::now();
        glib::idle_add_local(move || {
            if let Ok(devices) = rx.try_recv() {
                let row_count = devices.len();
                for device in devices.iter() {
                    let row_widget = DeviceRow::new(device);

                    if let Some(switch) = &row_widget.connect_switch {
                        let s = service_clone.clone();
                        let addr = device.address;
                        let bt = bt_enabled;
                        switch.connect_state_set(move |_, state| {
                            if bt {
                                if let Ok(service) = s.lock() {
                                    if state {
                                        let _ = service.connect_device(addr);
                                    } else {
                                        let _ = service.disconnect_device(addr);
                                    }
                                }
                            }
                            glib::Propagation::Proceed
                        });
                    }

                    if let Some(button) = &row_widget.pair_button {
                        let s = service_clone.clone();
                        let addr = device.address;
                        let bt = bt_enabled;
                        button.connect_clicked(move |_| {
                            if bt {
                                if let Ok(service) = s.lock() {
                                    let _ = service.pair_device(addr);
                                }
                            }
                        });
                    }

                    list_box_clone.append(&row_widget.row);
                }
                println!(
                    "[SPEED] UI update with {} devices took: {:.2?}",
                    row_count,
                    ui_update_start.elapsed()
                );
                println!(
                    "[SPEED] Total refresh_devices() including async work: {:.2?}",
                    refresh_start.elapsed()
                );
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    fn setup_signals(&self, refresh_btn: Button, settings_btn: Button, close_btn: Button) {
        let service = self.bluetooth_service.clone();
        let status_label = self.status_label.clone();
        let bt_enabled = self.config.enable_bluetooth_functionality;

        // Toggle Bluetooth
        self.toggle_switch.connect_state_set(move |_, state| {
            if bt_enabled {
                if let Ok(service) = service.lock() {
                    if state {
                        let _ = service.power_on();
                        status_label.set_markup("<b>Bluetooth</b> <span foreground='green'>On</span>");
                    } else {
                        let _ = service.power_off();
                        status_label.set_markup("<b>Bluetooth</b> <span foreground='red'>Off</span>");
                    }
                }
            } else {
                // UI testing mode - just update the label without actually changing bluetooth
                if state {
                    status_label.set_markup("<b>Bluetooth</b> <span foreground='green'>On</span> <span foreground='orange'>(UI Test)</span>");
                } else {
                    status_label.set_markup("<b>Bluetooth</b> <span foreground='red'>Off</span> <span foreground='orange'>(UI Test)</span>");
                }
            }
            // Trigger refresh
            // In a real app we'd want to trigger this on the main struct
            glib::Propagation::Proceed
        });

        // Refresh button
        refresh_btn.connect_clicked(|_| {
            // We need a way to call refresh_devices here.
            // For simplicity in this structure, we might need to rethink ownership or use channels.
            // For now, let's just print.
            println!("Refresh clicked");
        });

        // Close button
        let window_weak_close = self.window.downgrade();
        close_btn.connect_clicked(move |_| {
            if let Some(window) = window_weak_close.upgrade() {
                window.close();
            }
        });

        // Settings button
        settings_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("blueman-manager")
                .spawn()
                .or_else(|_| {
                    std::process::Command::new("gnome-control-center")
                        .arg("bluetooth")
                        .spawn()
                });
        });

        // Track if a click/touch is currently in progress
        let click_in_progress = Arc::new(Mutex::new(false));

        // Set up click gesture to track touch/click interactions
        let click_gesture = GestureClick::new();
        let click_flag = click_in_progress.clone();
        click_gesture.connect_pressed(move |_, _, _, _| {
            *click_flag.lock().unwrap() = true;
            println!("Click/touch started");
        });

        let click_flag = click_in_progress.clone();
        click_gesture.connect_released(move |_, _, _, _| {
            let click_flag = click_flag.clone();
            // Keep flag true for a bit after release to prevent premature closing
            glib::timeout_add_local(Duration::from_millis(100), move || {
                *click_flag.lock().unwrap() = false;
                println!("Click/touch ended");
                glib::ControlFlow::Break
            });
        });

        self.window.add_controller(click_gesture);

        // Close on focus lost using EventControllerFocus
        // This properly handles touch interactions unlike is_active()
        let focus_controller = EventControllerFocus::new();
        let window_weak = self.window.downgrade();
        let click_flag = click_in_progress.clone();

        focus_controller.connect_leave(move |_| {
            // This fires when focus leaves the window hierarchy entirely
            println!("Focus left window");

            // Don't close if a click/touch is in progress
            if *click_flag.lock().unwrap() {
                println!("Click in progress - not closing");
                return;
            }

            if let Some(win) = window_weak.upgrade() {
                // Small delay to allow any in-flight events to complete
                let win_weak = win.downgrade();
                glib::timeout_add_local(Duration::from_millis(50), move || {
                    if let Some(w) = win_weak.upgrade() {
                        println!("Closing window");
                        w.close();
                    }
                    glib::ControlFlow::Break
                });
            }
        });

        self.window.add_controller(focus_controller);
    }

    fn setup_gestures(&self) {
        let gesture = GestureDrag::new();
        let start_y = Arc::new(Mutex::new(0.0));
        let start_time = Arc::new(Mutex::new(0));

        let start_y_clone = start_y.clone();
        let start_time_clone = start_time.clone();

        gesture.connect_drag_begin(move |_, _, y| {
            if let Ok(mut sy) = start_y_clone.lock() {
                *sy = y;
            }
            if let Ok(mut st) = start_time_clone.lock() {
                *st = glib::monotonic_time();
            }
        });

        let window_weak = self.window.downgrade();
        gesture.connect_drag_end(move |_, _, y| {
            let sy = *start_y.lock().unwrap();
            let st = *start_time.lock().unwrap();

            let swipe_distance = sy - y; // Positive if swiping up
            let swipe_time = (glib::monotonic_time() - st) as f64 / 1_000_000.0;

            if swipe_distance > 100.0 && swipe_time < 1.0 {
                println!("Swipe up detected - closing");
                if let Some(win) = window_weak.upgrade() {
                    win.close();
                }
            }
        });

        self.window.add_controller(gesture);
    }
}
