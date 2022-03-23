use super::checksum::Checksum;
use super::IdeasEvent;
use crossbeam::channel::Sender;
use std::sync::{Arc, Mutex};

pub struct Idea {
    pub name: String,
    pub num_pkg_required: usize,
}

pub struct IdeaGenerator {
    idea_start_idx: usize,
    num_ideas: usize,
    num_students: usize,
    num_pkgs: usize,
    ideas_event_sender: Sender<IdeasEvent>
}

impl IdeaGenerator {
    pub fn new(
        idea_start_idx: usize,
        num_ideas: usize,
        num_students: usize,
        num_pkgs: usize,
        ideas_event_sender: Sender<IdeasEvent>
    ) -> Self {
        Self {
            idea_start_idx,
            num_ideas,
            num_students,
            num_pkgs,
            ideas_event_sender
        }
    }

    // Idea names are generated from cross products between product names and customer names
    fn get_next_idea_name(idx: usize, ideas: &Vec<(String, String)>) -> String {
        let pair = ideas.get(idx % ideas.len()).unwrap();
        format!("{} for {}", pair.0, pair.1)
    }

    pub fn cross_product(products: String, customers: String) -> Vec<(String, String)> {
        products
            .lines()
            .flat_map(|p| customers.lines().map(move |c| (p.to_owned(), c.to_owned())))
            .collect()
    }

    pub fn run(&self, idea_checksum: Arc<Mutex<Checksum>>, ideas: Arc<Vec<(String, String)>>) {
        let pkg_per_idea = self.num_pkgs / self.num_ideas;
        let extra_pkgs = self.num_pkgs % self.num_ideas;

        // Generate a set of new ideas and place them into the event-queue
        // Update the idea checksum with all generated idea names
        let mut checksum_xor_temp = Checksum::default();
        for i in 0..self.num_ideas {
            let name = Self::get_next_idea_name(self.idea_start_idx + i, ideas.as_ref());
            let extra = (i < extra_pkgs) as usize;
            let num_pkg_required = pkg_per_idea + extra;
            let idea = Idea {
                name,
                num_pkg_required,
            };

            checksum_xor_temp.update(Checksum::with_sha256(&idea.name));

            idea_checksum
                .lock()
                .unwrap()
                .update(Checksum::with_sha256(&idea.name));

            self.ideas_event_sender.send(IdeasEvent { idea: Some(idea) }).unwrap();
        }
        idea_checksum
            .lock()
            .unwrap()
            .update(checksum_xor_temp);

        // Push student termination events into the event queue
        for _ in 0..self.num_students {
            self.ideas_event_sender.send(IdeasEvent { idea: None }).unwrap();
        }
    }
}
