use super::{ TreeDump, SyntaxTreeNode, CodeLocation, Loc };

pub struct AssignmentNode {
    pub start: Loc,
    pub identifier: String,
    pub data: Box<SyntaxTreeNode>
}

impl CodeLocation for AssignmentNode {
    fn get_start(&self) -> Loc { self.start }
}

impl TreeDump for AssignmentNode {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}({}): Assignment of '{}' to", 
            indent_style.repeat(indent), self.start, self.identifier);
        self.data.print_with_indent(indent + 1, indent_style);
    }
}

impl SyntaxTreeNode for AssignmentNode {}
