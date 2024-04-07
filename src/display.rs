use std::io;

use crate::{
    render_action::{RenderAction, RenderAction2},
    State,
};

pub trait Display {
    fn enqueue_action(&mut self, action: RenderAction);
    fn enqueue_action2(&mut self, action: RenderAction2);
    fn draw_initial(&mut self, state: &State) -> io::Result<()>;
    fn draw(&mut self, state: &State) -> io::Result<()>;
}
