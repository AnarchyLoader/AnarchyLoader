use egui::{emath::easing, text::LayoutJob, Id, TextFormat};
use egui_alignments::center_vertical;
use whoami;

use crate::MyApp;

#[derive(Debug, PartialEq)]
pub enum AnimationPhase {
    Initial,
    FadeIn,
    Display,
    FadeOut,
    Complete,
}

#[derive(Debug)]
pub struct AnimationState {
    pub image_opacity: f32,
    pub text_decode_time: f32,
    pub text_opacity: f32,
    pub delay_timer: f32,
    pub display_timer: f32,
    pub phase: AnimationPhase,
    pub subtitle_opacity: f32,
    pub image_scale_animation_started: bool,
    pub start_delay_timer: f32,
    pub hello_opacity: f32,
    pub hello_scale: f32,
    pub hello_animation_progress: f32,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            image_opacity: 0.0,
            text_decode_time: 0.0,
            text_opacity: 1.0,
            delay_timer: 0.0,
            display_timer: 0.0,
            phase: AnimationPhase::Initial,
            subtitle_opacity: 0.0,
            image_scale_animation_started: false,
            start_delay_timer: 0.0,
            hello_opacity: 0.0,
            hello_scale: 1.0,
            hello_animation_progress: 0.0,
        }
    }
}

impl MyApp {
    pub(crate) fn update_animation(&mut self, dt: f32) {
        const START_DELAY: f32 = 1.0;
        const INITIAL_DELAY: f32 = 0.5;
        const FADE_DURATION: f32 = 0.8;
        const DISPLAY_DURATION: f32 = 2.5;
        const ANIMATION_SPEED: f32 = 1.0 / FADE_DURATION;
        const HELLO_ANIMATION_DURATION: f32 = 0.7;

        match self.ui.animation.phase {
            AnimationPhase::Initial => {
                if self.ui.animation.start_delay_timer < START_DELAY {
                    self.ui.animation.start_delay_timer += dt;
                } else {
                    if self.ui.animation.hello_animation_progress < 1.0 {
                        self.ui.animation.hello_animation_progress =
                            (self.ui.animation.hello_animation_progress
                                + dt / HELLO_ANIMATION_DURATION)
                                .min(1.0);

                        let hello_easing =
                            easing::cubic_out(self.ui.animation.hello_animation_progress);
                        self.ui.animation.hello_opacity = hello_easing;
                        self.ui.animation.hello_scale = 1.0 + (0.5 - 1.0) * hello_easing;
                    } else {
                        self.ui.animation.delay_timer += dt;
                        if self.ui.animation.delay_timer >= INITIAL_DELAY {
                            self.ui.animation.phase = AnimationPhase::FadeIn;
                            self.ui.animation.image_scale_animation_started = true;
                        }
                    }
                }
            }

            AnimationPhase::FadeIn => {
                if self.ui.animation.image_opacity < 1.0 {
                    self.ui.animation.image_opacity =
                        (self.ui.animation.image_opacity + dt * ANIMATION_SPEED).min(1.0);
                } else if self.ui.animation.text_decode_time < 1.0 {
                    self.ui.animation.text_decode_time += dt * ANIMATION_SPEED;
                    self.ui.animation.text_decode_time =
                        self.ui.animation.text_decode_time.min(1.0);
                } else if self.ui.animation.subtitle_opacity < 1.0 {
                    self.ui.animation.subtitle_opacity =
                        (self.ui.animation.subtitle_opacity + dt * ANIMATION_SPEED).min(1.0);
                } else {
                    self.ui.animation.phase = AnimationPhase::Display;
                    self.ui.animation.display_timer = 0.0;
                }
            }

            AnimationPhase::Display => {
                self.ui.animation.display_timer += dt;
                if self.ui.animation.display_timer >= DISPLAY_DURATION {
                    self.ui.animation.phase = AnimationPhase::FadeOut;
                    self.ui.animation.image_scale_animation_started = false;
                }
            }

            AnimationPhase::FadeOut => {
                if self.ui.animation.image_opacity > 0.0
                    || self.ui.animation.subtitle_opacity > 0.0
                    || self.ui.animation.text_opacity > 0.0
                {
                    self.ui.animation.image_opacity =
                        (self.ui.animation.image_opacity - dt * ANIMATION_SPEED).max(0.0);
                    self.ui.animation.subtitle_opacity =
                        (self.ui.animation.subtitle_opacity - dt * ANIMATION_SPEED).max(0.0);
                    self.ui.animation.text_opacity =
                        (self.ui.animation.text_opacity - dt * ANIMATION_SPEED).max(0.0);
                    self.ui.animation.hello_opacity =
                        (self.ui.animation.hello_opacity - dt * ANIMATION_SPEED).max(0.0);
                } else {
                    self.ui.animation.phase = AnimationPhase::Complete;
                }
            }

            AnimationPhase::Complete => {}
        }
    }

    pub(crate) fn render_intro_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            center_vertical(ui, |ui| {
                let is_dark_mode = ui.visuals().dark_mode;
                let text_color = if is_dark_mode {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::BLACK
                };

                let hello_text = format!("Hello, {}!", whoami::username());
                ui.label(
                    egui::RichText::new(hello_text)
                        .size(40.0 * self.ui.animation.hello_scale)
                        .color(text_color.gamma_multiply(self.ui.animation.hello_opacity)),
                );

                let scale_factor = ctx.animate_bool_with_time_and_easing(
                    Id::new("image_scale_animation"),
                    self.ui.animation.image_scale_animation_started,
                    0.8,
                    |x| easing::cubic_out(x),
                );

                let scale = 50.0 + 50.0 * scale_factor;

                let image = egui::Image::new(egui::include_image!("../../resources/img/icon.ico"))
                    .max_width(scale);

                let tint_color = text_color.gamma_multiply(self.ui.animation.image_opacity);

                ui.add(image.tint(tint_color));

                let decoded_text = "anarchyloader";
                let chars: Vec<char> = decoded_text.chars().collect();
                let num_chars = chars.len();
                let visible_chars_float = self.ui.animation.text_decode_time * num_chars as f32;
                let visible_chars = visible_chars_float.floor() as usize;
                let remainder = visible_chars_float - visible_chars_float.floor();

                let mut job = LayoutJob::default();

                ui.horizontal_wrapped(|_ui| {
                    for (i, ch) in chars.iter().enumerate() {
                        let char_alpha_f32: f32 = if i < visible_chars {
                            self.ui.animation.text_opacity
                        } else if i == visible_chars && i < num_chars {
                            remainder * self.ui.animation.text_opacity
                        } else {
                            0.0
                        };
                        job.append(
                            &ch.to_string(),
                            0.0,
                            TextFormat {
                                color: text_color.gamma_multiply(char_alpha_f32),
                                ..Default::default()
                            },
                        );
                    }
                });

                ui.label(job);

                ui.add_space(5.0);
                ui.label(
                    egui::RichText::new("Open Source Loader, by dest4590")
                        .size(12.0)
                        .color(text_color.gamma_multiply(self.ui.animation.subtitle_opacity)),
                );
            });
        });
    }
}
