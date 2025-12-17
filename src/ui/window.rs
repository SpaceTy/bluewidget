use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, Button, CssProvider, GestureDrag, Image, Label, ListBox,
    Orientation, ScrolledWindow, Separator, Switch, Align, STYLE_PROVIDER_PRIORITY_APPLICATION,
    style_context_add_provider_for_display,
};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

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
        let config = Config::load();
        let bluetooth_service = Arc::new(Mutex::new(BluetoothService::new().unwrap_or_else(|e| {
            eprintln!("Failed to initialize Bluetooth service: {}", e);
            // In a real app we might want to show an error dialog or exit
            panic!("Bluetooth service init failed");
        })));

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Bluetooth Widget")
            .default_width(config.window_width)
            .default_height(config.window_height)
            .resizable(true)
            .build();

        let provider = CssProvider::new();
        provider.load_from_data(
            "window { background-color: rgba(0, 0, 0, 0.85); color: white; }
             list { background-color: transparent; }
             row { background-color: transparent; }
             row:hover { background-color: rgba(255, 255, 255, 0.1); }
             .dim-label { opacity: 0.7; }"
        );
        
        style_context_add_provider_for_display(
            &gtk4::prelude::WidgetExt::display(&window),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
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
        let toggle_switch = Switch::builder()
            .valign(Align::Center)
            .build();
        
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

        win.setup_signals(refresh_button, settings_button, close_button);
        win.setup_gestures();
        win.refresh_devices();

        win
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
            let _ = std::process::Command::new("blueman-manager").spawn()
                .or_else(|_| std::process::Command::new("gnome-control-center").arg("bluetooth").spawn());
        });
        
        // Close on focus lost
        self.window.connect_is_active_notify(|win| {
            if !win.is_active() {
                println!("Window lost focus - closing");
                // Delay close slightly to allow clicks to register
                let win_weak = win.downgrade();
                glib::timeout_add_local(Duration::from_millis(50), move || {
                    if let Some(win) = win_weak.upgrade() {
                        win.close();
                    }
                    glib::ControlFlow::Break
                });
            }
        });
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

    pub fn refresh_devices(&self) {
        // Clear list
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        let service = self.bluetooth_service.clone();
        let list_box = self.list_box.clone();
        let service_clone = self.bluetooth_service.clone();
        let bt_enabled = self.config.enable_bluetooth_functionality;

        // Use channel to send devices from thread to main thread
        let (tx, rx) = mpsc::channel();

        // Spawn thread to fetch devices (without moving GTK widgets)
        let service_for_thread = service.clone();
        thread::spawn(move || {
            let devices = if let Ok(s) = service_for_thread.lock() {
                s.get_devices()
            } else {
                vec![]
            };

            let _ = tx.send(devices);
        });

        // Receive devices on main thread and update UI
        let list_box_clone = list_box.clone();
        glib::idle_add_local(move || {
            if let Ok(devices) = rx.try_recv() {
                for device in devices.iter() {
                    let row_widget = DeviceRow::new(device);

                    // Connect signals for row
                    if let Some(switch) = &row_widget.connect_switch {
                        let s = service_clone.clone();
                        let addr = device.address;
                        switch.connect_state_set(move |_, state| {
                            if bt_enabled {
                                if let Ok(service) = s.lock() {
                                    if state {
                                        let _ = service.connect_device(addr);
                                    } else {
                                        let _ = service.disconnect_device(addr);
                                    }
                                }
                            } else {
                                println!("UI Test Mode: Would {} device {}", if state { "connect" } else { "disconnect" }, addr);
                            }
                            glib::Propagation::Proceed
                        });
                    }

                    if let Some(button) = &row_widget.pair_button {
                        let s = service_clone.clone();
                        let addr = device.address;
                        button.connect_clicked(move |_| {
                            if bt_enabled {
                                if let Ok(service) = s.lock() {
                                    let _ = service.pair_device(addr);
                                }
                            } else {
                                println!("UI Test Mode: Would pair device {}", addr);
                            }
                        });
                    }

                    list_box_clone.append(&row_widget.row);
                }
                glib::ControlFlow::Break
            } else {
                // Keep checking until we receive the data
                glib::ControlFlow::Continue
            }
        });
    }
}