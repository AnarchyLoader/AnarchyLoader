use egui::{Context, Id, Ui};

pub struct Modal<'a> {
    ctx: &'a Context,
    id: Id,
    title: String,
    close_on_outside_click: bool,
}

impl<'a> Modal<'a> {
    pub fn new<T: std::hash::Hash + ToString>(ctx: &'a Context, id_source: T) -> Self {
        let title = id_source.to_string();
        Modal {
            ctx,
            id: Id::new(&title),
            title,
            close_on_outside_click: false,
        }
    }

    pub fn with_close_on_outside_click(mut self, v: bool) -> Self {
        self.close_on_outside_click = v;
        self
    }

    pub fn open(&self) {
        self.ctx.data_mut(|d| d.insert_temp(self.id, true));
        self.ctx.request_repaint();
    }

    pub fn close(&self) {
        self.ctx.data_mut(|d| d.insert_temp(self.id, false));
        self.ctx.request_repaint();
    }

    pub fn is_open(&self) -> bool {
        self.ctx
            .data(|d| d.get_temp::<bool>(self.id))
            .unwrap_or(false)
    }

    pub fn show(&self, add_contents: impl FnOnce(&mut Ui)) {
        if !self.is_open() {
            return;
        }

        let rect = self.ctx.content_rect();
        let painter = self
            .ctx
            .layer_painter(egui::LayerId::new(egui::Order::Background, self.id));

        painter.rect_filled(rect, 0.0, egui::Color32::from_black_alpha(160));

        let inner = egui::Window::new(self.title.clone())
            .collapsible(false)
            .resizable(false)
            .title_bar(true)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .frame(egui::Frame::popup(&*self.ctx.style()))
            .show(self.ctx, |ui| {
                add_contents(ui);
            });

        if self.close_on_outside_click {
            let clicked_outside = self.ctx.input(|i| {
                if let Some(pos) = i.pointer.interact_pos() {
                    if let Some(inner) = &inner {
                        return i.pointer.any_released() && !inner.response.rect.contains(pos);
                    }
                }
                false
            });

            if clicked_outside {
                self.close();
            }
        }
    }
}
