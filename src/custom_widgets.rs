use egui::CursorIcon::PointingHand as Clickable;

pub trait Button {
    fn cbutton(&mut self, label: &str) -> egui::Response;
    fn button_with_tooltip(&mut self, label: &str, tooltip: &str) -> egui::Response;
}

impl Button for egui::Ui {
    fn cbutton(&mut self, label: &str) -> egui::Response {
        self.button(label).on_hover_cursor(Clickable)
    }

    fn button_with_tooltip(&mut self, label: &str, tooltip: &str) -> egui::Response {
        self.button(label)
            .on_hover_cursor(Clickable)
            .on_hover_text(tooltip)
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
