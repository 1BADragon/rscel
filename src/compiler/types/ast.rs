pub enum AstType {}

pub enum DataType {
    Int,
    UInt,
    Double,
    Bool,
    String,
    Bytes,
    List,
    Map,
    NullType,
    Message,
    Type,
}

pub enum AstChild {
    IntLiteral(i64),
    UIntLiteral(u64),
    DoubleLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),
    BytesLiteral(Vec<u8>),
    NullLiteral,

    Ast(Box<Ast>),
}

pub struct Ast {
    ast_type: AstType,
    data_type: DataType,

    children: Vec<AstChild>,
}

impl Ast {
    pub fn new(ast_type: AstType, data_type: DataType, children: Vec<AstChild>) -> Self {
        Self {
            ast_type,
            data_type,
            children,
        }
    }

    pub fn ast_type<'a>(&'a self) -> &'a AstType {
        &self.ast_type
    }

    pub fn data_type<'a>(&'a self) -> &'a DataType {
        &self.data_type
    }

    pub fn children<'a>(&'a self) -> &'a [AstChild] {
        &self.children
    }
}
