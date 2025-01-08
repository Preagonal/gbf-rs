#![deny(missing_docs)]

use crate::cfg_dot::RenderableNode;
use crate::decompiler::ast::visitors::emit_context::{EmitContextBuilder, EmitVerbosity};
use crate::decompiler::ast::visitors::emitter::Gs2Emitter;
use crate::decompiler::ast::visitors::AstVisitor;
use crate::decompiler::ast::AstKind;
use crate::utils::GBF_YELLOW;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::slice::Iter;

/// Represents the type of control-flow region.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegionType {
    /// Simply moves on to the next region without control flow
    Linear,
    /// Control flow construct (e.g. for, while, if, switch, etc.)
    ControlFlow,
    /// A tail (e.g, return)
    Tail,
}

/// Describes a region
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegionId {
    /// The index of the region
    pub index: usize,
    /// The region type
    pub region_type: RegionType,
}

impl RegionId {
    /// Create a new `BasicBlockId`.
    ///
    /// # Arguments
    /// - `index`: The index of the basic block in the function.
    ///
    /// # Returns
    /// - A new `BasicBlockId` instance.
    ///
    /// Example
    /// ```
    /// use gbf_core::decompiler::region::RegionId;
    /// use gbf_core::decompiler::region::RegionType;
    ///
    /// let block = RegionId::new(0, RegionType::Linear);
    /// ```
    pub fn new(index: usize, region_type: RegionType) -> Self {
        Self { index, region_type }
    }
}

/// Represents a region in the control-flow graph.
#[derive(Debug, Clone)]
pub struct Region {
    id: RegionId,
    nodes: Vec<AstKind>,
}

impl Region {
    /// Creates a new region with the specified type and initializes with no statements.
    ///
    /// # Arguments
    /// * `id` - The id of the region.
    pub fn new(id: RegionId) -> Self {
        Self {
            id,
            nodes: Vec::new(),
        }
    }

    /// Returns the type of the region.
    pub fn region_type(&self) -> &RegionType {
        &self.id.region_type
    }

    /// Adds a statement to the region.
    ///
    /// # Arguments
    ///
    /// * `node` - The AST node to add.
    pub fn push_instruction(&mut self, node: AstKind) {
        self.nodes.push(node);
    }

    /// Returns an iterator over the statements in the region.
    pub fn iter_statements(&self) -> Iter<AstKind> {
        self.nodes.iter()
    }
}

// === Other Implementations ===

/// Allows iterating over the statements in a region.
impl<'a> IntoIterator for &'a Region {
    type Item = &'a AstKind;
    type IntoIter = Iter<'a, AstKind>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

/// Allows iterating over the statements in a region (mutable).
impl<'a> IntoIterator for &'a mut Region {
    type Item = &'a mut AstKind;
    type IntoIter = std::slice::IterMut<'a, AstKind>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter_mut()
    }
}

impl RenderableNode for Region {
    /// Render the region's node representation for Graphviz with customizable padding.
    ///
    /// # Arguments
    /// * `padding` - The number of spaces to use for indentation.
    ///
    /// # Return
    /// The rendered node
    fn render_node(&self, padding: usize) -> String {
        let mut label = String::new();
        let indent = " ".repeat(padding);

        // Start the HTML-like table for Graphviz.
        writeln!(
            &mut label,
            r#"{indent}<TABLE BORDER="0" CELLBORDER="0" CELLSPACING="0" CELLPADDING="0">"#,
            indent = indent
        )
        .unwrap();

        // Render each statement as a table row with indentation.
        for node in &self.nodes {
            // Build a new EmitContext with debug
            let context = EmitContextBuilder::default()
                .verbosity(EmitVerbosity::Debug)
                .build();
            let mut emitter = Gs2Emitter::new(context);
            emitter.visit_node(node);
            let result = emitter.output();

            writeln!(
                &mut label,
                r##"{indent}    <TR>
{indent}        <TD ALIGN="LEFT"><FONT COLOR="{GBF_YELLOW}">{}</FONT></TD>
{indent}    </TR>"##,
                result,
                indent = indent
            )
            .unwrap();
        }

        // Close the HTML-like table.
        writeln!(&mut label, "{indent}</TABLE>", indent = indent).unwrap();

        label
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompiler::ast::assignable::AssignableKind;
    use crate::decompiler::ast::bin_op::{BinOpType, BinaryOperationNode};
    use crate::decompiler::ast::expr::ExprKind;
    use crate::decompiler::ast::identifier::IdentifierNode;
    use crate::decompiler::ast::literal::LiteralNode;
    use crate::decompiler::ast::statement::StatementNode;

    fn create_identifier(id: &str) -> Box<AssignableKind> {
        Box::new(AssignableKind::Identifier(IdentifierNode::new(
            id.to_string(),
        )))
    }

    fn create_integer_literal(value: i32) -> Box<ExprKind> {
        Box::new(ExprKind::Literal(LiteralNode::Number(value)))
    }

    fn create_addition(lhs: Box<ExprKind>, rhs: Box<ExprKind>) -> Box<ExprKind> {
        Box::new(ExprKind::BinOp(
            BinaryOperationNode::new(lhs, rhs, BinOpType::Add).unwrap(),
        ))
    }

    fn create_subtraction(lhs: Box<ExprKind>, rhs: Box<ExprKind>) -> Box<ExprKind> {
        Box::new(ExprKind::BinOp(
            BinaryOperationNode::new(lhs, rhs, BinOpType::Sub).unwrap(),
        ))
    }

    fn create_statement(lhs: Box<AssignableKind>, rhs: Box<ExprKind>) -> StatementNode {
        StatementNode::new(lhs, rhs).unwrap()
    }

    #[test]
    fn test_region_creation_and_instruction_addition() {
        let region_id = RegionId::new(0, RegionType::Linear);
        let mut region = Region::new(region_id);

        assert_eq!(region.region_type(), &RegionType::Linear);
        assert_eq!(region.iter_statements().count(), 0);

        let ast_node1 = create_statement(
            create_identifier("x"),
            create_addition(create_integer_literal(1), create_integer_literal(2)),
        );

        let ast_node2 = create_statement(
            create_identifier("y"),
            create_subtraction(create_integer_literal(3), create_integer_literal(4)),
        );

        region.push_instruction(AstKind::Statement(ast_node1.clone()));
        region.push_instruction(AstKind::Statement(ast_node2.clone()));

        let mut iter = region.iter_statements();
        assert_eq!(iter.next(), Some(&AstKind::Statement(ast_node1)));
        assert_eq!(iter.next(), Some(&AstKind::Statement(ast_node2)));
    }

    #[test]
    fn test_region_into_iter() {
        let region_id = RegionId::new(0, RegionType::Linear);
        let region = Region::new(region_id);
        let mut iter = region.into_iter();
        assert_eq!(iter.next(), None);
    }
}
