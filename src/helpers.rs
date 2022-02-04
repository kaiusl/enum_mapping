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
