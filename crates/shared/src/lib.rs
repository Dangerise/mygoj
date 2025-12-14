use serde::{Deserialize, Serialize};

pub mod judge;
pub mod problem;
pub mod record;
pub mod submission;
pub mod front;
pub mod user;
pub mod token;
pub mod headers;

use token::*;
use problem::*;
use submission::*;
use record::*;
use judge::*;
use user::*;
