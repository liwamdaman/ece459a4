use super::{checksum::Checksum, idea::Idea, package::Package, DownloadCompleteEvent, IdeasEvent};
use crossbeam::channel::{Receiver, Sender};
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};

pub struct Student {
    id: usize,
    idea: Option<Idea>,
    pkgs: Vec<Package>,
    skipped_idea: bool,
    ideas_event_sender: Sender<IdeasEvent>,
    ideas_event_recv: Receiver<IdeasEvent>,
    download_complete_event_sender: Sender<DownloadCompleteEvent>,
    download_complete_event_recv: Receiver<DownloadCompleteEvent>,
}

impl Student {
    pub fn new(
        id: usize,
        ideas_event_sender: Sender<IdeasEvent>,
        ideas_event_recv: Receiver<IdeasEvent>,
        download_complete_event_sender: Sender<DownloadCompleteEvent>,
        download_complete_event_recv: Receiver<DownloadCompleteEvent>,
    ) -> Self {
        Self {
            id,
            ideas_event_sender,
            ideas_event_recv,
            download_complete_event_sender,
            download_complete_event_recv,
            idea: None,
            pkgs: vec![],
            skipped_idea: false,
        }
    }

    fn build_idea(
        &mut self,
        idea_checksum: &Arc<Mutex<Checksum>>,
        pkg_checksum: &Arc<Mutex<Checksum>>,
    ) {
        if let Some(ref idea) = self.idea {
            // Can only build ideas if we have acquired sufficient packages
            let pkgs_required = idea.num_pkg_required;
            if pkgs_required <= self.pkgs.len() {
                // Update idea and package checksums
                // All of the packages used in the update are deleted, along with the idea
                let idea_checksum_copy_for_print;
                {
                    let mut idea_checksum = idea_checksum.lock().unwrap();
                    idea_checksum.update(Checksum::with_sha256(&idea.name));
                    idea_checksum_copy_for_print = idea_checksum.to_string();
                }
                let pkg_checksum_copy_for_print;
                let pkgs_used = self.pkgs.drain(0..pkgs_required).collect::<Vec<_>>();
                {
                    let mut pkg_checksum = pkg_checksum.lock().unwrap();
                    for pkg in pkgs_used.iter() {
                        pkg_checksum.update(Checksum::with_sha256(&pkg.name));
                    }
                    pkg_checksum_copy_for_print = pkg_checksum.to_string();
                }

                // We want the subsequent prints to be together, so we lock stdout
                let stdout = stdout();
                let mut handle = stdout.lock();
                writeln!(handle, "\nStudent {} built {} using {} packages\nIdea checksum: {}\nPackage checksum: {}",
                    self.id, idea.name, pkgs_required, idea_checksum_copy_for_print, pkg_checksum_copy_for_print).unwrap();
                for pkg in pkgs_used.iter() {
                    writeln!(handle, "> {}", pkg.name).unwrap();
                }

                self.idea = None;
            }
        }
    }

    pub fn run(&mut self, idea_checksum: Arc<Mutex<Checksum>>, pkg_checksum: Arc<Mutex<Checksum>>) {
        loop {
            match self.ideas_event_recv.try_recv() {
                Ok(ideas_event) => {
                    match ideas_event.idea {
                        Some(idea) => {
                            // If the student is not working on an idea, then they will take the new idea
                            // and attempt to build it. Otherwise, the idea is skipped.
                            if self.idea.is_none() {
                                self.idea = Some(idea);
                                self.build_idea(&idea_checksum, &pkg_checksum);
                            } else {
                                self.ideas_event_sender.send(IdeasEvent { idea: Some(idea) }).unwrap();
                                self.skipped_idea = true;
                            }
                        }
                        None => {
                            // If an idea was skipped, it may still be in the event queue.
                            // If the student has an unfinished idea, they have to finish it, since they
                            // might be the last student remaining.
                            // In both these cases, we can't terminate, so the termination event is
                            // deferred ti the back of the queue.
                            if self.skipped_idea || self.idea.is_some() {
                                self.ideas_event_sender.send(IdeasEvent { idea: None }).unwrap();
                                self.skipped_idea = false;
                            } else {
                                // Any unused packages are returned to the queue upon termination
                                for pkg in self.pkgs.drain(..) {
                                    self.download_complete_event_sender
                                        .send(DownloadCompleteEvent { package: pkg })
                                        .unwrap();
                                }
                                return;
                            }
                        }
                    }
                }
                Err(_) => {} // Don't need to do anything, it was a try_recv() anyways
            }
            match self.download_complete_event_recv.try_recv() {
                Ok(download_complete_event) => {
                    // Getting a new package means the current idea may now be buildable, so the
                    // student attempts to build it
                    self.pkgs.push(download_complete_event.package);
                    self.build_idea(&idea_checksum, &pkg_checksum);
                }
                Err(_) => {} // Don't need to do anything, it was a try_recv() anyways
            }
        }
    }
}
