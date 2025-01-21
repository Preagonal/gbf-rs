#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{ast_vec::AstVec, block::BlockNode, expr::ExprKind, AstKind, AstVisitable};

/// Represents a metadata node in the AST
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(AstKind::ControlFlow)]
pub struct ControlFlowNode {
    name: String,
    condition: Option<ExprKind>,
    body: BlockNode,
}

impl ControlFlowNode {
    /// Creates a new `ControlFlowNode` with the given name, condition, and body.
    ///
    /// # Arguments
    /// - `name`: The name of the control flow node.
    /// - `condition`: The condition of the control flow node.
    /// - `body`: The body of the control flow node.
    ///
    /// # Returns
    /// A new `ControlFlowNode`.
    pub fn new<N, E, V>(name: N, condition: Option<E>, body: V) -> Self
    where
        N: Into<String>,
        E: Into<ExprKind>,
        V: Into<AstVec<AstKind>>,
    {
        Self {
            name: name.into(),
            condition: condition.map(|e| e.into()),
            body: BlockNode::new(body),
        }
    }

    /// Returns the condition of the ControlFlowNode.
    pub fn condition(&self) -> &Option<ExprKind> {
        &self.condition
    }

    /// Returns the body of the ControlFlowNode.
    pub fn body(&self) -> &BlockNode {
        &self.body
    }

    /// Returns the name of the ControlFlowNode .
    pub fn name(&self) -> &String {
        &self.name
    }
}

// == Other implementations for literal ==
impl AstVisitable for ControlFlowNode {
    fn accept(&self, visitor: &mut dyn super::visitors::AstVisitor) {
        visitor.visit_control_flow(self);
    }
}

impl PartialEq for ControlFlowNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.condition == other.condition && self.body == other.body
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompiler::ast::{
        bin_op::BinOpType, emit, new_bin_op, new_fn, new_id, new_member_access, new_num,
        new_return, new_str, AstNodeError,
    };

    #[test]
    fn test_control_flow_node() -> Result<(), AstNodeError> {
        /* if (foo.bar == "baz")  { return 1; } */
        let condition = new_member_access(new_id("foo"), new_id("bar"))?;
        let condition = new_bin_op(condition, new_str("baz"), BinOpType::Equal)?;
        let body = BlockNode::new(vec![new_return(new_num(1))]);
        let control_flow = ControlFlowNode::new("if", Some(condition), vec![body]);
        assert_eq!(control_flow.name(), "if");
        assert!(control_flow.condition().is_some());
        assert_eq!(control_flow.body().instructions.len(), 1);
        Ok(())
    }

    #[test]
    fn test_control_flow_node_emit() -> Result<(), AstNodeError> {
        /* if (foo.bar == "baz")  { return 1; } */
        let condition = new_member_access(new_id("foo"), new_id("bar"))?;
        let condition = new_bin_op(condition, new_str("baz"), BinOpType::Equal)?;
        let body = vec![new_return(new_num(1))];
        let control_flow = ControlFlowNode::new("if", Some(condition), body);
        let function = new_fn(
            Some("onCreated".to_string()),
            vec![new_id("test")],
            vec![control_flow],
        );
        let output = emit(function);
        assert_eq!(output, "function onCreated(test)\n{\n    if (foo.bar == \"baz\") \n    {\n        return 1;\n    }\n}");
        Ok(())
    }

    #[test]
    fn test_control_flow_else_emit() -> Result<(), AstNodeError> {
        /* if (foo.bar == "baz")  { return 1; } else { return 2; } */
        let condition = new_member_access(new_id("foo"), new_id("bar"))?;
        let condition = new_bin_op(condition, new_str("baz"), BinOpType::Equal)?;
        let body = vec![new_return(new_num(1))];
        let else_body = vec![new_return(new_num(2))];
        let control_flow = ControlFlowNode::new("if", Some(condition), body);
        let else_control_flow = ControlFlowNode::new("else", None::<ExprKind>, else_body);
        let function = new_fn(
            Some("onCreated".to_string()),
            vec![new_id("test")],
            vec![control_flow, else_control_flow],
        );
        let output = emit(function);
        assert_eq!(output, "function onCreated(test)\n{\n    if (foo.bar == \"baz\") \n    {\n        return 1;\n    }\n    else\n    {\n        return 2;\n    }\n}");
        Ok(())
    }
}
