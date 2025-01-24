#![deny(missing_docs)]

use crate::cfg_dot::RenderableNode;
use crate::decompiler::ast::expr::ExprKind;
use crate::decompiler::ast::visitors::emit_context::{EmitContextBuilder, EmitVerbosity};
use crate::decompiler::ast::visitors::emitter::Gs2Emitter;
use crate::decompiler::ast::visitors::AstVisitor;
use crate::decompiler::ast::AstKind;
use crate::utils::GBF_YELLOW;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::slice::Iter;

/// Represents the type of control-flow region.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub enum RegionType {
    /// Simply moves on to the next region without control flow
    Linear,
    /// Control flow construct (e.g. for, while, if, switch, etc.)
    ControlFlow,
    /// A tail (e.g, return)
    Tail,
    /// A region that is inactive and removed from the graph from structure analysis
    Inactive,
}

/// Describes a region
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub struct RegionId {
    /// The index of the region
    pub index: usize,
}

impl RegionId {
    /// Create a new `RegionId`.
    ///
    /// # Arguments
    /// - `index`: The index of the region in the graph.
    ///
    /// # Returns
    /// - A new `RegionId` instance.
    ///
    /// Example
    /// ```
    /// use gbf_core::decompiler::region::RegionId;
    ///
    /// let block = RegionId::new(0);
    /// ```
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

/// Represents a region in the control-flow graph.
#[derive(Debug, Clone)]
pub struct Region {
    nodes: Vec<AstKind>,
    jump_expr: Option<ExprKind>,
    region_type: RegionType,
}

impl Region {
    /// Creates a new region with the specified type and initializes with no statements.
    ///
    /// # Arguments
    /// * `id` - The id of the region.
    pub fn new(region_type: RegionType) -> Self {
        Self {
            nodes: Vec::new(),
            jump_expr: None,
            region_type,
        }
    }

    /// Returns the type of the region.
    pub fn region_type(&self) -> &RegionType {
        &self.region_type
    }

    /// Adds a statement to the region.
    ///
    /// # Arguments
    /// * `node` - The AST node to add.
    pub fn push_node(&mut self, node: AstKind) {
        self.nodes.push(node);
    }

    /// Adds multiple statements to the region.
    ///
    /// # Arguments
    /// * `nodes` - The AST nodes to add.
    pub fn push_nodes(&mut self, nodes: Vec<AstKind>) {
        self.nodes.extend(nodes);
    }

    /// Gets the nodes in the region.
    ///
    /// # Return
    /// The nodes in the region.
    pub fn get_nodes(&self) -> &Vec<AstKind> {
        &self.nodes
    }

    /// Gets the region type.
    ///
    /// # Return
    /// The region type.
    pub fn get_region_type(&self) -> RegionType {
        self.region_type
    }

    /// Sets the type of the region.
    ///
    /// # Arguments
    /// * `region_type` - The new type of the region.
    pub fn set_region_type(&mut self, region_type: RegionType) {
        self.region_type = region_type;
    }

    /// Remove the jump expression from the region.
    pub fn remove_jump_expr(&mut self) {
        self.jump_expr = None;
    }

    /// Gets the jump expression.
    ///
    /// # Return
    /// The jump expression.
    pub fn get_jump_expr(&self) -> Option<&ExprKind> {
        self.jump_expr.as_ref()
    }

    /// Sets the jump expression.
    ///
    /// # Arguments
    /// * `jump_expr` - The new optional jump expression.
    pub fn set_jump_expr(&mut self, jump_expr: Option<ExprKind>) {
        self.jump_expr = jump_expr;
    }

    /// Returns an iterator over the statements in the region.
    ///
    /// # Return
    /// An iterator over the statements in the region.
    pub fn iter_nodes(&self) -> Iter<AstKind> {
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
                .include_ssa_versions(true)
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
    use crate::decompiler::ast::assignment::AssignmentNode;
    use crate::decompiler::ast::bin_op::{BinOpType, BinaryOperationNode};
    use crate::decompiler::ast::expr::ExprKind;
    use crate::decompiler::ast::identifier::IdentifierNode;
    use crate::decompiler::ast::literal::LiteralNode;

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

    fn create_statement(lhs: Box<AssignableKind>, rhs: Box<ExprKind>) -> AssignmentNode {
        AssignmentNode::new(lhs, rhs).unwrap()
    }

    #[test]
    fn test_region_creation_and_instruction_addition() {
        let mut region = Region::new(RegionType::Linear);

        assert_eq!(region.region_type(), &RegionType::Linear);
        assert_eq!(region.iter_nodes().count(), 0);

        let ast_node1 = create_statement(
            create_identifier("x"),
            create_addition(create_integer_literal(1), create_integer_literal(2)),
        );

        let ast_node2 = create_statement(
            create_identifier("y"),
            create_subtraction(create_integer_literal(3), create_integer_literal(4)),
        );

        region.push_node(ast_node1.clone().into());
        region.push_node(ast_node2.clone().into());

        let mut iter = region.iter_nodes();
        assert_eq!(iter.next(), Some(&ast_node1.clone().into()));
        assert_eq!(iter.next(), Some(&ast_node2.clone().into()));
    }

    #[test]
    fn test_region_into_iter() {
        let region = Region::new(RegionType::Linear);
        let mut iter = region.into_iter();
        assert_eq!(iter.next(), None);
    }
}
