use egui::{CursorIcon::PointingHand as Clickable, Response, Ui, WidgetText};
use egui_material_icons::icons::{ICON_CHECK, ICON_RESTART_ALT};
use egui_notify::Toasts;

pub trait Button {
    fn cbutton(&mut self, label: impl Into<WidgetText>) -> Response;
    fn cibutton(&mut self, label: &str, icon: &str) -> Response;
    fn button_with_tooltip(
        &mut self,
        label: impl Into<WidgetText>,
        tooltip: impl Into<WidgetText>,
    ) -> Response;
    fn link_button(
        &mut self,
        label: impl Into<WidgetText>,
        url: &str,
        toasts: &mut Toasts,
    ) -> Response;
    fn confirm_button(&mut self) -> Response;
    fn reset_button(&mut self, label: &str) -> Response;
}

impl Button for Ui {
    fn cbutton(&mut self, label: impl Into<WidgetText>) -> Response {
        self.button(label).on_hover_cursor(Clickable)
    }

    fn cibutton(&mut self, label: &str, icon: &str) -> Response {
        self.button(format!("{} {}", icon, label))
            .on_hover_cursor(Clickable)
    }

    fn button_with_tooltip(
        &mut self,
        label: impl Into<WidgetText>,
        tooltip: impl Into<WidgetText>,
    ) -> Response {
        self.button(label)
            .on_hover_cursor(Clickable)
            .on_hover_text(tooltip)
    }

    fn link_button(
        &mut self,
        label: impl Into<WidgetText>,
        url: &str,
        toasts: &mut Toasts,
    ) -> Response {
        let response = self
            .button(label)
            .on_hover_cursor(Clickable)
            .on_hover_text(url);

        if response.clicked() {
            if let Err(e) = opener::open(url) {
                toasts.error(format!("Failed to open URL: {}", e));
            }
        }

        response
    }

    fn confirm_button(&mut self) -> Response {
        self.cibutton("Confirm", ICON_CHECK)
    }

    fn reset_button(&mut self, label: &str) -> Response {
        self.cibutton(label, ICON_RESTART_ALT)
    }
}

pub trait TextEdit {
    fn ctext_edit(&mut self, text: &mut String, default_value: String) -> Response;
}

impl TextEdit for Ui {
    fn ctext_edit(&mut self, text: &mut String, default_value: String) -> Response {
        let response = self.text_edit_singleline(text);

        response.context_menu(|ui| {
            if ui.cbutton("Reset").clicked() {
                *text = default_value.clone();
                ui.close_menu();
            }
        });

        response
    }
}

pub trait SelectableLabel {
    fn cselectable_label(&mut self, checked: bool, text: &str) -> Response;
}

impl SelectableLabel for Ui {
    fn cselectable_label(&mut self, checked: bool, text: &str) -> Response {
        self.selectable_label(checked, text)
            .on_hover_cursor(Clickable)
    }
}

pub trait CheckBox {
    fn ccheckbox(&mut self, checked: &mut bool, text: impl Into<WidgetText>) -> Response;
}

impl CheckBox for Ui {
    fn ccheckbox(&mut self, checked: &mut bool, text: impl Into<WidgetText>) -> Response {
        self.checkbox(checked, text).on_hover_cursor(Clickable)
    }
}

pub trait Hyperlink {
    fn clink(&mut self, text: impl Into<WidgetText>, url: &str) -> Response;
}

impl Hyperlink for Ui {
    fn clink(&mut self, text: impl Into<WidgetText>, url: &str) -> Response {
        self.hyperlink_to(text, url)
            .on_hover_cursor(Clickable)
            .on_hover_text(url)
    }
}
