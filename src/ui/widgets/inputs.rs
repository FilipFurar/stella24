use egui::Ui;
use std::hash::Hash;

/// Draws a compact labeled single-line text edit and returns whether it changed.
pub fn labeled_text_edit(
    ui: &mut Ui,
    label: &str,
    value: &mut String,
    id: impl Hash,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        if !label.is_empty() {
            ui.label(label);
        }
        if ui
            .add(
                egui::TextEdit::singleline(value)
                    .id_source(id)
                    .desired_width(10.0)
                    .clip_text(false),
            )
            .changed()
        {
            changed = true;
        }
    });

    changed
}
