#![warn(clippy::all)]
pub mod checksum;
pub mod idea;
pub mod package;
pub mod student;
pub mod printer;

use idea::Idea;
use package::Package;

// Some means newly generated idea for students to work on, None means termination event for student threads
pub struct IdeasEvent {
    pub idea: Option<Idea>
}

// Packages that students can take to work on their ideas
pub struct DownloadCompleteEvent {
    pub package: Package
}
