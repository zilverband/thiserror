#![allow(clippy::needless_late_init)]

use std::fmt::{self, Debug, Display};
use thiserror::Error;

pub struct NoFormat;

#[derive(Debug)]
pub struct DebugOnly;

pub struct DisplayOnly;

impl Display for DisplayOnly {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("display only")
    }
}

#[derive(Debug)]
pub struct DebugAndDisplay;

impl Display for DebugAndDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("debug and display")
    }
}

// Should expand to:
//
//     impl<E> Display for EnumDebugField<E>
//     where
//         E: Debug;
//
//     impl<E> Error for EnumDebugField<E>
//     where
//         Self: Debug + Display;
//
#[derive(Error, Debug)]
pub enum EnumDebugGeneric<E> {
    #[error("{0:?}")]
    FatalError(E),
}

// Should expand to:
//
//     impl<E> Display for EnumFromGeneric<E>;
//
//     impl<E> Error for EnumFromGeneric<E>
//     where
//         EnumDebugGeneric<E>: Error + 'static,
//         Self: Debug + Display;
//
#[derive(Error, Debug)]
pub enum EnumFromGeneric<E> {
    #[error("enum from generic")]
    Source(#[from] EnumDebugGeneric<E>),
}

// Should expand to:
//
//     impl<E> std::fmt::Display for EnumSourceGeneric<E>;
//
//     impl<E> std::error::Error for EnumSourceGeneric<E>
//     where
//         E: std::error::Error + 'static,
//         Self: std::fmt::Debug + std::fmt::Display;
//
#[derive(Error, Debug)]
pub enum EnumSourceGeneric<E> {
    #[error("enum with a generic source")]
    Source(#[source] E),
}

#[test]
fn test_enum_source_generic() {
    #[derive(Debug)]
    struct SourceError {
        message: &'static str,
    }

    impl Display for SourceError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.message)
        }
    }

    impl std::error::Error for SourceError {}

    let err_with_generic_source = EnumSourceGeneric::Source(SourceError {
        message: "Source error",
    });

    assert!(match &err_with_generic_source {
        EnumSourceGeneric::Source(source_err) => source_err.message == "Source error",
    });

    let err: &dyn std::error::Error = &err_with_generic_source;

    assert!(match err.source() {
        Some(err_source) => err_source.to_string() == "Source error",
        None => false,
    });
}

// Should expand to:
//
//     impl<E> std::fmt::Display for EnumSourceGenericErrAsRef<E>;
//
//     impl<E> std::error::Error for EnumSourceGenericErrAsRef<E>
//     where
//         E: AsRef<dyn std::error::Error + 'static>,
//         Self: std::fmt::Debug + std::fmt::Display;
//
#[derive(Error, Debug)]
#[thiserror(generics_err_as_ref)]
pub enum EnumSourceGenericErrAsRef<E> {
    #[error("enum with a generic source")]
    Source(#[source] E),
}

#[test]
fn test_enum_source_generic_err_as_ref() {
    #[derive(Debug)]
    struct SourceError {
        inner: EnumDebugGeneric<u64>,
    }

    impl AsRef<dyn std::error::Error + 'static> for SourceError {
        fn as_ref(&self) -> &(dyn std::error::Error + 'static) {
            &self.inner
        }
    }

    let err_with_generic_source = EnumSourceGenericErrAsRef::Source(SourceError {
        inner: EnumDebugGeneric::FatalError(100),
    });

    assert!(match &err_with_generic_source {
        EnumSourceGenericErrAsRef::Source(source_err) => source_err.inner.to_string() == "100",
    });

    let err: &dyn std::error::Error = &err_with_generic_source;

    assert!(match err.source() {
        Some(err_source) => err_source.to_string() == "100",
        None => false,
    });
}

// Should expand to:
//
//     impl<HasDisplay, HasDebug, HasNeither> Display
//         for EnumCompound<HasDisplay, HasDebug, HasNeither>
//     where
//         HasDisplay: Display,
//         HasDebug: Debug;
//
//     impl<HasDisplay, HasDebug, HasNeither> Error
//         for EnumCompound<HasDisplay, HasDebug, HasNeither>
//     where
//         Self: Debug + Display;
//
#[derive(Error)]
pub enum EnumCompound<HasDisplay, HasDebug, HasNeither> {
    #[error("{0} {1:?}")]
    DisplayDebug(HasDisplay, HasDebug),
    #[error("{0}")]
    Display(HasDisplay, HasNeither),
    #[error("{1:?}")]
    Debug(HasNeither, HasDebug),
}

impl<HasDisplay, HasDebug, HasNeither> Debug for EnumCompound<HasDisplay, HasDebug, HasNeither> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("EnumCompound")
    }
}

#[test]
fn test_display_enum_compound() {
    let mut instance: EnumCompound<DisplayOnly, DebugOnly, NoFormat>;

    instance = EnumCompound::DisplayDebug(DisplayOnly, DebugOnly);
    assert_eq!(format!("{}", instance), "display only DebugOnly");

    instance = EnumCompound::Display(DisplayOnly, NoFormat);
    assert_eq!(format!("{}", instance), "display only");

    instance = EnumCompound::Debug(NoFormat, DebugOnly);
    assert_eq!(format!("{}", instance), "DebugOnly");
}

// Should expand to:
//
//     impl<E> Display for EnumTransparentGeneric<E>
//     where
//         E: Display;
//
//     impl<E> Error for EnumTransparentGeneric<E>
//     where
//         E: Error,
//         Self: Debug + Display;
//
#[derive(Error, Debug)]
pub enum EnumTransparentGeneric<E> {
    #[error(transparent)]
    Other(E),
}

// Should expand to:
//
//     impl<E> Display for StructDebugGeneric<E>
//     where
//         E: Debug;
//
//     impl<E> Error for StructDebugGeneric<E>
//     where
//         Self: Debug + Display;
//
#[derive(Error, Debug)]
#[error("{underlying:?}")]
pub struct StructDebugGeneric<E> {
    pub underlying: E,
}

// Should expand to:
//
//     impl<E> Error for StructFromGeneric<E>
//     where
//         StructDebugGeneric<E>: Error + 'static,
//         Self: Debug + Display;
//
#[derive(Error, Debug)]
pub struct StructFromGeneric<E> {
    #[from]
    pub source: StructDebugGeneric<E>,
}

// Should expand to:
//
//     impl<E> Display for StructTransparentGeneric<E>
//     where
//         E: Display;
//
//     impl<E> Error for StructTransparentGeneric<E>
//     where
//         E: Error,
//         Self: Debug + Display;
//
#[derive(Error, Debug)]
#[error(transparent)]
pub struct StructTransparentGeneric<E>(E);
