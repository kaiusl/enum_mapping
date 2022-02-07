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
    SetTwice {
        arg: &'static str,
        span: proc_macro2::Span,
    },
    NotSet {
        arg: &'static str,
        span: proc_macro2::Span,
    },
}

impl Error {
    pub(crate) fn set_twice(arg: &'static str) -> Self {
        Self::SetTwice {
            arg,
            span: proc_macro2::Span::call_site(),
        }
    }

    pub(crate) fn not_set(arg: &'static str) -> Self {
        Self::NotSet {
            arg,
            span: proc_macro2::Span::call_site(),
        }
    }
}

impl From<Error> for syn::Error {
    fn from(v: Error) -> syn::Error {
        match v {
            Error::SetTwice { arg, span } => {
                syn::Error::new(span, format! {"argument `{arg}` is set twice"})
            }
            Error::NotSet { arg, span } => {
                syn::Error::new(span, format! {"argument `{arg}` is not set"})
            }
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetTwice { arg, .. } => {
                write!(f, "argument `{arg}` is set twice")
            }
            Self::NotSet { arg, .. } => {
                write!(f, "argument `{arg}` is not set")
            }
        }
    }
}
