pub(crate) mod langs;
mod language_tsx;
mod language_typescript;

pub use langs::Langs;
pub use language_typescript::Typescript;

pub trait Lang {}
