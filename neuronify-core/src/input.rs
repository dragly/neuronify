use visula::winit::dpi::PhysicalPosition;

#[derive(Clone, Copy, Debug)]
pub struct Mouse {
    pub left_down: bool,
    pub position: Option<PhysicalPosition<f64>>,
    pub delta_position: Option<PhysicalPosition<f64>>,
}

pub struct Keyboard {
    pub shift_down: bool,
}
