use crate::types::{FloatOp, ForthVal, IntOp};
use crate::context::WorkspaceContext;

pub fn binary_op(fi: IntOp, ff: FloatOp) -> Box<dyn Fn(&mut WorkspaceContext) -> ForthVal>{
    Box::new(move |ws|{
       let a = ws.pop().unwrap();
       let b = ws.pop().unwrap();
       a.operate(&b, fi, ff).unwrap()
   })
}