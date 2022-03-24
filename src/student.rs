use super::{checksum::Checksum, idea::Idea, package::Package, DownloadCompleteEvent, IdeasEvent};
use crossbeam::channel::{Receiver, Sender};
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
    print_sender: Sender<String>
}

impl Student {
    pub fn new(
        id: usize,
        ideas_event_sender: Sender<IdeasEvent>,
        ideas_event_recv: Receiver<IdeasEvent>,
        download_complete_event_sender: Sender<DownloadCompleteEvent>,
        download_complete_event_recv: Receiver<DownloadCompleteEvent>,
        print_sender: Sender<String>
    ) -> Self {
        Self {
            id,
            ideas_event_sender,
            ideas_event_recv,
            download_complete_event_sender,
            download_complete_event_recv,
            print_sender,
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
                let mut checksum_xor_temp = Checksum::default();
                for pkg in pkgs_used.iter() {
                    checksum_xor_temp.update(Checksum::with_sha256(&pkg.name));
                }
                {
                    let mut pkg_checksum = pkg_checksum.lock().unwrap();
                    pkg_checksum.update(checksum_xor_temp);
                    pkg_checksum_copy_for_print = pkg_checksum.to_string();
                }

                // Send message to print to printing thread
                let mut student_print = format!("\nStudent {} built {} using {} packages\nIdea checksum: {}\nPackage checksum: {}",
                                        self.id, idea.name, pkgs_required, idea_checksum_copy_for_print, pkg_checksum_copy_for_print);
                for pkg in pkgs_used.iter() {
                    student_print.push_str(format!("\n> {}", pkg.name).as_str());
                }
                self.print_sender.send(student_print).unwrap();

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
                                // Send message to kill the corresponding printing thread
                                self.print_sender.send(String::new()).unwrap();
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
