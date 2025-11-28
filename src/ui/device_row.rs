use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Image, Label, ListBoxRow, Orientation, Switch};
use crate::bluetooth::BluetoothDevice;

pub struct DeviceRow {
    pub row: ListBoxRow,
    pub connect_switch: Option<Switch>,
    pub pair_button: Option<Button>,
}

impl DeviceRow {
    pub fn new(device: &BluetoothDevice) -> Self {
        let row = ListBoxRow::new();
        
        let box_container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(3)
            .margin_bottom(3)
            .build();

        // Device icon
        let icon = Image::builder()
            .icon_name(&device.get_icon_name())
            .pixel_size(16)
            .valign(Align::Center)
            .build();
        box_container.append(&icon);

        // Device info
        let info_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(1)
            .build();

        let name_label = Label::builder()
            .label(&device.name)
            .use_markup(true)
            .xalign(0.0)
            .valign(Align::Center)
            .build();
        name_label.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(&device.name)));
        info_box.append(&name_label);

        let addr_label = Label::builder()
            .label(&device.address.to_string())
            .xalign(0.0)
            .valign(Align::Center)
            .css_classes(vec!["dim-label"])
            .build();
        info_box.append(&addr_label);

        box_container.append(&info_box);

        // Spacer
        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        box_container.append(&spacer);

        let mut connect_switch = None;
        let mut pair_button = None;

        if device.paired {
            let switch = Switch::builder()
                .active(device.connected)
                .tooltip_text("Connect/Disconnect")
                .valign(Align::Center)
                .build();
            box_container.append(&switch);
            connect_switch = Some(switch);
        } else {
            let button = Button::builder()
                .label("Pair")
                .css_classes(vec!["flat"])
                .valign(Align::Center)
                .build();
            box_container.append(&button);
            pair_button = Some(button);
        }

        row.set_child(Some(&box_container));

        Self {
            row,
            connect_switch,
            pair_button,
        }
    }
}