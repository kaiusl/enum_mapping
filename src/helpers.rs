/// Wrapper to simplify combining multiple errors into one
pub(crate) struct MultiError {
    pub(crate) inner: syn::Result<()>,
}

impl MultiError {
    pub(crate) fn new() -> Self {
        Self { inner: Ok(()) }
    }

    pub(crate) fn update(&mut self, new_err: syn::Error) {
        if let Err(ref mut e) = self.inner {
            e.combine(new_err);
        } else {
            self.inner = Err(new_err);
        }
    }
}

/// Internal error type used to create compile errors
// pub(crate) enum Error {
//     ArgSetTwice {
//         arg: &'static str,
//         span: proc_macro2::Span,
//     },
//     ArgNotSet {
//         arg: &'static str,
//         span: proc_macro2::Span,
//     },
//     TraitAlreadyImplemented {
//         tr: &'static str,
//         span: proc_macro2::Span,
//     },
// }

pub(crate) struct Error<'a> {
    error: ErrorType<'a>,
    span: proc_macro2::Span
}

impl<'a> Error<'a> {
    pub(crate) fn arg_set_twice(arg: &'a str, span: proc_macro2::Span) -> Self {
        Self {
            error: ErrorType::ArgSetTwice(arg),
            span
        }
    }

    pub(crate) fn arg_not_set(arg: &'a str, span: proc_macro2::Span) -> Self {
        Self {
            error: ErrorType::ArgNotSet(arg),
            span
        }
    }

    pub(crate) fn trait_already_implemented(tr: &'a str, span: proc_macro2::Span) -> Self {
        Self {
            error: ErrorType::TraitAlreadyImplemented(tr),
            span
        }
    }

    pub(crate) fn duplicate_maping(name: &'a str, span: proc_macro2::Span) -> Self {
        Self {
            error: ErrorType::DuplicateMaping(name),
            span
        }
    }
}

pub(crate) enum ErrorType<'a> {
    ArgSetTwice(&'a str),
    ArgNotSet(&'a str),
    TraitAlreadyImplemented(&'a str),
    DuplicateMaping(&'a str),
}

impl<'a> From<Error<'a>> for syn::Error {
    fn from(v: Error) -> syn::Error {
        let msg = format!("{}", v.error);
        syn::Error::new(v.span, msg)
    }
}

impl<'a> std::fmt::Display for ErrorType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArgSetTwice(arg) => {
                write!(f, "argument `{arg}` is set twice")
            }
            Self::ArgNotSet(arg) => {
                write!(f, "argument `{arg}` is not set")
            }
            Self::TraitAlreadyImplemented(tr) => {
                write!(f, "trait `{tr}` is already implemented")
            },
            Self::DuplicateMaping(name) => {
                write!(f, "maping with name=`{name}` set twice")
            }
        }
    }
}
