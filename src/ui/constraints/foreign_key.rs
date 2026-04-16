use crate::model::constraints::constraint::ForeignKey;
use eframe::epaint::Color32;
use egui::{RichText, Ui};

const BLUE: Color32 = Color32::from_rgb(75, 67, 185);

impl ForeignKey {
    /// This will draw a single whole ForeignKey constraint
    pub fn draw(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔗").color(BLUE));
            ui.text_edit_singleline(&mut self.name);
        });
    }
}
