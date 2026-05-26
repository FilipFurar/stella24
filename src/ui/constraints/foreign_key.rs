use crate::model::constraints::constraint::ForeignKey;
use crate::ui::widgets::inputs::labeled_text_edit;
use eframe::epaint::Color32;
use egui::{RichText, Ui};
use std::hash::Hash;

const BLUE: Color32 = Color32::from_rgb(75, 67, 185);

impl ForeignKey {
    /// This will draw a single whole ForeignKey constraint
    pub fn draw(&mut self, ui: &mut Ui, id_source: impl Hash) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔗").color(BLUE));
            let _ = labeled_text_edit(ui, "", &mut self.name, ("foreign_key_name", id_source));
        });
    }
}
