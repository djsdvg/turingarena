#![feature(decl_macro, proc_macro_hygiene)]
#![feature(external_doc)]
#![doc(include = "../README.md")]

#[cfg(feature = "diesel")]
#[macro_use]
extern crate diesel;

#[cfg(feature = "diesel_migrations")]
#[macro_use]
extern crate diesel_migrations;

#[macro_use]
extern crate serde;

#[macro_use]
pub mod juniper_ext;

#[macro_use]
extern crate log;

pub mod award;
pub mod content;

#[cfg(feature = "contest")]
pub mod contest;

pub mod evallib;
pub mod evaluation;
pub mod exitstatus;
pub mod feedback;
pub mod file;
pub mod problem;
pub mod rusage;
pub mod submission;

#[cfg(feature = "task-maker")]
pub mod task_maker;

#[cfg(feature = "archive")]
mod archive;
