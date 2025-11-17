use std::{
    sync::mpsc::{Receiver, channel},
    time::Duration,
};

use ratatui::crossterm::event::{self, Event};

use crate::{
    PLAN_PATH,
    config::Config,
    scan_dir::{self, move_file},
};

pub enum MsEvent {
    Crossterm(std::io::Result<Event>),
    PlanMoved,
}
pub fn setup(config: &Config) -> Receiver<MsEvent> {
    let (tx, rx) = channel();
    let tx2 = tx.clone();
    std::thread::spawn(move || {
        loop {
            tx2.send(MsEvent::Crossterm(event::read())).unwrap();
        }
    });
    if let Some(scan_path) = config.scan_path.clone() {
        // let scan_path = scan_path.clone();
        std::thread::spawn(move || {
            loop {
                if move_file(&scan_path, PLAN_PATH).unwrap() {
                    tx.send(MsEvent::PlanMoved).unwrap();
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        });
    }

    rx
}
