//! Unbound declarations.

use grammar::v1::Rule;
use pest::iterators::Pair;
use wdl_grammar as grammar;
use wdl_macros::check_node;

use crate::v1::document::declaration::r#type;
use crate::v1::document::declaration::r#type::Type;
use crate::v1::document::identifier::singular;
use crate::v1::document::identifier::singular::Identifier;

pub mod builder;

pub use builder::Builder;

/// An error related to a [`Declaration`].
#[derive(Debug)]
pub enum Error {
    /// A builder error.
    Builder(builder::Error),

    /// An identifier error.
    Identifier(singular::Error),

    /// A WDL type error.
    Type(r#type::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Builder(err) => write!(f, "builder error: {err}"),
            Error::Identifier(err) => write!(f, "identifier error: {err}"),
            Error::Type(err) => write!(f, "wdl type error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

/// An unbound declaration.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Declaration {
    /// The name.
    name: Identifier,

    /// The WDL type.
    r#type: Type,
}

impl Declaration {
    /// Gets the name of the [`Declaration`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use ast::v1::document::declaration::r#type::Kind;
    /// use ast::v1::document::declaration::unbound::Builder;
    /// use ast::v1::document::declaration::Type;
    /// use ast::v1::document::identifier::singular::Identifier;
    /// use wdl_ast as ast;
    ///
    /// let declaration = Builder::default()
    ///     .name(Identifier::try_from("hello_world")?)?
    ///     .r#type(Type::new(Kind::Boolean, false))?
    ///     .try_build()?;
    ///
    /// assert_eq!(declaration.name().as_str(), "hello_world");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    /// Gets the WDL [type](Type) of the [`Declaration`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use ast::v1::document::declaration::r#type::Kind;
    /// use ast::v1::document::declaration::unbound::Builder;
    /// use ast::v1::document::declaration::Type;
    /// use ast::v1::document::identifier::singular::Identifier;
    /// use wdl_ast as ast;
    ///
    /// let declaration = Builder::default()
    ///     .name(Identifier::try_from("hello_world")?)?
    ///     .r#type(Type::new(Kind::Boolean, false))?
    ///     .try_build()?;
    ///
    /// assert!(matches!(declaration.r#type().kind(), &Kind::Boolean));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn r#type(&self) -> &Type {
        &self.r#type
    }
}

impl TryFrom<Pair<'_, grammar::v1::Rule>> for Declaration {
    type Error = Error;

    fn try_from(node: Pair<'_, grammar::v1::Rule>) -> Result<Self, Self::Error> {
        check_node!(node, unbound_declaration);
        let mut builder = Builder::default();

        for node in node.into_inner() {
            match node.as_rule() {
                Rule::wdl_type => {
                    let r#type = Type::try_from(node).map_err(Error::Type)?;
                    builder = builder.r#type(r#type).map_err(Error::Builder)?;
                }
                Rule::unbound_declaration_name => {
                    let name = Identifier::try_from(node.as_str().to_owned())
                        .map_err(Error::Identifier)?;
                    builder = builder.name(name).map_err(Error::Builder)?;
                }
                Rule::WHITESPACE => {}
                Rule::COMMENT => {}
                rule => unreachable!("unbound declaration should not contain {:?}", rule),
            }
        }

        builder.try_build().map_err(Error::Builder)
    }
}

#[cfg(test)]
mod tests {
    use wdl_macros::test::create_invalid_node_test;
    use wdl_macros::test::valid_node;

    use super::*;
    use crate::v1::document::declaration::r#type::Kind;

    #[test]
    fn it_parses_from_a_supported_node_type() {
        let declaration = valid_node!("String? hello", unbound_declaration, Declaration);
        assert_eq!(declaration.r#type().kind(), &Kind::String);
        assert!(declaration.r#type().optional());
        assert_eq!(declaration.name().as_str(), "hello");
    }

    create_invalid_node_test!(
        "version 1.1\n\ntask hello { command <<<>>> }",
        document,
        unbound_declaration,
        Declaration,
        it_fails_to_parse_from_an_unsupported_node_type
    );
}
