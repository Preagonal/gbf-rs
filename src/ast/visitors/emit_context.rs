#![deny(missing_docs)]

use thiserror::Error;

/// Represents an error that occurred while converting an AST node.
#[derive(Debug, Error)]
pub enum EmitError {}

/// Represents the verbosity mode in which the AST should be emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitVerbosity {
    /// Emit the AST in a format that is readable by humans.
    Pretty,
    /// Emit the AST in a format that is readable by the parser. This format ensures that no comments or extra whitespace is emitted.
    Minified,
    /// Debug mode, which emits the AST in a format that is useful for debugging.
    Debug,
}

/// Represents the formatting indentation style for blocks of code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentStyle {
    /// Allman style indentation.
    Allman,
    /// K&R style indentation.
    KAndR,
}

/// Contains the emitting context for the AST.
#[derive(Debug, Clone, Copy)]
pub struct EmitContext {
    /// The current indentation level.
    pub indent: usize,
    /// The number of spaces to indent by.
    pub indent_step: usize,
    /// Whether to format numbers in hexadecimal.
    pub format_number_hex: bool,
    /// The mode in which to emit the AST.
    pub verbosity: EmitVerbosity,
    /// The style of indentation to use.
    pub indent_style: IndentStyle,
    /// The root of the expression tree.
    pub expr_root: bool,
}

impl EmitContext {
    /// Allow temporarily changing the EmitContext for a block of code.
    ///
    /// # Arguments
    /// - `f` - The function to call with the new EmitContext.
    ///
    /// # Returns
    /// The EmitContext after the function has been called.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::ast::visitors::emit_context::EmitContext;
    ///
    /// let mut context = EmitContext::default();
    /// let body_context = context.scoped(|ctx| ctx.with_indent());
    /// ```
    pub fn scoped<F>(&mut self, action: F) -> Self
    where
        F: FnOnce(&mut Self) -> EmitContext,
    {
        action(self)
    }

    /// Returns a new EmitContext with the indent increased by the indent step.
    ///
    /// # Returns
    /// A new EmitContext with the indent increased by the indent step.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::ast::visitors::emit_context::EmitContext;
    ///
    /// let mut context = EmitContext::default();
    /// let body_context = context.with_indent();
    /// ```
    pub fn with_indent(&self) -> EmitContext {
        let mut new_context = *self;
        new_context.indent += self.indent_step;
        new_context
    }

    /// Returns a new EmitContext with expr_root set to the given value.
    ///
    /// # Arguments
    /// - `expr_root` - The value to set expr_root to.
    ///
    /// # Returns
    /// A new EmitContext with expr_root set to the given value.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::ast::visitors::emit_context::EmitContext;
    ///
    /// let mut context = EmitContext::default();
    /// let body_context = context.with_expr_root(true);
    /// ```
    pub fn with_expr_root(&self, expr_root: bool) -> EmitContext {
        let mut new_context = *self;
        new_context.expr_root = expr_root;
        new_context
    }

    /// Creates a builder for `EmitContext`.
    ///
    /// # Returns
    /// A new instance of `EmitContextBuilder`.
    pub fn builder() -> EmitContextBuilder {
        EmitContextBuilder::default()
    }
}

/// Builder for `EmitContext` to provide a fluent API for customization.
#[derive(Debug, Clone)]
pub struct EmitContextBuilder {
    indent: usize,
    indent_step: usize,
    format_number_hex: bool,
    verbosity: EmitVerbosity,
    indent_style: IndentStyle,
    expr_root: bool,
}

impl EmitContextBuilder {
    /// Sets the initial indentation level.
    pub fn indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    /// Sets the number of spaces per indentation step.
    pub fn indent_step(mut self, indent_step: usize) -> Self {
        self.indent_step = indent_step;
        self
    }

    /// Configures whether to format numbers in hexadecimal.
    pub fn format_number_hex(mut self, format_number_hex: bool) -> Self {
        self.format_number_hex = format_number_hex;
        self
    }

    /// Sets the verbosity mode.
    pub fn verbosity(mut self, verbosity: EmitVerbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Sets the indentation style.
    pub fn indent_style(mut self, indent_style: IndentStyle) -> Self {
        self.indent_style = indent_style;
        self
    }

    /// Sets the `expr_root` flag.
    pub fn expr_root(mut self, expr_root: bool) -> Self {
        self.expr_root = expr_root;
        self
    }

    /// Builds the `EmitContext` with the specified parameters.
    pub fn build(self) -> EmitContext {
        EmitContext {
            indent: self.indent,
            indent_step: self.indent_step,
            format_number_hex: self.format_number_hex,
            verbosity: self.verbosity,
            indent_style: self.indent_style,
            expr_root: self.expr_root,
        }
    }
}

// == Other Implementations for EmitContext ==

impl Default for EmitContextBuilder {
    fn default() -> Self {
        Self {
            indent: 0,
            indent_step: 4,
            format_number_hex: false,
            verbosity: EmitVerbosity::Pretty,
            indent_style: IndentStyle::Allman,
            expr_root: true,
        }
    }
}

impl Default for EmitContext {
    fn default() -> Self {
        EmitContextBuilder::default().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_context_with_indent() {
        let context = EmitContext::default();
        let new_context = context.with_indent();
        assert_eq!(new_context.indent, 4);
    }

    #[test]
    fn test_emit_context_scoped() {
        let mut context = EmitContext::default();
        let body_context = context.scoped(|ctx| ctx.with_indent());
        assert_eq!(body_context.indent, 4);
    }

    #[test]
    fn test_builder_default_values() {
        let builder = EmitContext::builder();
        let context = builder.build();
        assert_eq!(context.indent, 0);
        assert_eq!(context.indent_step, 4);
        assert!(!context.format_number_hex);
        assert_eq!(context.verbosity, EmitVerbosity::Pretty);
        assert_eq!(context.indent_style, IndentStyle::Allman);
    }

    #[test]
    fn test_builder_custom_values() {
        let context = EmitContext::builder()
            .indent(2)
            .indent_step(8)
            .format_number_hex(true)
            .verbosity(EmitVerbosity::Debug)
            .indent_style(IndentStyle::KAndR)
            .build();
        assert_eq!(context.indent, 2);
        assert_eq!(context.indent_step, 8);
        assert!(context.format_number_hex);
        assert_eq!(context.verbosity, EmitVerbosity::Debug);
        assert_eq!(context.indent_style, IndentStyle::KAndR);
    }
}
