use super::checksum::Checksum;
use super::Event;
use crossbeam::channel::Sender;
use std::sync::{Arc, Mutex};

pub struct Package {
    pub name: String,
}

pub struct PackageDownloader {
    pkgs: Vec<String>,
    event_sender: Sender<Event>,
}

impl PackageDownloader {
    pub fn new(pkgs: Vec<String>, event_sender: Sender<Event>) -> Self {
        Self {
            pkgs,
            event_sender,
        }
    }

    pub fn run(&self, pkg_checksum: Arc<Mutex<Checksum>>) {
        // Generate a set of packages and place them into the event queue
        // Update the package checksum with each package name
        for name in self.pkgs.to_vec() {
            pkg_checksum
                .lock()
                .unwrap()
                .update(Checksum::with_sha256(&name));
            self.event_sender
                .send(Event::DownloadComplete(Package { name }))
                .unwrap();
        }
    }
}
