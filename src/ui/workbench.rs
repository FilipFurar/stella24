use std::collections::HashMap;

use crate::AppStella;
use crate::app::{Command, TableId};
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;
use crate::ui::changes::extend_commands;
use crate::ui::context::TableUiContext;
use crate::ui::widgets::crow_foot::{build_edges, draw_crow_foot_edge};
use eframe::emath::{Rect, Vec2, pos2, vec2};
use egui::{Area, Frame, Id, Order, PointerButton, Sense};

const CANVAS_PADDING: f32 = 8.0;
const DEFAULT_TABLE_SIZE: Vec2 = Vec2::new(300.0, 200.0);
const DEFAULT_TABLE_GAP: Vec2 = Vec2::new(360.0, 260.0);

fn default_workbench_rect(index: usize) -> Rect {
    let col = (index % 3) as f32;
    let row = (index / 3) as f32;
    Rect::from_min_size(
        pos2(
            40.0 + col * DEFAULT_TABLE_GAP.x,
            40.0 + row * DEFAULT_TABLE_GAP.y,
        ),
        DEFAULT_TABLE_SIZE,
    )
}

fn world_to_screen_rect(rect: Rect, pan: Vec2, zoom: f32) -> Rect {
    let zoom = zoom.max(0.0001);
    let min = pos2(rect.min.x * zoom + pan.x, rect.min.y * zoom + pan.y);
    Rect::from_min_size(min, rect.size() * zoom)
}

impl AppStella {
    pub fn draw_workbench_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("workbenchmenu_panel").show(ctx, |ui| {
            ui.add_space(3.0);
            egui::MenuBar::new().ui(ui, |ui| {
                if ui
                    .add(
                        egui::Button::new("Table")
                            .min_size(vec2(120.0, 25.0))
                            .stroke(egui::Stroke::new(1.0, crate::app::BLUE)),
                    )
                    .clicked()
                {
                    self.dispatch(Command::CreateTable {
                        title: Table::default().title,
                    });
                }
                if ui
                    .add(
                        egui::Button::new("Domain")
                            .min_size(vec2(120.0, 25.0))
                            .stroke(egui::Stroke::new(1.0, crate::app::GREEN)),
                    )
                    .clicked()
                {
                    let domain = Domain::default();
                    self.dispatch(Command::CreateDomain {
                        name: domain.name,
                        data_type: domain.data_type,
                    });
                }
            });
            ui.add_space(2.0);
        });
    }

    pub fn draw_workbench(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workbench");
            let canvas_rect = ui.available_rect_before_wrap().shrink(CANVAS_PADDING);
            let canvas_id = Id::new("workbench_canvas");
            let canvas_response = ui.interact(canvas_rect, canvas_id, Sense::drag());

            ui.painter()
                .rect_filled(canvas_rect, 8.0, ui.visuals().faint_bg_color);

            let is_modifier_pan = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
            let drag_pan_active = canvas_response.dragged_by(PointerButton::Primary)
                && is_modifier_pan
                && !ui.ctx().is_pointer_over_area();

            if drag_pan_active {
                let delta = ui.input(|i| i.pointer.delta());
                if delta != Vec2::ZERO {
                    self.workbench_pan += delta;
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                }
            }

            // Touchpads often emit pan gestures as scroll deltas instead of pointer drags.
            if ui.rect_contains_pointer(canvas_rect) && !is_modifier_pan {
                let scroll_delta = ui.input(|i| i.smooth_scroll_delta);
                if scroll_delta != Vec2::ZERO {
                    self.workbench_pan += scroll_delta;
                }
            }

            let mut any_table_dragging = false;
            let mut table_to_delete: Option<TableId> = None;
            let mut table_commands: Vec<Command> = Vec::new();
            let mut screen_table_rects: HashMap<TableId, egui::Rect> = HashMap::new();
            let mut world_table_rects: HashMap<TableId, egui::Rect> = HashMap::new();
            let table_keys: Vec<TableId> = self.tables.keys().collect();

            for (index, id) in table_keys.into_iter().enumerate() {
                let title = self.tables[id].title.clone();
                let mut world_rect = self
                    .workbench_table_rects
                    .get(&id)
                    .copied()
                    .unwrap_or_else(|| default_workbench_rect(index));
                let zoom = 1.0;
                let screen_rect = world_to_screen_rect(world_rect, self.workbench_pan, zoom);
                let area = Area::new(Id::new(id))
                    .order(Order::Background)
                    .constrain(false)
                    .fixed_pos(screen_rect.min);

                let mut should_delete = false;
                let mut drag_delta = Vec2::ZERO;

                let response = area.show(ctx, |ui| {
                    ui.set_min_size(screen_rect.size());
                    Frame::window(ui.style())
                        .fill(ui.visuals().window_fill())
                        .stroke(ui.visuals().window_stroke())
                        .show(ui, |ui| {
                            Frame::new()
                                .fill(ui.visuals().widgets.noninteractive.bg_fill)
                                .stroke(ui.visuals().window_stroke())
                                .inner_margin(egui::Margin::symmetric(8, 6))
                                .show(ui, |ui| {
                                    let header_response = ui
                                        .add_sized(
                                            [ui.available_width(), 18.0],
                                            egui::Label::new(egui::RichText::new(&title).strong())
                                                .sense(Sense::drag()),
                                        )
                                        .on_hover_text("Drag table");

                                    if header_response.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                                    }
                                    if header_response.dragged() {
                                        drag_delta = ui.input(|i| i.pointer.delta());
                                        any_table_dragging = true;
                                    }
                                });

                            ui.add_space(2.0);
                            ui.separator();

                            let ui_ctx = TableUiContext::from_app(&self.tables, &self.domains, id);
                            let table = self.tables.get_mut(id).expect("table missing");
                            let changes = table.draw(ui, &ui_ctx, id);
                            extend_commands(&mut table_commands, changes);

                            ui.separator();
                            if ui.button("Delete").clicked() {
                                should_delete = true;
                            }
                        });
                });

                if drag_delta != Vec2::ZERO {
                    world_rect = world_rect.translate(drag_delta / zoom.max(0.0001));
                }

                let shown_rect = response.response.rect;
                let shown_world_size = shown_rect.size() / zoom.max(0.0001);
                world_rect = Rect::from_min_size(world_rect.min, shown_world_size);
                world_table_rects.insert(id, world_rect);
                screen_table_rects.insert(
                    id,
                    world_to_screen_rect(world_rect, self.workbench_pan, zoom),
                );

                if should_delete {
                    table_to_delete = Some(id);
                }
            }

            self.workbench_table_rects = world_table_rects;
            let relation_painter = ui.painter();
            for edge in build_edges(&self.tables, &screen_table_rects) {
                draw_crow_foot_edge(relation_painter, &edge);
            }

            for cmd in table_commands {
                self.dispatch(cmd);
            }

            if let Some(idx) = table_to_delete {
                self.dispatch(Command::DeleteTable { table: idx });
            }

            if any_table_dragging {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.label("Pan: Ctrl+drag.");
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
