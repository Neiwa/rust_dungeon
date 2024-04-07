use std::io;

use crate::{render_action::RenderAction, State};

pub trait Display {
    fn enqueue_action(&mut self, action: RenderAction);
    fn draw_initial(&mut self, state: &State) -> io::Result<()>;
    fn draw(&mut self, state: &State) -> io::Result<()>;
}
