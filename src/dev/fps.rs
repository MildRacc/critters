use bevy::{color::Color};


pub struct OverlayColor;
impl OverlayColor
{
    pub const RED: Color = Color::srgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::srgb(0.0, 1.0, 1.0);
}
