use super::Variable;

#[derive(Debug, PartialEq)]
pub enum ExprKind {
    String(String),
    Variable(Variable),
}
