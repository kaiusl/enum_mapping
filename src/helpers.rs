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

pub(crate) enum Error {
    ArgSetTwice {
        arg: &'static str,
        span: proc_macro2::Span,
    },
    ArgNotSet {
        arg: &'static str,
        span: proc_macro2::Span,
    },
    TraitImplemented {
        tr: &'static str,
        span: proc_macro2::Span
    }
}

impl Error {
    pub(crate) fn set_twice(arg: &'static str) -> Self {
        Self::ArgSetTwice {
            arg,
            span: proc_macro2::Span::call_site(),
        }
    }

    pub(crate) fn not_set(arg: &'static str) -> Self {
        Self::ArgNotSet {
            arg,
            span: proc_macro2::Span::call_site(),
        }
    }
}

impl From<Error> for syn::Error {
    fn from(v: Error) -> syn::Error {
        let msg = format!("{}", v);
        let span = match v {
            Error::ArgSetTwice { span,.. } => span,
            Error::ArgNotSet { span, .. } => span,
            Error::TraitImplemented {span, ..} => span,
        };
        syn::Error::new(span, msg)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArgSetTwice { arg, .. } => {
                write!(f, "argument `{arg}` is set twice")
            }
            Self::ArgNotSet { arg, .. } => {
                write!(f, "argument `{arg}` is not set")
            }
            Self::TraitImplemented {tr, ..} => {
                write!(f, "trait `{tr}` is already implemented")
            }
        }
    }
}
