use crate::spawner::LocalSpawner;
use eframe::egui;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum WalletMessage {
    CreateNewWallet,
}

pub struct App {
    spawner: LocalSpawner,
    tx: mpsc::UnboundedSender<WalletMessage>,
}

impl App {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        spawner: LocalSpawner,
        tx: mpsc::UnboundedSender<WalletMessage>,
    ) -> Self {
        Self { spawner, tx }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Create new wallet").clicked() {
                let _ = self.tx.send(WalletMessage::CreateNewWallet);
            }
        });
    }
}
