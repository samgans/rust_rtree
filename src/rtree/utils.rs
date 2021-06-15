use std::mem::MaybeUninit;
use std::rc::Rc;

use uuid::Uuid;

use crate::INF;
use crate::geometries::BoundingRectangle;
use crate::nodes::{RtreeNode, RtreeObject, TreeGeometry, TreeNode};

pub fn generate_id() -> String {
    Uuid::new_v4().to_hyphenated().to_string()
}

pub fn find_least_enlargement<'a>(
    list_nodes: &'a mut Vec<TreeNode>,
    mbr: &BoundingRectangle
) -> (TreeNode, BoundingRectangle) {

    let mut min_enlargement = INF;
    let mut min_mbr = MaybeUninit::<BoundingRectangle>::uninit();
    let mut chosen_node = MaybeUninit::<TreeNode>::uninit();

    for node in list_nodes {
        let node_val = node.borrow();
        let node_mbr = node_val.mbr();
        let enlarged = BoundingRectangle::common_mbr(
            &vec!(node_mbr, mbr)
        );
        let enlargement = enlarged.area - node_mbr.area;

        if enlargement < min_enlargement {
            min_enlargement = enlargement;
            unsafe {
                min_mbr.as_mut_ptr().write(enlarged);
                chosen_node.as_mut_ptr().write((*node).clone());
            }
        }
    };
    
    let min_mbr = unsafe { min_mbr.assume_init() };
    let chosen_node = unsafe { chosen_node.assume_init() };

    (chosen_node, min_mbr)   
}
