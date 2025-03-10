#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use std::fs::File;
use binc::node_id::NodeId;
use bincgui::app::{create_toolbar, Application};
use eframe::{egui, App, CreationContext, Storage};
use binc::document::Document;

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Got issues",
        options,
        Box::new(|cc| Ok(IssuesApp::new_or_from_storage(cc))),
    )
}

struct IssuesApp {
    application: Application,
    search_string: String,
    found_issues: Vec<NodeId>,
}

impl IssuesApp {
    fn new_or_from_storage(cc: &CreationContext) -> Box<dyn App> {
        if let Some(storage) = cc.storage {
            if let Some(path) = storage.get_string("document_path") {
                if let Ok(mut file) = File::open(&path) {
                    if let Ok(doc) = Document::read(&mut file) {
                        let mut app = IssuesApp::new();
                        app.application.set_document(doc);
                        return Box::new(app);
                    }
                }
            }
        }
        Box::new(IssuesApp::new())
    }
}

impl IssuesApp {
    fn new() -> Self {
        Self {
            application: Application::new(),
            search_string: String::new(),
            found_issues: vec![],
        }
    }

    fn update_search(&mut self) {
        self.found_issues = self.get_issues_for_search(&self.search_string, 30);
    }

    fn get_issues_for_search(&self, search_string: &str, limit: usize) -> Vec<NodeId> {
        if !search_string.is_empty() {
            let search_string = search_string.to_lowercase();
            let terms = search_string.split(" ");

            if let Some(issue_id) = self.application.document.nodes.type_names.get_index("issue") {
                let mut issues = vec![];
                let summary_id = self.application.document.nodes.attribute_names.get_index("summary");

                for node in self.application.document.nodes.nodes().iter().rev() {
                    if Some(issue_id) == node.type_id {
                        if let Some(summary) = node.get_string_attribute(summary_id.unwrap()) {
                            let mut found = true;
                            let mut t = terms.clone();
                            while let Some(term) = t.next() {
                                if !summary.to_lowercase().contains(&term) {
                                    found = false;
                                    break;
                                }
                            }
                            if found {
                                issues.push(node.id);

                                if issues.len() >= limit {
                                    break;
                                }
                            }
                        }
                    }
                }
                return issues;
            }
        }
        vec![]
    }
}

impl eframe::App for IssuesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let summary_id = self.application.document.nodes.attribute_names.get_index("summary").unwrap_or(0);
        let permalink_id = self.application.document.nodes.attribute_names.get_index("permalink").unwrap_or(0);
        let assignee_id = self.application.document.nodes.attribute_names.get_index("assignee").unwrap_or(0);
        let status_id = self.application.document.nodes.attribute_names.get_index("status").unwrap_or(0);

        let frame = egui::Frame::default()
            .inner_margin(8.0)
            .fill(ctx.style().visuals.panel_fill);
        egui::TopBottomPanel::top("toolbar")
            .frame(frame)
            .show(ctx, |ui| {
                create_toolbar(&mut self.application, ui, |ui| {});
            });

        egui::CentralPanel::default().show(ctx, |ui| {

            ui.vertical_centered(|ui| {
                if ui.text_edit_singleline(&mut self.search_string).changed() {
                    self.update_search();
                }

                ui.separator();

                let f = egui::Frame::default()
                    .inner_margin(4.0)
                    .corner_radius(4)
                    .fill(ctx.style().visuals.panel_fill)
                    .shadow(ctx.style().visuals.window_shadow)
                    .stroke(ctx.style().visuals.widgets.noninteractive.bg_stroke);

                for id in &self.found_issues {
                    f.show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            let node = self.application.document.nodes.get(*id).unwrap();
                            let key = node.get_name().unwrap_or("?");
                            let label = node.get_string_attribute(summary_id).unwrap_or("?");
                            let status = node.get_string_attribute(status_id).unwrap_or("?").to_string();
                            let assignee = node.get_string_attribute(assignee_id).unwrap_or("?");
                            let url = node.get_string_attribute(permalink_id) .unwrap_or("?");
                            /*ComboBox::from_label(status.clone()).show_ui(ui, |ui| {
                                /*ui.selectable_value(&mut status, "Open".to_string(), "Open");
                                ui.selectable_value(&mut status, "Resolved".to_string(), "Resolved");
                                ui.selectable_value(&mut status, "Closed".to_string(), "Closed");*/
                            });*/
                            ui.label(status);
                            ui.hyperlink_to(key, url);
                            ui.label(label);
                            ui.label(assignee);
                        })
                    });
                }
            })
        });
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        if let Some(path) = &self.application.document_path {
            storage.set_string("document_path", path.to_str().unwrap().to_string());
        }
    }
}
