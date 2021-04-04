use std::cmp;
use std::fmt::{Display, Debug, Formatter};
use std::fmt::Result as FmtResult;
use std::rc::{Rc, Weak};

use crate::{INF, NEGINF, Coordinates, Geometry};
use crate::nodes::{RtreeNode, RtreeObject};
use crate::utils::generate_id;

#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum GeometryType {
    Point,
    Polygon,
    Line
}

#[derive(Copy, Clone)]
pub struct BoundingRectangle {
    pub left: Coordinates,
    pub right: Coordinates,
    pub area: i64
}

pub struct RtreeGeometry {
    pub id: String,
    pub coords: Geometry,
    pub mbr: BoundingRectangle,
    pub coordtype: GeometryType,
    parent: Option<Weak<RtreeNode>>
}

impl Display for GeometryType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Line => f.write_str("Line"),
            Self::Polygon => f.write_str("Polygon"),
            Self::Point => f.write_str("Point")
        }
    }
}

impl Display for BoundingRectangle {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_fmt(format_args!(
            "BL: ({}, {}), UR: ({}, {})",
            self.left.0,
            self.left.1,
            self.right.0,
            self.right.1
        ))
    }
}

impl BoundingRectangle {

    pub fn new(left: Coordinates, right: Coordinates) -> BoundingRectangle {
        BoundingRectangle {
            left: left,
            right: right,
            area: BoundingRectangle::count_area(&left, &right)
        }
    }

    fn count_area(left: &Coordinates, right: &Coordinates) -> i64 {
        (right.0 - left.0) * (right.1 - left.1)
    }

    pub fn generate_mbr(coords: &Geometry) -> BoundingRectangle {
        let mut min_x = INF;
        let mut min_y = INF;
        let mut max_x = NEGINF;
        let mut max_y = NEGINF;

        for coord in coords {
            let x = coord.0;
            let y = coord.1;

            if x < min_x {
                min_x = x
            } else if x > max_x {
                max_x = x
            };

            if y < min_y {
                min_y = y
            } else if y > max_y {
                max_y = y
            };
        }
        BoundingRectangle::new((min_x, min_y), (max_x, max_y))
    }

    pub fn overlap_rectangle(rect_1: &BoundingRectangle,
                            rect_2: &BoundingRectangle) -> BoundingRectangle {
        let left = (
            cmp::max(rect_1.left.0, rect_2.left.0),
            cmp::max(rect_1.left.1, rect_2.left.1)
        );
        let right = (
            cmp::min(rect_1.right.0, rect_2.right.0),
            cmp::min(rect_1.right.1, rect_2.right.1)
        );
        BoundingRectangle::new(left, right)
    }

    pub fn common_mbr(list_mbrs: &Vec<&BoundingRectangle>) -> BoundingRectangle {
        let mut min_x = INF;
        let mut min_y = INF;
        let mut max_x = NEGINF;
        let mut max_y = NEGINF;

        for mbr in list_mbrs {
            if mbr.left.0 < min_x {
                min_x = mbr.left.0
            };
            if mbr.right.0 > max_x {
                max_x = mbr.right.0
            };
            if mbr.left.1 < min_y {
                min_y = mbr.left.1
            }
            if mbr.right.1 > max_y {
                max_y = mbr.right.1
            };
        }

        BoundingRectangle::new((min_x, min_y), (max_x, max_y))
    }

    pub fn intersects(&self, rectangle: &BoundingRectangle) -> bool {
        if (self.left.0 > rectangle.right.0) ||
                (rectangle.left.0 > self.right.0) {
            false
        } else if (self.right.1 < rectangle.left.1) ||
                (rectangle.right.1 < self.left.1) {
            false
        } else {
            true
        }
    }
}

impl Display for RtreeGeometry {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_fmt(format_args!(
            "{} with mbr {}",
            self.coordtype,
            self.mbr
        ))
    }
}

impl RtreeGeometry {

    pub fn new(coords: Geometry) -> RtreeGeometry {
        let length = coords.len();
        let coordtype = if length < 2 {
            GeometryType::Point
        } else if coords[0] == coords[length - 1] {
            GeometryType::Polygon
        } else {
            GeometryType::Line
        };
        
        let mbr = RtreeGeometry::find_mbr(&coordtype, &coords);
        RtreeGeometry {
            id: generate_id(),
            coords,
            mbr,
            coordtype,
            parent: None
        }
    }

    fn find_mbr(coordtype: &GeometryType, coords: &Geometry) -> BoundingRectangle {
        match *coordtype {
            GeometryType::Point => BoundingRectangle::new(coords[0], coords[0]),
            _ => {
                BoundingRectangle::generate_mbr(&coords)
            }
        }
    }
}

impl RtreeObject for RtreeGeometry {

    fn id(&self) -> &str {
        &self.id
    }

    fn mbr(&self) -> &BoundingRectangle {
        &self.mbr
    }

    fn set_mbr(&mut self, mbr: BoundingRectangle) -> () {
        self.mbr = mbr
    }

    fn set_parent(&mut self, node: &Rc<RtreeNode>) -> () {
        self.parent = Some(Rc::downgrade(node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mbr_creation() {
        let left = (5, 2);
        let right = (9, 7);
        let rect = BoundingRectangle::new(left, right);
        assert_eq!(rect.left, left);
        assert_eq!(rect.right, right);
        assert_eq!(rect.area, 20);
    }

    #[test]
    fn test_generate_mbr() {
        let coords_rect = &vec!((2, 2), (6, 2), (6, 4), (2, 4), (2, 2));
        let coords_line = &vec!((2, 2), (6, 4));

        let rect_r = BoundingRectangle::generate_mbr(coords_rect);
        let rect_l = BoundingRectangle::generate_mbr(coords_line);

        assert_eq!(rect_r.left, (2, 2));
        assert_eq!(rect_r.right, (6, 4));

        assert_eq!(rect_l.left, (2, 2));
        assert_eq!(rect_r.right, (6, 4));
    }

    #[test]
    fn test_overlap_rectangle() {
        let rect_1 = BoundingRectangle::new(
            (2, 1), (5, 3)
        );
        let rect_2 = BoundingRectangle::new(
            (4, 2), (7, 4)
        );

        let overlap = BoundingRectangle::overlap_rectangle(&rect_1, &rect_2);

        assert_eq!(overlap.left, (4, 2));
        assert_eq!(overlap.right, (5, 3));
    }

    #[test]
    fn test_common_mbr() {
        let rect_1 = BoundingRectangle::new(
            (2, 1), (5, 3)
        );
        let rect_2 = BoundingRectangle::new(
            (4, 2), (7, 4)
        );
        let rect_3 = BoundingRectangle::new(
            (3, 0), (6, 2)
        );

        let common = BoundingRectangle::common_mbr(&vec!(&rect_1, &rect_2, &rect_3));

        assert_eq!(common.left, (2, 0));
        assert_eq!(common.right, (7, 4));
    }

    #[test]
    fn test_intersects() {
        let rect_1 = BoundingRectangle::new(
            (2, 1), (5, 3)
        );
        let rect_2 = BoundingRectangle::new(
            (4, 2), (7, 4)
        );
        let rect_3 = BoundingRectangle::new(
            (3, 0), (6, 1)
        );

        assert_eq!(rect_1.intersects(&rect_2), true);
        assert_eq!(rect_2.intersects(&rect_3), false);
        assert_eq!(rect_1.intersects(&rect_3), true);
    }

    #[test]
    fn test_create_geometry() {
        let coords_line = vec!((2, 1), (5, 3));
        let coords_rect = vec!((2, 2), (6, 2), (6, 4), (2, 4), (2, 2));
        let coords_point = vec!((8, 6));

        let line = RtreeGeometry::new(coords_line);
        let rect = RtreeGeometry::new(coords_rect);
        let point = RtreeGeometry::new(coords_point);

        assert_eq!(line.coordtype, GeometryType::Line);
        assert_eq!(rect.coordtype, GeometryType::Polygon);
        assert_eq!(point.coordtype, GeometryType::Point);

        assert_eq!(line.mbr.left, (2, 1));
        assert_eq!(line.mbr.right, (5, 3));

        assert_eq!(rect.mbr.left, (2, 2));
        assert_eq!(rect.mbr.right, (6, 4));

        assert_eq!(point.mbr.left, point.mbr.right);
    }
}
