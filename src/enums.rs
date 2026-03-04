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
    TopCenter,
    TopRight,
    CenterRight,
    BottomRight,
    BottomCenter,
    BottomLeft,
    CenterLeft,
}

impl Corner {
    pub fn all() -> Vec<Corner> {
        vec![
            Corner::TopLeft,
            Corner::TopCenter,
            Corner::TopRight,
            Corner::CenterRight,
            Corner::BottomRight,
            Corner::BottomCenter,
            Corner::BottomLeft,
            Corner::CenterLeft,
        ]
    }
}