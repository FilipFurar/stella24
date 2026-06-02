use egui::TextBuffer;

/// Renders highlighted multiline code in a read-only editor-like control.
pub fn draw_highlighted_code(ui: &mut egui::Ui, content: &str, language: &str, rows: usize) {
    let mut view = content.to_owned();
    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
    let mut layouter = |ui: &egui::Ui, buf: &dyn TextBuffer, wrap_width: f32| {
        let mut job = egui_extras::syntax_highlighting::highlight(
            ui.ctx(),
            ui.style(),
            &theme,
            buf.as_str(),
            language,
        );
        job.wrap.max_width = wrap_width;
        ui.fonts_mut(|fonts| fonts.layout_job(job))
    };

    ui.add(
        egui::TextEdit::multiline(&mut view)
            .desired_width(f32::INFINITY)
            .desired_rows(rows)
            .code_editor()
            .font(egui::TextStyle::Monospace)
            .interactive(true)
            .layouter(&mut layouter),
    );
}
