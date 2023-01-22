pub trait CodePass<'input>: Sized {
    type Prev;
    type Arg;

    #[allow(unused)]
    fn pass(
        previous: Self::Prev,
        input: &'input str,
        arg: &mut impl Is<Self::Arg>,
    ) -> miette::Result<Self> {
        unimplemented!()
    }
    #[allow(unused)]
    fn check(
        previous: Self::Prev,
        input: &'input str,
        arg: &impl Is<Self::Arg>,
    ) -> miette::Result<Self::Prev> {
        unimplemented!()
    }
}

pub struct PassRunner<'input, P, A> {
    cur: P,
    input: &'input str,
    arg: A,
}
impl<'input, P, A> PassRunner<'input, P, A> {
    pub fn new(input: &'input str) -> PassRunner<(), ()> {
        PassRunner::<(), ()> {
            cur: (),
            input,
            arg: (),
        }
    }

    pub fn set_arg<N>(self, arg: N) -> PassRunner<'input, P, N> {
        PassRunner {
            cur: self.cur,
            input: self.input,
            arg,
        }
    }

    pub fn inject<N: CodePass<'input>>(self) -> miette::Result<Self>
    where
        P: From<N::Prev>,
        N::Prev: From<P>,
        A: Is<N::Arg>,
    {
        let next = N::check(self.cur.into(), self.input, &self.arg)
            .map_err(|e| e.with_source_code(self.input.to_string()))?;
        Ok(Self {
            cur: next.into(),
            input: self.input,
            arg: self.arg,
        })
    }

    // pub fn inject_with_arg<N: CodePass<'input>>(self, mut arg: N::Arg) -> miette::Result<Self>
    // where
    //     P: From<N::Prev>,
    //     N::Prev: From<P>,
    // {
    //     let cur = N::check(self.cur.into(), self.input, &mut arg)
    //         .map_err(|e| e.with_source_code(self.input.to_string()))?;
    //     Ok(Self {
    //         cur: cur.into(),
    //         input: self.input,
    //         arg: self.arg,
    //     })
    // }

    pub fn run<N: CodePass<'input>>(mut self) -> miette::Result<PassRunner<'input, N, A>>
    where
        P: Into<N::Prev>,
        A: Is<N::Arg>,
    {
        let cur = N::pass(self.cur.into(), self.input, &mut self.arg)
            .map_err(|e| e.with_source_code(self.input.to_string()))?;
        Ok(PassRunner::<N, A> {
            cur,
            input: self.input,
            arg: self.arg,
        })
    }

    // pub fn run_with_arg<N: CodePass<'input>>(
    //     self,
    //     arg: N::Arg,
    // ) -> miette::Result<PassRunner<'input, N, N::Arg>>
    // where
    //     P: Into<N::Prev>,
    // {
    //     let new_self = PassRunner::<P, N::Arg> {
    //         cur: self.cur,
    //         input: self.input,
    //         arg,
    //     };
    //     let new_self = new_self.run::<N>()?;
    //     Ok(new_self)
    // }
}

pub trait Is<T> {
    fn is(&self) -> &T;
    fn is_mut(&mut self) -> &mut T;
}
impl<T> Is<T> for T {
    fn is(&self) -> &T {
        self
    }
    fn is_mut(&mut self) -> &mut T {
        self
    }
}
