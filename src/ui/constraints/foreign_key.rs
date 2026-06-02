use crate::model::constraints::constraint::ForeignKey;
use crate::ui::widgets::inputs::labeled_text_edit;
use egui::{RichText, Ui};
use std::hash::Hash;
use crate::app::AppColors;

impl ForeignKey {
    /// This will draw a single whole ForeignKey constraint
    pub fn draw(&mut self, ui: &mut Ui, id_source: impl Hash, app_colors: &AppColors) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔗").color(app_colors.fk_color));
            let _ = labeled_text_edit(ui, "", &mut self.name, ("foreign_key_name", id_source));
        });
    }
}
