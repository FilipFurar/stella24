// ui/entities/domain.rs

use crate::app::Command;
use crate::app::DomainId;
use crate::model::constraints::check::Check;
use crate::model::datatype::{DATA_TYPES, DataType};
use crate::model::entities::domain::Domain;
use crate::ui::changes::IntoCommands;
use crate::ui::constraints::check::draw_check;
use crate::ui::widgets::inputs::labeled_text_edit;
use egui::Ui;
use slotmap::Key;

/// Staged edits coming from domain UI.
#[derive(Default)]
pub struct DomainChanges {
    pub commands: Vec<Command>,
}

impl IntoCommands for DomainChanges {
    fn into_commands(self) -> Vec<Command> {
        self.commands
    }
}

/// UI implementation for domains
impl Domain {
    /// Draw domain's contents in Workbench and return change flags.
    pub fn draw(&mut self, ui: &mut Ui, id: DomainId) -> DomainChanges {
        let mut changes = DomainChanges::default();

        if labeled_text_edit(
            ui,
            "Name:",
            &mut self.name,
            format!("domain_name_{}", id.data().as_ffi()),
        ) {
            changes.commands.push(Command::RenameDomain {
                domain: id,
                name: self.name.clone(),
            });
        }

        ui.horizontal(|ui| {
            ui.label("Type:");
            let mut data_type_changed = false;
            egui::ComboBox::from_id_salt(format!("type_{}", id.data().as_ffi()))
                .selected_text(DATA_TYPES[self.data_type.base].name)
                .show_ui(ui, |ui| {
                    for (i, def) in DATA_TYPES.iter().enumerate() {
                        if ui
                            .selectable_label(self.data_type.base == i, def.name)
                            .clicked()
                        {
                            self.data_type = DataType {
                                base: i,
                                params: vec![0; def.param_count],
                            };
                            data_type_changed = true;
                        }
                    }
                });

            if !self.data_type.params.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("(");
                    for param in &mut self.data_type.params {
                        if ui
                            .add(egui::DragValue::new(param).speed(1).range(0..=1_000_000))
                            .changed()
                        {
                            data_type_changed = true;
                        }
                    }
                    ui.label(")");
                });
            }

            if data_type_changed {
                changes.commands.push(Command::SetDomainType {
                    domain: id,
                    data_type: self.data_type.clone(),
                });
            }
        });

        ui.separator();
        ui.label("Check constraints:");

        let mut to_delete: Vec<usize> = Vec::new();
        for (i, check) in self.check_constraints.iter_mut().enumerate() {
            if draw_check(ui, check, ("domain_check", id.data().as_ffi(), i), "sql") {
                to_delete.push(i);
            }
        }

        for i in to_delete.iter().copied().rev() {
            self.check_constraints.remove(i);
        }

        for i in to_delete.into_iter().rev() {
            changes.commands.push(Command::DeleteDomainCheck {
                domain: id,
                index: i,
            });
        }

        if ui.button("Add Check").clicked() {
            changes.commands.push(Command::AddDomainCheck {
                domain: id,
                check: Check::new(),
            });
        }

        changes
    }
}
