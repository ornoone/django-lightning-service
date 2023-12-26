use std::rc::Rc;
use crate::entity::{DatabaseValue, Entity};
use crate::errors::EntityError;

enum Expression {
    Exact(ExactExpression),
}

type Attribute = String;

#[derive(Clone)]
struct ExactExpression {
    attribute: Attribute,
    value: DatabaseValue,
}

trait ExpressionTrait {
    fn match_entity(&self, entity: Rc<Entity>) -> Result<bool, EntityError>;

    /// return if *other* in included in the actual expression
    /// it make sens to verify if our current expression
    /// is not a superset of the given *other*
    fn contains(&self, other: &Expression) -> bool;
}

impl From<ExactExpression> for Expression {
    fn from(value: ExactExpression) -> Self {
        Expression::Exact(value)
    }
} 



impl ExpressionTrait for ExactExpression {
    fn contains(&self, other: &Expression) -> bool {
        if let Expression::Exact(other_eq) = other {
            self.attribute == other_eq.attribute && self.value == other_eq.value
        } else {
            false
        }
    }

    fn match_entity(&self, entity: Rc<Entity>) -> Result<bool, EntityError>{
        Ok(entity.get(&self.attribute[..])?.get_value() == &self.value)
    }
}

impl ExactExpression {
    fn new(attribute: Attribute, value: DatabaseValue) -> Self {
        ExactExpression {
            attribute,
            value
        }
    }
}

#[cfg(test)]
mod test  {
    use crate::entity::DatabaseValue;
    use crate::expression::{ExactExpression, Expression, ExpressionTrait};

    #[test]
    fn test_equal_expression_include() {
        let exact_expr1 = ExactExpression::new("name".to_string(), DatabaseValue::String("john".to_string()));
        let exact_expr2 = ExactExpression::new("name".to_string(), DatabaseValue::String("john".to_string()));
        let exact_expr3 = ExactExpression::new("name".to_string(), DatabaseValue::String("doe".to_string()));
        let exact_expr4 = ExactExpression::new("surname".to_string(), DatabaseValue::String("doe".to_string()));

        let expr1: Expression = exact_expr1.clone().into();
        let expr2: Expression = exact_expr2.clone().into();
        let expr3: Expression = exact_expr3.clone().into();
        let expr4: Expression = exact_expr4.clone().into();

        assert!(exact_expr1.contains(&expr1));
        assert!(exact_expr1.contains(&expr2));
        assert!(!exact_expr1.contains(&expr3));
        assert!(!exact_expr1.contains(&expr4));

        assert!(exact_expr2.contains(&expr1));
        assert!(exact_expr2.contains(&expr2));
        assert!(!exact_expr2.contains(&expr3));
        assert!(!exact_expr2.contains(&expr4));

        assert!(!exact_expr3.contains(&expr1));
        assert!(!exact_expr3.contains(&expr2));
        assert!(exact_expr3.contains(&expr3));
        assert!(!exact_expr3.contains(&expr4));

        assert!(!exact_expr4.contains(&expr1));
        assert!(!exact_expr4.contains(&expr2));
        assert!(!exact_expr4.contains(&expr3));
        assert!(exact_expr4.contains(&expr4));


    }
}