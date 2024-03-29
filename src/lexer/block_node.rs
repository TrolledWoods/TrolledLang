use super::{ SyntaxTreeNode, TreeDump, CodeLocation, Loc, ScopeHandle, ScopePool, TypeCollection };

pub struct BlockNode {
    pub start: Loc,
    pub scope: ScopeHandle,
    pub contents: Vec<Box<SyntaxTreeNode>>,
    pub _return: Option<Box<SyntaxTreeNode>>
}

impl CodeLocation for BlockNode {
    fn get_start(&self) -> Loc { self.start }
}

impl TreeDump for BlockNode {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}({}): Block", indent_style.repeat(indent), self.start);
        if self.contents.len() > 0 {
            println!("{}Contents:", indent_style.repeat(indent + 1));
            for content in &self.contents {
                content.print_with_indent(indent + 2, indent_style);
            }
        }
        if let Some(node) = &self._return {
            println!("{}Returns:", indent_style.repeat(indent + 1));
            node.print_with_indent(indent + 2, indent_style);
        }
    }
}

impl SyntaxTreeNode for BlockNode {
    fn get_possible_returns(&self, scope: ScopeHandle, scopes: &ScopePool) -> TypeCollection {
        if let Some(node) = &self._return {
            node.get_possible_returns(self.scope, scopes)
        }else {
            TypeCollection::undef()
        }
    }
}