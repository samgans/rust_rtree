use std::rc::Rc;

use crate::geometries::{BoundingRectangle, RtreeGeometry};
use crate::nodes::{ChildrenType, RtreeNode, RtreeObject};

pub trait RtreeSplit {
    fn split(&mut self) -> ();

    fn execute(&mut self) -> (Rc<RtreeNode>, Rc<RtreeNode>);

    fn pick_seeds<T: RtreeObject>(
        &mut self,
        objects: &mut Vec<Rc<T>>, 
        child_type: fn(Vec<Rc<T>>) -> ChildrenType) -> (Rc<RtreeNode>, Rc<RtreeNode>);

    fn distribute_nodes(
        &self,
        node_1: &mut Rc<RtreeNode>,
        node_2: &mut Rc<RtreeNode>,
        objects: &mut Vec<Rc<RtreeNode>>
    ) -> ();

    fn distribute_leafs(
        &self,
        node_1: &mut Rc<RtreeNode>,
        node_2: &mut Rc<RtreeNode>,
        objects: &mut Vec<Rc<RtreeGeometry>>
    ) -> ();

    fn pick_next_node(
        &self,
        node_1: &mut Rc<RtreeNode>,
        node_2: &mut Rc<RtreeNode>,
        objects: &mut Vec<Rc<RtreeNode>>)
    -> Rc<RtreeNode>;

    fn pick_next_leaf(
        &self,
        node_1: &mut Rc<RtreeNode>,
        node_2: &mut Rc<RtreeNode>,
        objects: &mut Vec<Rc<RtreeGeometry>>
    ) -> Rc<RtreeGeometry>;

    fn validate_nodes_quantity(
        &self,
        node_1: &mut Rc<RtreeNode>,
        node_2: &mut Rc<RtreeNode>,
        objects: &mut Vec<Rc<RtreeNode>>
    ) -> bool;

    fn validate_leafs_quantity(
        &self,
        node_1: &mut Rc<RtreeNode>,
        node_2: &mut Rc<RtreeNode>,
        objects: &mut Vec<Rc<RtreeGeometry>>
    ) -> bool;

    fn pick_underfull<'a>(
        &'a self,
        node_1: &'a mut Rc<RtreeNode>,
        node_2: &'a mut Rc<RtreeNode>
    ) -> Option<&'a mut Rc<RtreeNode>>;
}