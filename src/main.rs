use std::cell::RefCell;
use std::rc::Rc; 

use rtree::nodes::{ChildrenType, RtreeNode, RtreeObject};
use rtree::geometries::{BoundingRectangle, RtreeGeometry};

fn main() {
    let bounding = BoundingRectangle::new((5, 6), (5, 6));
    let bounding_root = BoundingRectangle::new((5, 6), (5, 6));
    let mut root = RtreeNode::new(bounding_root, 4, ChildrenType::InnerNodes);
    let mut node = Rc::new(RefCell::new(RtreeNode::new(bounding, 4, ChildrenType::Leafs)));
    let mut geometry = Rc::new(RefCell::new(RtreeGeometry::new(vec!((5, 6)))));
    node.borrow_mut().children = ChildrenType::Leafs(vec!(geometry));
    root.children = ChildrenType::InnerNodes(vec!(node));
    let mut geometry_2 = Rc::new(RefCell::new(RtreeGeometry::new(vec!((1, 2), (3, 4), (1, 2)))));
    root.insert(geometry_2);
    root.print("");
}
