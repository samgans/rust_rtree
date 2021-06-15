use std::cell::RefCell;
use std::fmt::{Display, Debug, Formatter};
use std::fmt::Result as FmtResult;
use std::mem::MaybeUninit;
use std::rc::{Rc, Weak};

use crate::{INF, NEGINF};
use crate::geometries::{BoundingRectangle, RtreeGeometry};
use crate::split::RtreeSplit;
use crate::utils::{find_least_enlargement, generate_id};


pub type TreeNode = Rc<RefCell<RtreeNode>>;
pub type TreeGeometry = Rc<RefCell<RtreeGeometry>>;


pub trait RtreeObject {
    fn id(&self) -> &str;
    fn mbr(&self) -> &BoundingRectangle;
    fn set_mbr(&mut self, mbr: BoundingRectangle) -> ();
    fn set_parent(&mut self, node: &TreeNode) -> ();
}

pub enum ChildrenType {
    InnerNodes(Vec<TreeNode>),
    Leafs(Vec<TreeGeometry>)
}

pub struct RtreeNode {
    pub id: String,
    pub children: ChildrenType,
    pub mbr: BoundingRectangle,
    pub max_children: u8,
    pub parent: Option<Weak<RefCell<RtreeNode>>>
}

impl ChildrenType {
    fn len(&self) -> usize {
        match self {
            Self::InnerNodes(ref nodes) => {
                nodes.len()
            },
            Self::Leafs(ref leafs) => {
                leafs.len()
            }
        }
    }

    fn add_node(&mut self, object: TreeNode) {
        if let Self::InnerNodes(ref mut nodes) = self {
            nodes.push(object)
        }
    }

    fn add_leaf(&mut self, object: TreeGeometry) {
        if let Self::Leafs(ref mut leafs) = self {
            leafs.push(object)
        };
    }
}

impl RtreeNode {
    pub fn new<T>(rectangle: BoundingRectangle, max_children: u8, 
                  obj_type: fn(Vec<T>) -> ChildrenType) -> RtreeNode {
        RtreeNode {
            id: generate_id(),
            children: obj_type(vec!()),
            mbr: rectangle,
            max_children: max_children,
            parent: None
        }
    }

    pub fn print(&self, prev: &str) -> () {
        println!("{}├──{}", prev, self);
        match &self.children {
            ChildrenType::InnerNodes(nodes) => {
                for node in nodes {
                    node.borrow().print(&format!("|  {}", prev))
                };
            },
            ChildrenType::Leafs(ref leafs) => {
                let next = format!("| {}", prev);
                for leaf in leafs {
                    println!("{}└──{}", next, leaf.borrow());
                }
            }
        }
    }

    pub fn insert(&mut self, geom: TreeGeometry) -> () {
        match &mut self.children {
            ChildrenType::InnerNodes(ref mut objs) => {
                let mut least_enl_vec: Vec<TreeNode> = vec!();
                for obj in objs {
                    least_enl_vec.push(obj.clone())
                }
                let least_enl = find_least_enlargement(
                    &mut least_enl_vec,
                    &geom.borrow().mbr()
                );
                let obj_to_enl = least_enl.0;
                obj_to_enl.borrow_mut().set_mbr(least_enl.1);
                obj_to_enl.borrow_mut().insert(geom);
            },
            ChildrenType::Leafs(ref mut objs) => {
                objs.push(geom);
            }
        }
    }
}

impl PartialEq for RtreeNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Display for RtreeNode {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_fmt(format_args!("Node {} with MBR {}", self.id, self.mbr))
    }
}

impl RtreeObject for RtreeNode {

    fn id(&self) -> &str {
        &self.id
    }

    fn mbr(&self) -> &BoundingRectangle {
        &self.mbr
    }

    fn set_mbr(&mut self, mbr: BoundingRectangle) -> () {
        self.mbr = mbr
    }

    fn set_parent(&mut self, node: &TreeNode) -> () {
        self.parent = Some(Rc::downgrade(node));
    }
}

impl RtreeSplit for RtreeNode {

    fn split(&mut self) -> () {
        let nodes = self.execute();
        self.children = nodes.0.children;
        self.mbr = nodes.0.mbr;

        match self.children {
            ChildrenType::InnerNodes(ref mut nodes) => {
                let self_ptr = self as *const RtreeNode;
                let self_rc = Weak::from_raw(self_ptr).upgrade();

                if let Some(ref ptr) = self_rc {
                    for node in nodes {
                        node.set_parent(ptr)
                    }
                }
            }
        }
    }

    fn execute(&mut self) -> (Rc<RtreeNode>, Rc<RtreeNode>) {
        let node_1: Rc<RtreeNode>;
        let node_2: Rc<RtreeNode>;
        match self.children {
            ChildrenType::InnerNodes(ref mut nodes) => {
                let picked = self.pick_seeds(nodes, ChildrenType::InnerNodes);
                node_1 = picked.0;
                node_2 = picked.1;
                self.distribute_nodes(&mut node_1, &mut node_2, nodes);
            },
            ChildrenType::Leafs(ref mut leafs) => {
                let picked = self.pick_seeds(leafs, ChildrenType::Leafs);
                node_1 = picked.0;
                node_2 = picked.1;
                self.distribute_leafs(&mut node_1, &mut node_2, leafs);
            }
        };
        (node_1, node_2)
    }

    fn pick_seeds<T: RtreeObject>(&mut self, objects: &mut Vec<Rc<T>>, 
                                  child_type: fn(Vec<Rc<T>>) -> ChildrenType)
                -> (Rc<RtreeNode>, Rc<RtreeNode>) {
        let mut max_area = NEGINF;
        let mut objs = MaybeUninit::<(usize, usize)>::uninit();
        
        for i in 0..objects.len() {
            let obj_1 = &objects[i];
            for j in i + 1..objects.len() {
                let obj_2 = &objects[j];
                let common_mbr = BoundingRectangle::common_mbr(
                    &vec!(&obj_1.mbr(), &obj_2.mbr())
                );
                if common_mbr.area > max_area {
                    max_area = common_mbr.area;
                    unsafe {
                        objs.as_mut_ptr().write((i, j))
                    }
                };
            }
        };

        let objs = unsafe {
            objs.assume_init()
        };

        let mut obj_1 = objects.remove(objs.0);
        let mut obj_2 = objects.remove(objs.1 - 1);

        let mut node_1 = RtreeNode::new(
            *obj_1.mbr(),
            self.max_children,
            child_type
        );
        let mut node_2 = RtreeNode::new(
            *obj_2.mbr(),
            self.max_children,
            child_type
        );

        node_1.children = child_type(vec!(obj_1));
        node_2.children = child_type(vec!(obj_2));

        (Rc::new(node_1), Rc::new(node_2))
    }

    fn distribute_nodes(
        &self,
        node_1: &mut Rc<RtreeNode>,
        node_2: &mut Rc<RtreeNode>,
        objects: &mut Vec<Rc<RtreeNode>>
    ) -> () {
        for i in 0..objects.len() {
            if self.validate_nodes_quantity(node_1, node_2, objects) {
                break
            } else {
                let insert_obj = self.pick_next_node(node_1, node_2, objects);
                let to_insert = find_least_enlargement(
                    &mut vec!(node_1, node_2),
                    &insert_obj.mbr
                );
                to_insert.0.mbr = to_insert.1;
                to_insert.0.children.add_node(insert_obj);
            }
        }
    }

    fn distribute_leafs(
        &self,
        node_1: &mut Rc<RtreeNode>,
        node_2: &mut Rc<RtreeNode>,
        objects: &mut Vec<Rc<RtreeGeometry>>
    ) -> () {
        for i in 0..objects.len() {
            if self.validate_leafs_quantity(node_1, node_2, objects) {
                break
            } else {
                let insert_obj = self.pick_next_leaf(node_1, node_2, objects);
                let to_insert = find_least_enlargement(
                    &mut vec!(node_1, node_2),
                    &insert_obj.mbr
                );
                to_insert.0.mbr = to_insert.1;
                to_insert.0.children.add_leaf(insert_obj);
            }
        }
    }

    fn pick_next_node(&self, node_1: &mut Rc<RtreeNode>, node_2: &mut Rc<RtreeNode>,
                          objects: &mut Vec<Rc<RtreeNode>>)
            -> Rc<RtreeNode> {
        let min_d = INF;
        let chosen: usize;

        for i in 0..objects.len() {
            let obj = objects[i];
            let area_1 = BoundingRectangle::common_mbr(
                &vec!(&node_1.mbr, &obj.mbr)
            ).area;
            let area_2 = BoundingRectangle::common_mbr(
                &vec!(&node_2.mbr, &obj.mbr)
            ).area;

            let d = area_1 - area_2;
            if d < min_d {
                min_d = d;
                chosen = i;
            };
        };
        objects.remove(chosen)
    }

    fn pick_next_leaf(&self, node_1: &mut Rc<RtreeNode>, node_2: &mut Rc<RtreeNode>,
                      objects: &mut Vec<Rc<RtreeGeometry>>) -> Rc<RtreeGeometry> {
        let min_d = INF;
        let chosen: usize;

        for i in 0..objects.len() {
            let obj = objects[i];
            let area_1 = BoundingRectangle::common_mbr(
                &vec!(&node_1.mbr, &obj.mbr)
            ).area;
            let area_2 = BoundingRectangle::common_mbr(
                &vec!(&node_2.mbr, &obj.mbr)
            ).area;

            let d = area_1 - area_2;
            if d < min_d {
                min_d = d;
                chosen = i;
            };
        };
        objects.remove(chosen)
    }

    fn pick_underfull<'a>(&'a self, node_1: &'a mut Rc<RtreeNode>, node_2: &'a mut Rc<RtreeNode>)
            -> Option<&'a mut Rc<RtreeNode>>  {
        let to_final_push: Option<&mut Rc<RtreeNode>> = None;
        let max_objs = node_1.max_children;
        let peak = (max_objs - max_objs / 2) + 1;

        if node_1.children.len() == peak as usize {
            to_final_push = Some(node_1);
        } else if node_2.children.len() == peak as usize {
            to_final_push = Some(node_2);
        };

        to_final_push
    }

    fn validate_nodes_quantity(
        &self,
        node_1: &mut Rc<RtreeNode>,
        node_2: &mut Rc<RtreeNode>,
        objects: &mut Vec<Rc<RtreeNode>>
    ) -> bool {

        let to_final_push = self.pick_underfull(node_1, node_2);
        match to_final_push {
            Some(value) => {
                for i in 0..objects.len() {
                    let obj = objects.remove(i);
                    let new_mbr = BoundingRectangle::common_mbr(
                        &vec!(obj.mbr(), &value.mbr)
                    );
                    value.children.add_node(obj) 
                }
                true
            },
            None => false
        }
    }

    fn validate_leafs_quantity(
        &self,
        node_1: &mut Rc<RtreeNode>,
        node_2: &mut Rc<RtreeNode>,
        objects: &mut Vec<Rc<RtreeGeometry>>
    ) -> bool {

        let to_final_push = self.pick_underfull(node_1, node_2);
        match to_final_push {
            Some(value) => {
                for i in 0..objects.len() {
                    let obj = objects.remove(i);
                    let new_mbr = BoundingRectangle::common_mbr(
                        &vec!(obj.mbr(), &value.mbr)
                    );
                    value.children.add_leaf(obj) 
                }
                true
            },
            None => false
        }
    }
}
