use super::{ CodeLocation, TreeDump, SyntaxTreeNode, Loc, ScopeHandle, ScopePool, TypeCollection, Type };
use super::super::tokenizer::LiteralType;

pub struct LiteralNode {
    pub start: Loc,
    pub literal: LiteralType
}

impl CodeLocation for LiteralNode {
    fn get_start(&self) -> Loc { self.start }
}

impl TreeDump for LiteralNode {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}({}): literal {}", indent_style.repeat(indent), self.start, self.literal);
    }   
}

impl SyntaxTreeNode for LiteralNode {
    fn get_possible_returns(&self, scope: ScopeHandle, scopes: &ScopePool) -> TypeCollection {
        use LiteralType::*;
        match self.literal {
            _String(_) => TypeCollection::from(vec![Type::Str]),
            Integer(_) => TypeCollection::from(vec![Type::Int, Type::Float]),
            Float(_) => TypeCollection::from(vec![Type::Float])
        }
    }
}
