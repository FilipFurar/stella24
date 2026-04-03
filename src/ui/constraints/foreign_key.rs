use crate::model::constraints::constraint::ForeignKey;
use eframe::epaint::{Color32, Stroke};
use egui::{RichText, Ui};

const BLUE: Color32 = Color32::from_rgb(75, 67, 185);

impl ForeignKey {
    /// This will draw a single whole ForeignKey constraint
    pub fn display(&self, ui: &mut Ui) {
        egui::Frame::group(ui.style())
            .stroke(Stroke::new(1.0, BLUE))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🔗").color(BLUE));
                    ui.label(&self.name);
                });
            });
    }
}
