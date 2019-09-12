use rtplot::Figure;
use num::Complex;
use crate::prelude::*;

struct ThreadFigure<'a>(Figure<'a>);

unsafe impl<'a> Send for ThreadFigure<'a> {}

pub struct PlotNode<'a, T>
where
    T: Into<f32> + Copy + Send,
{
    pub input: NodeReceiver<Vec<T>>,
    figure: Option<ThreadFigure<'a>>,
    num_points: usize,
}

impl<'a, T> PlotNode<'a, T>
where
    T: Into<f32> + Copy + Send,
{
    pub fn new(num_points: usize) -> Self {
        PlotNode {
            input: Default::default(),
            figure: None,
            num_points,
        }
    }

    pub fn run(&mut self, input: &[T]) -> Result<(), NodeError> {
        match self.figure {
            Some(ref mut fig) => fig.0.plot_stream(input),
            None => return Err(NodeError::PermanentError),
        }

        Ok(())
    }
}

impl<'a, T> Node for PlotNode<'a, T>
where
    T: Into<f32> + Copy + Send,
{
    fn start(&mut self) {
        self.figure = Some(ThreadFigure(
            Figure::new().init_renderer(self.num_points),
        ));
        loop {
            if self.call().is_err() {
                break;
            }
        }
    }

    fn call(&mut self) -> Result<(), NodeError> {
        let input = match self.input {
            Some(ref r) => r.recv().unwrap(),
            None => return Err(NodeError::PermanentError),
        };
        self.run(&input)
    }

    fn is_connected(&self) -> bool {
        self.input.is_some()
    }
}

pub struct ComplexPlotNode<'a, T> where T: Into<f32> + Copy + Send {
    pub input: NodeReceiver<Vec<Complex<T>>>,
    figure: Option<ThreadFigure<'a>>,
    num_points: usize,
}

impl<'a, T> ComplexPlotNode<'a, T>
where
    T: Into<f32> + Copy + Send,
{
    pub fn new(num_points: usize) -> Self {
        ComplexPlotNode {
            input: Default::default(),
            figure: None,
            num_points,
        }
    }

    pub fn run(&mut self, input: &[Complex<T>]) -> Result<(), NodeError> {
        match self.figure {
            Some(ref mut fig) => fig.0.plot_complex_stream(input),
            None => return Err(NodeError::PermanentError),
        }

        Ok(())
    }
}

impl<'a, T> Node for ComplexPlotNode<'a, T>
where
    T: Into<f32> + Copy + Send,
{
    fn start(&mut self) {
        self.figure = Some(ThreadFigure(
            Figure::new().init_renderer(self.num_points),
        ));
        loop {
            if self.call().is_err() {
                break;
            }
        }
    }

    fn call(&mut self) -> Result<(), NodeError> {
        let input = match self.input {
            Some(ref r) => r.recv().unwrap(),
            None => return Err(NodeError::PermanentError),
        };
        self.run(&input)
    }

    fn is_connected(&self) -> bool {
        self.input.is_some()
    }
}
