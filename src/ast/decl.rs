use super::ExprKind;

#[derive(Debug, PartialEq)]
pub struct Decl {
    pub name: String,
    pub value: Box<ExprKind>,
}
