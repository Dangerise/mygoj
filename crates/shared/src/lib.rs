use serde::{Deserialize, Serialize};

pub mod judge;
pub mod problem;
pub mod record;
pub mod submission;
pub mod front;

use problem::*;
use submission::*;
use record::*;
use judge::*;
