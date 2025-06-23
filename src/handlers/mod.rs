use std::fmt::Display;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Types of handlers
pub mod courses;
pub mod modules;
pub mod tasks;
pub mod users;

