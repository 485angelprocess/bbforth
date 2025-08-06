use crate::types::{FloatOp, ForthVal, IntOp};
use crate::context::WorkspaceContext;

use std::rc::Rc;

pub fn binary_op(fi: IntOp, ff: FloatOp) -> Rc<dyn Fn(&mut WorkspaceContext) -> ForthVal>{
    Rc::new(move |ws|{
       let a = ws.pop().unwrap();
       let b = ws.pop().unwrap();
       a.operate(&b, fi, ff).unwrap()
   })
}