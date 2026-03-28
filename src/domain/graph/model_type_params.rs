use serde::{Deserialize, Serialize};
use std::fmt;

/// Lifetime parameter (e.g., 'a, 'static)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Lifetime(String);

impl Lifetime {
    pub fn new(name: String) -> Result<Self, LifetimeError> {
        if name.is_empty() {
            return Err(LifetimeError::Empty);
        }
        if !name.starts_with('\'') {
            return Err(LifetimeError::MissingQuote { name });
        }
        if name.len() == 1 {
            return Err(LifetimeError::MissingName { name });
        }
        Ok(Self(name))
    }

    pub fn static_lifetime() -> Self {
        Self("'static".to_string())
    }

    pub fn anonymous() -> Self {
        Self("'_".to_string())
    }

    pub fn name(&self) -> &str {
        &self.0
    }

    pub fn is_static(&self) -> bool {
        self.0 == "'static"
    }

    pub fn is_anonymous(&self) -> bool {
        self.0 == "'_"
    }
}

impl fmt::Display for Lifetime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LifetimeError {
    #[error("Lifetime name cannot be empty")]
    Empty,
    #[error("Lifetime must start with single quote: {name}")]
    MissingQuote { name: String },
    #[error("Lifetime must have a name after quote: {name}")]
    MissingName { name: String },
}

/// Trait bound on a type parameter (e.g., T: Clone + Send)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitBound {
    pub trait_name: String,
    pub is_maybe: bool,  // ?Trait for relaxed bounds
}

impl TraitBound {
    pub fn new(trait_name: String) -> Self {
        Self {
            trait_name,
            is_maybe: false,
        }
    }

    pub fn maybe(trait_name: String) -> Self {
        Self {
            trait_name,
            is_maybe: true,
        }
    }
}

/// Lifetime bound on a type parameter (e.g., T: 'a)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifetimeBound {
    pub lifetime: Lifetime,
}

/// Generic constraint on a type parameter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenericConstraint {
    /// Trait bound: T: Clone
    Trait(TraitBound),
    /// Lifetime bound: T: 'a
    Lifetime(LifetimeBound),
    /// Higher-ranked trait bound: for<'a> F: Fn(&'a str)
    HigherRanked {
        lifetimes: Vec<Lifetime>,
        trait_bound: TraitBound,
    },
}

/// Type parameter with constraints (e.g., T: Clone + Send + 'static)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeParameter {
    pub name: String,
    pub constraints: Vec<GenericConstraint>,
    pub default_type: Option<String>,  // T = DefaultType
}

impl TypeParameter {
    pub fn new(name: String) -> Result<Self, TypeParameterError> {
        if name.is_empty() {
            return Err(TypeParameterError::EmptyName);
        }
        Ok(Self {
            name,
            constraints: Vec::new(),
            default_type: None,
        })
    }

    pub fn with_constraint(mut self, constraint: GenericConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn with_default(mut self, default_type: String) -> Self {
        self.default_type = Some(default_type);
        self
    }

    pub fn has_trait_bound(&self, trait_name: &str) -> bool {
        self.constraints.iter().any(|c| match c {
            GenericConstraint::Trait(tb) => tb.trait_name == trait_name,
            _ => false,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TypeParameterError {
    #[error("Type parameter name cannot be empty")]
    EmptyName,
}

/// Const generic parameter (e.g., N in [T; N])
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstParameter {
    pub name: String,
    pub const_type: String,  // Usually usize
}

impl ConstParameter {
    pub fn new(name: String, const_type: String) -> Result<Self, ConstParameterError> {
        if name.is_empty() {
            return Err(ConstParameterError::EmptyName);
        }
        if const_type.is_empty() {
            return Err(ConstParameterError::EmptyType);
        }
        Ok(Self { name, const_type })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConstParameterError {
    #[error("Const parameter name cannot be empty")]
    EmptyName,
    #[error("Const parameter type cannot be empty")]
    EmptyType,
}

/// Associated type in a trait (e.g., type Item = ...)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssociatedType {
    pub name: String,
    pub constraints: Vec<GenericConstraint>,
    pub default_type: Option<String>,
}

impl AssociatedType {
    pub fn new(name: String) -> Result<Self, AssociatedTypeError> {
        if name.is_empty() {
            return Err(AssociatedTypeError::EmptyName);
        }
        Ok(Self {
            name,
            constraints: Vec::new(),
            default_type: None,
        })
    }

    pub fn with_constraint(mut self, constraint: GenericConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn with_default(mut self, default_type: String) -> Self {
        self.default_type = Some(default_type);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AssociatedTypeError {
    #[error("Associated type name cannot be empty")]
    EmptyName,
}

/// Trait object type (e.g., dyn Trait + Send)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitObject {
    pub primary_trait: String,
    pub additional_bounds: Vec<TraitBound>,
    pub lifetime: Option<Lifetime>,
}

impl TraitObject {
    pub fn new(primary_trait: String) -> Result<Self, TraitObjectError> {
        if primary_trait.is_empty() {
            return Err(TraitObjectError::EmptyTrait);
        }
        Ok(Self {
            primary_trait,
            additional_bounds: Vec::new(),
            lifetime: None,
        })
    }

    pub fn with_bound(mut self, bound: TraitBound) -> Self {
        self.additional_bounds.push(bound);
        self
    }

    pub fn with_lifetime(mut self, lifetime: Lifetime) -> Self {
        self.lifetime = Some(lifetime);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TraitObjectError {
    #[error("Trait object must have a primary trait")]
    EmptyTrait,
}

/// Closure type (Fn, FnMut, FnOnce)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClosureKind {
    Fn,
    FnMut,
    FnOnce,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosureType {
    pub kind: ClosureKind,
    pub input_types: Vec<String>,
    pub output_type: Option<String>,
}

impl ClosureType {
    pub fn new(kind: ClosureKind, input_types: Vec<String>) -> Self {
        Self {
            kind,
            input_types,
            output_type: None,
        }
    }

    pub fn with_output(mut self, output_type: String) -> Self {
        self.output_type = Some(output_type);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifetime_new() {
        assert!(Lifetime::new("'a".to_string()).is_ok());
        assert!(Lifetime::new("'static".to_string()).is_ok());
        assert!(Lifetime::new("'_".to_string()).is_ok());
        assert!(Lifetime::new("".to_string()).is_err());
        assert!(Lifetime::new("a".to_string()).is_err());
        assert!(Lifetime::new("'".to_string()).is_err());
    }

    #[test]
    fn test_lifetime_static() {
        let lt = Lifetime::static_lifetime();
        assert!(lt.is_static());
        assert_eq!(lt.name(), "'static");
    }

    #[test]
    fn test_lifetime_anonymous() {
        let lt = Lifetime::anonymous();
        assert!(lt.is_anonymous());
        assert_eq!(lt.name(), "'_");
    }

    #[test]
    fn test_type_parameter_new() {
        let tp = TypeParameter::new("T".to_string()).unwrap();
        assert_eq!(tp.name, "T");
        assert!(tp.constraints.is_empty());
        assert!(tp.default_type.is_none());
    }

    #[test]
    fn test_type_parameter_with_constraint() {
        let tb = TraitBound::new("Clone".to_string());
        let tp = TypeParameter::new("T".to_string())
            .unwrap()
            .with_constraint(GenericConstraint::Trait(tb));
        
        assert!(tp.has_trait_bound("Clone"));
        assert!(!tp.has_trait_bound("Send"));
    }

    #[test]
    fn test_type_parameter_with_default() {
        let tp = TypeParameter::new("T".to_string())
            .unwrap()
            .with_default("String".to_string());
        
        assert_eq!(tp.default_type, Some("String".to_string()));
    }

    #[test]
    fn test_const_parameter_new() {
        let cp = ConstParameter::new("N".to_string(), "usize".to_string()).unwrap();
        assert_eq!(cp.name, "N");
        assert_eq!(cp.const_type, "usize");
    }

    #[test]
    fn test_const_parameter_invalid() {
        assert!(ConstParameter::new("".to_string(), "usize".to_string()).is_err());
        assert!(ConstParameter::new("N".to_string(), "".to_string()).is_err());
    }

    #[test]
    fn test_associated_type_new() {
        let at = AssociatedType::new("Item".to_string()).unwrap();
        assert_eq!(at.name, "Item");
        assert!(at.constraints.is_empty());
        assert!(at.default_type.is_none());
    }

    #[test]
    fn test_trait_object_new() {
        let to = TraitObject::new("Iterator".to_string()).unwrap();
        assert_eq!(to.primary_trait, "Iterator");
        assert!(to.additional_bounds.is_empty());
        assert!(to.lifetime.is_none());
    }

    #[test]
    fn test_trait_object_with_bounds() {
        let to = TraitObject::new("Iterator".to_string())
            .unwrap()
            .with_bound(TraitBound::new("Send".to_string()))
            .with_lifetime(Lifetime::static_lifetime());
        
        assert_eq!(to.additional_bounds.len(), 1);
        assert!(to.lifetime.is_some());
    }

    #[test]
    fn test_closure_type() {
        let ct = ClosureType::new(
            ClosureKind::Fn,
            vec!["i32".to_string(), "i32".to_string()],
        ).with_output("i32".to_string());
        
        assert!(matches!(ct.kind, ClosureKind::Fn));
        assert_eq!(ct.input_types.len(), 2);
        assert_eq!(ct.output_type, Some("i32".to_string()));
    }

    #[test]
    fn test_trait_bound_maybe() {
        let tb = TraitBound::maybe("Sized".to_string());
        assert!(tb.is_maybe);
        assert_eq!(tb.trait_name, "Sized");
    }

    #[test]
    fn test_higher_ranked_trait_bound() {
        let hrtb = GenericConstraint::HigherRanked {
            lifetimes: vec![Lifetime::new("'a".to_string()).unwrap()],
            trait_bound: TraitBound::new("Fn".to_string()),
        };
        
        match hrtb {
            GenericConstraint::HigherRanked { lifetimes, .. } => {
                assert_eq!(lifetimes.len(), 1);
            }
            _ => panic!("Expected HigherRanked variant"),
        }
    }
}
