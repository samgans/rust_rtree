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
