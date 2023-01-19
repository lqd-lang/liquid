pub trait CodePass<'input>: Sized {
    type Prev;
    type Arg;

    #[allow(unused)]
    fn pass(previous: Self::Prev, input: &'input str, arg: Self::Arg) -> miette::Result<Self> {
        unimplemented!()
    }
    #[allow(unused)]
    fn check(
        previous: Self::Prev,
        input: &'input str,
        arg: Self::Arg,
    ) -> miette::Result<Self::Prev> {
        unimplemented!()
    }
}

pub struct PassRunner<'input, P> {
    cur: P,
    input: &'input str,
}
impl<'input, P> PassRunner<'input, P> {
    pub fn new(input: &'input str) -> PassRunner<()> {
        PassRunner::<()> { cur: (), input }
    }

    pub fn inject<N: CodePass<'input>>(self) -> miette::Result<Self>
    where
        P: From<N::Prev>,
        N::Prev: From<P>,
        N::Arg: From<()>,
    {
        let next = N::check(self.cur.into(), self.input, ().into())
            .map_err(|e| e.with_source_code(self.input.to_string()))?;
        Ok(Self {
            cur: next.into(),
            input: self.input,
        })
    }

    pub fn run<N: CodePass<'input>>(self) -> miette::Result<PassRunner<'input, N>>
    where
        P: Into<N::Prev>,
        N::Arg: From<()>,
    {
        let cur = N::pass(self.cur.into(), self.input, ().into())
            .map_err(|e| e.with_source_code(self.input.to_string()))?;
        Ok(PassRunner::<N> {
            cur,
            input: self.input,
        })
    }

    pub fn run_with_arg<N: CodePass<'input>>(
        self,
        arg: N::Arg,
    ) -> miette::Result<PassRunner<'input, N>>
    where
        P: Into<N::Prev>,
    {
        let cur = N::pass(self.cur.into(), self.input, arg)
            .map_err(|e| e.with_source_code(self.input.to_string()))?;
        Ok(PassRunner::<N> {
            cur,
            input: self.input,
        })
    }
}
