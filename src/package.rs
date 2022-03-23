use super::checksum::Checksum;
use super::DownloadCompleteEvent;
use crossbeam::channel::Sender;
use std::sync::{Arc, Mutex};

pub struct Package {
    pub name: String,
}

pub struct PackageDownloader {
    pkgs: Vec<String>,
    download_complete_event_sender: Sender<DownloadCompleteEvent>,
}

impl PackageDownloader {
    pub fn new(pkgs: Vec<String>, download_complete_event_sender: Sender<DownloadCompleteEvent>) -> Self {
        Self {
            pkgs,
            download_complete_event_sender,
        }
    }

    pub fn run(&self, pkg_checksum: Arc<Mutex<Checksum>>) {
        let mut checksum_xor_temp = Checksum::default();
        for name in self.pkgs.to_vec() {
            // Generate a set of packages and place them into the event queue
            // Update the package checksum with each package name
            checksum_xor_temp.update(Checksum::with_sha256(&name));
            self.download_complete_event_sender
                .send(DownloadCompleteEvent { package: Package { name } })
                .unwrap();
        }
        pkg_checksum
            .lock()
            .unwrap()
            .update(checksum_xor_temp);
    }
}
