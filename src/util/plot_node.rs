use crate::prelude::*;
use num::Complex;
use rtplot::{Figure, FigureConfig};

struct ThreadFigure<'a>(Figure<'a>);

unsafe impl<'a> Send for ThreadFigure<'a> {}

pub struct PlotNode<'a, T>
where
    T: Into<f32> + Copy + Send,
{
    pub input: NodeReceiver<Vec<T>>,
    figure: Option<ThreadFigure<'a>>,
    config: FigureConfig<'a>,
    queue_size: usize,
    use_stream_plotting: bool,
}

impl<'a, T> PlotNode<'a, T>
where
    T: Into<f32> + Copy + Send,
{
    pub fn new(config: FigureConfig<'a>, queue_size: usize, use_stream_plotting: bool) -> Self {
        PlotNode {
            input: Default::default(),
            figure: None,
            config,
            use_stream_plotting,
            queue_size,
        }
    }

    pub fn run(&mut self, input: &[T]) -> Result<(), NodeError> {
        match self.figure {
            Some(ref mut fig) => {
                fig.0.should_close_window();
                if self.use_stream_plotting {
                    fig.0.plot_stream(input);
                } else {
                    fig.0.plot_y(input);
                }
            }
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
            Figure::new_with_config(self.config.clone(), self.queue_size)
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

pub struct ComplexPlotNode<'a, T>
where
    T: Into<f32> + Copy + Send,
{
    pub input: NodeReceiver<Vec<Complex<T>>>,
    figure: Option<ThreadFigure<'a>>,
    config: FigureConfig<'a>,
    use_stream_plotting: bool,
    queue_size: usize,
}

impl<'a, T> ComplexPlotNode<'a, T>
where
    T: Into<f32> + Copy + Send,
{
    pub fn new(config: FigureConfig<'a>, queue_size: usize, use_stream_plotting: bool) -> Self {
        ComplexPlotNode {
            input: Default::default(),
            figure: None,
            config,
            use_stream_plotting,
            queue_size,
        }
    }

    pub fn run(&mut self, input: &[Complex<T>]) -> Result<(), NodeError> {
        match self.figure {
            Some(ref mut fig) => {
                fig.0.should_close_window();
                if self.use_stream_plotting {
                    fig.0.plot_complex_stream(input);
                } else {
                    fig.0.plot_complex(input);
                }
            }
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
            Figure::new_with_config(self.config.clone(), self.queue_size)
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
