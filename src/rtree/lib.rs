use std::f64::{INFINITY, NEG_INFINITY};
use std::fmt::{Display, Debug, Formatter};
use std::fmt::Result as FmtResult;

pub type Coordinates = (i64, i64);
pub type Geometry = Vec<Coordinates>;
pub type Geoms = Vec<Geometry>;

const INF: i64 = INFINITY as i64;
const NEGINF: i64 = NEG_INFINITY as i64;

pub mod geometries;
pub mod nodes;
pub mod split;
mod utils;