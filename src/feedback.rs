use crate::prelude::*;
use egui::Ui;
use octocrab::Octocrab;

#[derive(PartialEq)]
enum SendIssueStatus {
    NotSent,
    Sending,
    Finished,
}

pub struct Feedback {
    product: Product,
    title: String,
    description: String,
    send_status: Arc<Mutex<SendIssueStatus>>,
}

impl Feedback {
    pub fn empty() -> Self {
        Self {
            product: Product::TwelveKnightsVigil,
            title: format!("Untitled"),
            description: format!("Enter a description"),
            send_status: Arc::new(Mutex::new(SendIssueStatus::NotSent)),
        }
    }

    fn send(&mut self) {
        error!("Unimplemented!");
        tokio::spawn(async move {});
    }
}

impl crate::LauncherApp {
    pub fn draw_feedback_widget(&mut self, ui: &mut Ui) {
        ui.label("Send Feedback or Request a Feature");

        let status = self.feedback.send_status.clone();
        let send_status = status.lock().unwrap();
        ui.add_enabled_ui(*send_status == SendIssueStatus::NotSent, |ui| {
            ui.horizontal(|ui| {
                ui.label("Product: ");
                let current_product = self.feedback.product.clone();
                let menu_label = format!("{}", self.feedback.product.display_name());
                ui.menu_button(menu_label, |ui| {
                    crate::Product::all()
                        .iter()
                        .filter(|p| **p != current_product)
                        .for_each(|p| {
                            if ui.button(p.display_name()).clicked() {
                                self.feedback.product = *p;
                            }
                        });
                });
            });

            ui.text_edit_singleline(&mut self.feedback.title);
            ui.text_edit_multiline(&mut self.feedback.description);
            if ui.button("Send").clicked() {
                self.feedback.send();
            }
        });
    }
}
