#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScreenCoordinates {
    Absolute {
        x: i32,
        y: i32
    },
    
    Percent {
        x: f32,
        y: f32
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}