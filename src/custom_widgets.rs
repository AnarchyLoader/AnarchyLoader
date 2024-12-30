use egui::{CursorIcon::PointingHand as Clickable, WidgetText};
use egui_notify::Toasts;

pub trait Button {
    fn cbutton(&mut self, label: impl Into<egui::WidgetText>) -> egui::Response;
    fn button_with_tooltip(
        &mut self,
        label: impl Into<egui::WidgetText>,
        tooltip: impl Into<egui::WidgetText>,
    ) -> egui::Response;
    fn link_button(
        &mut self,
        label: impl Into<egui::WidgetText>,
        url: &str,
        toasts: &mut Toasts,
    ) -> egui::Response;
}

impl Button for egui::Ui {
    fn cbutton(&mut self, label: impl Into<egui::WidgetText>) -> egui::Response {
        self.button(label).on_hover_cursor(Clickable)
    }

    fn button_with_tooltip(
        &mut self,
        label: impl Into<egui::WidgetText>,
        tooltip: impl Into<egui::WidgetText>,
    ) -> egui::Response {
        self.button(label)
            .on_hover_cursor(Clickable)
            .on_hover_text(tooltip)
    }

    fn link_button(
        &mut self,
        label: impl Into<egui::WidgetText>,
        url: &str,
        toasts: &mut Toasts,
    ) -> egui::Response {
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
}

pub trait TextEdit {
    fn ctext_edit(&mut self, text: &mut String, default_value: String) -> egui::Response;
}

impl TextEdit for egui::Ui {
    fn ctext_edit(&mut self, text: &mut String, default_value: String) -> egui::Response {
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
    fn cselectable_label(&mut self, checked: bool, text: &str) -> egui::Response;
}

impl SelectableLabel for egui::Ui {
    fn cselectable_label(&mut self, checked: bool, text: &str) -> egui::Response {
        self.selectable_label(checked, text)
            .on_hover_cursor(Clickable)
    }
}

pub trait CheckBox {
    fn ccheckbox(&mut self, checked: &mut bool, text: impl Into<WidgetText>) -> egui::Response;
}

impl CheckBox for egui::Ui {
    fn ccheckbox(&mut self, checked: &mut bool, text: impl Into<WidgetText>) -> egui::Response {
        self.checkbox(checked, text).on_hover_cursor(Clickable)
    }
}

pub trait Hyperlink {
    fn clink(&mut self, text: &str, url: &str) -> egui::Response;
}

impl Hyperlink for egui::Ui {
    fn clink(&mut self, text: &str, url: &str) -> egui::Response {
        self.hyperlink_to(text, url)
            .on_hover_cursor(Clickable)
            .on_hover_text(url)
    }
}
