use std::collections::HashMap;
use super::TreeDump;

#[derive(Clone)]
pub struct TypeCollection {
    types: Vec<Type>
}

impl TypeCollection {
    pub fn from(types: Vec<Type>) -> TypeCollection {
        TypeCollection {
            types: types
        }
    }

    pub fn undef() -> TypeCollection {
        TypeCollection {
            types: Vec::new()
        }
    }

    pub fn is_undef(&self) -> bool {
        self.types.len() == 0
    }

    pub fn constrain(&mut self, other: &TypeCollection) {
        if self.types.len() == 0 {
            // We have an undefined type, so just grab the types that get constrained
            for t in other.types.iter() {
                self.types.push(t.clone());
            }
            return;
        }

        let mut new_types = Vec::with_capacity(self.types.len());
        for t in other.types.iter() {
            if self.types.contains(t) {
                new_types.push(t.clone());
            }
        }

        self.types.clear();
        self.types.append(&mut new_types);
    }
    
    pub fn collapse(&self) -> Option<Type> {
        if self.types.contains(&Type::Int) {
            Some(Type::Int)
        }else if self.types.contains(&Type::Float) {
            Some(Type::Float)
        }else if self.types.contains(&Type::Str) {
            Some(Type::Str)
        }else{
            None
        }
    }
}

impl std::fmt::Display for TypeCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.types.len() == 0 {
            write!(f, "undef")
        }else {
            for (i, elem) in self.types.iter().enumerate() {
                if i == 0 {
                    write!(f, "{}", elem)?;
                }else if i < self.types.len() - 1 {
                    write!(f, ", {}", elem)?;
                }else{
                    write!(f, " or {}", elem)?;
                }
            }

            Ok(())
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Str
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Str => write!(f, "string")
        }
    }
}

pub struct Scope {
    members: HashMap<String, TypeCollection>,
    parent_scope: Option<u32>,
    id: u32
}

pub struct ScopePool {
    scopes: HashMap<u32, Scope>,
    n_scopes: u32
}

impl ScopePool {
    pub fn new() -> ScopePool {
        ScopePool {
            scopes: HashMap::new(),
            n_scopes: 0
        }
    }

    fn get_member_loc(&self, scope_id: u32, var_name: &str) -> Option<u32> {
        let scope = self.scopes.get(&scope_id).expect("Expected a valid scope id in get_member_loc");
        
        if scope.members.contains_key(var_name) {
            Some(scope_id)
        }else{
            let parent_id = scope.parent_scope?;
            self.get_member_loc(parent_id, var_name)
        }
    }

    fn get_member_mut(&mut self, scope_id: u32, var_name: &str) -> Option<&mut TypeCollection> {
        let scope = self.scopes.get(&scope_id).expect("Expected a valid scope id in get_member_mut");
        
        if scope.members.contains_key(var_name) {
            let scope = self.scopes.get_mut(&scope_id).unwrap();
            scope.members.get_mut(var_name)
        }else{
            let parent_id = scope.parent_scope?;
            self.get_member_mut(parent_id, var_name)
        }
    }

    fn get_member(&self, scope_id: u32, var_name: &str) -> Option<&TypeCollection> {
        let scope = self.scopes.get(&scope_id).expect("Expected a valid scope id in get_member");
        
        if scope.members.contains_key(var_name) {
            scope.members.get(var_name)
        }else{
            let parent_id = scope.parent_scope?;
            self.get_member(parent_id, var_name)
        }
    }

    pub fn create_scope(&mut self) -> ScopeHandle {
        self.scopes.insert(
            self.n_scopes,
            Scope {
                members: HashMap::new(),
                parent_scope: None,
                id: self.n_scopes
            }
        );

        let handle = ScopeHandle { id: self.n_scopes };
        self.n_scopes += 1;
        handle
    }

    fn create_scope_with_parent(&mut self, parent_scope: u32) -> ScopeHandle {
        self.scopes.insert(
            self.n_scopes,
            Scope {
                members: HashMap::new(),
                parent_scope: Some(parent_scope),
                id: self.n_scopes
            }
        );

        let handle = ScopeHandle { id: self.n_scopes };
        self.n_scopes += 1;
        handle
    }
}

impl ScopePool {
    fn print_scope_with_indent(&self, scope: &Scope, indent: usize, indent_style: &str) {
        println!("{}Scope[{}]:", indent_style.repeat(indent), scope.id);
        for member in scope.members.iter() {
            println!("{}{}: {}", indent_style.repeat(indent + 1), member.0, member.1);
        }

        for sub_scope in self.scopes.values() {
            if let Some(parent) = sub_scope.parent_scope {
                if parent == scope.id {
                    self.print_scope_with_indent(sub_scope, indent + 1, indent_style);
                }
            }
        }
    }
}

impl TreeDump for ScopePool {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}ScopePool:", indent_style.repeat(indent));
        for scope in self.scopes.values() {
            if scope.parent_scope.is_none() {
                self.print_scope_with_indent(scope, indent + 1, indent_style);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct ScopeHandle {
    id: u32
}

impl ScopeHandle {
    pub fn create_subscope(&self, scope_pool: &mut ScopePool) -> ScopeHandle {
        scope_pool.create_scope_with_parent(self.id)
    }

    pub fn get_mut<'a>(&self, scope_pool: &'a mut ScopePool, var_name: &str) -> Option<&'a mut TypeCollection> {
        scope_pool.get_member_mut(self.id, var_name)
    }

    pub fn get<'a>(&self, scope_pool: &'a ScopePool, var_name: &str) -> Option<&'a TypeCollection> {
        scope_pool.get_member(self.id, var_name)
    }

    pub fn insert(&self, scope_pool: &mut ScopePool, var_name: &str, var_type: TypeCollection) -> Option<TypeCollection> {
        let scope = scope_pool.scopes.get_mut(&self.id).expect("ScopeHandle has an invalid ScopeID. Maybe you passed the wrong ScopePool");
        scope.members.insert(String::from(var_name), var_type)
    }
}

