use crate::model::constraints::check::Check;
use eframe::epaint::Color32;
use egui::{TextBuffer, Ui};
use std::hash::Hash;

const CHECK_COLOR: Color32 = Color32::from_rgb(149, 117, 205);

pub fn draw_check(ui: &mut Ui, check: &mut Check, id_source: impl Hash, language: &str) -> bool {
    let mut delete = false;
    let id_salt = &id_source;
    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

    egui::Frame::group(ui.style())
        .stroke(egui::Stroke::new(1.0, CHECK_COLOR))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("CHECK").color(CHECK_COLOR).strong());
                ui.add(egui::TextEdit::singleline(&mut check.name).desired_width(10.0).clip_text(false));
                if ui.button("🗑").clicked() {
                    delete = true;
                }
            });

            egui::CollapsingHeader::new("SQL condition")
                .id_salt(("check_sql", id_salt))
                .default_open(false)
                .show(ui, |ui| {
                    let mut layouter = |ui: &egui::Ui, buf: &dyn TextBuffer, wrap_width: f32| {
                        let mut job = egui_extras::syntax_highlighting::highlight(
                            ui.ctx(),
                            ui.style(),
                            &theme,
                            buf.as_str(),
                            language,
                        );
                        job.wrap.max_width = wrap_width;
                        ui.fonts_mut(|f| f.layout_job(job))
                    };

                    ui.add_sized(
                        [ui.available_width(), 96.0],
                        egui::TextEdit::multiline(&mut check.condition)
                            .id_source((id_salt, "check_sql_editor"))
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .desired_rows(3)
                            .lock_focus(true)
                            .layouter(&mut layouter),
                    );
                });
        });

    delete
}
