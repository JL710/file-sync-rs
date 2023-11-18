use iced::theme;
use iced::widget;
use iced::widget::button;
use iced::widget::svg;

#[derive(Copy, Clone, Debug)]
pub struct ButtonStyleSheet {
    active: button::Appearance,
    hovered: button::Appearance,
    pressed: button::Appearance,
    disabled: button::Appearance,
}

impl ButtonStyleSheet {
    pub fn new() -> Self {
        let appearance_heavy = button::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb8(51, 89, 218))),
            text_color: iced::Color::from_rgb8(255, 255, 255),
            border_radius: iced::BorderRadius::from(3.0),
            ..Default::default()
        };

        let appearance_light = button::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb8(
                94, 124, 226,
            ))),
            text_color: iced::Color::from_rgb8(255, 255, 255),
            border_radius: iced::BorderRadius::from(3.0),
            ..Default::default()
        };

        Self {
            active: appearance_heavy,
            hovered: appearance_light,
            pressed: appearance_heavy,
            disabled: appearance_light,
        }
    }

    pub fn set_border_radius(mut self, radius: iced::BorderRadius) -> Self {
        self.active.border_radius = radius;
        self.hovered.border_radius = radius;
        self.pressed.border_radius = radius;
        self.disabled.border_radius = radius;
        self
    }

    pub fn set_background(mut self, color_heavy: iced::Color, color_light: iced::Color) -> Self {
        self.active.background = Some(iced::Background::Color(color_heavy));
        self.pressed.background = Some(iced::Background::Color(color_heavy));
        self.disabled.background = Some(iced::Background::Color(color_light));
        self.hovered.background = Some(iced::Background::Color(color_light));
        self
    }
}

impl button::StyleSheet for ButtonStyleSheet {
    type Style = theme::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        self.active
    }

    fn disabled(&self, _style: &Self::Style) -> button::Appearance {
        self.disabled
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        self.hovered
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        self.pressed
    }
}

impl From<ButtonStyleSheet> for iced::theme::Button {
    fn from(value: ButtonStyleSheet) -> Self {
        iced::theme::Button::Custom(Box::new(value))
    }
}

impl Default for ButtonStyleSheet {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SvgStyleSheet {
    r: u8,
    g: u8,
    b: u8,
}

impl SvgStyleSheet {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl svg::StyleSheet for SvgStyleSheet {
    type Style = theme::Theme;

    fn appearance(&self, _style: &Self::Style) -> svg::Appearance {
        svg::Appearance {
            color: Some(iced::Color::from_rgb8(self.r, self.g, self.b)),
        }
    }
}

impl From<SvgStyleSheet> for iced::theme::Svg {
    fn from(value: SvgStyleSheet) -> Self {
        iced::theme::Svg::Custom(Box::new(value))
    }
}

pub struct ContainerStyleSheet {
    appearance: widget::container::Appearance,
}

impl widget::container::StyleSheet for ContainerStyleSheet {
    type Style = theme::Theme;

    fn appearance(&self, style: &Self::Style) -> widget::container::Appearance {
        self.appearance
    }
}

impl ContainerStyleSheet {
    pub fn new() -> Self {
        ContainerStyleSheet {
            appearance: widget::container::Appearance {
                ..Default::default()
            },
        }
    }

    pub fn border_radius(mut self, radius: iced::BorderRadius) -> Self {
        self.appearance.border_radius = radius;
        self
    }

    pub fn background(mut self, background: Option<iced::Background>) -> Self {
        self.appearance.background = background;
        self
    }

    pub fn border_color(mut self, color: iced::Color) -> Self {
        self.appearance.border_color = color;
        self
    }

    pub fn border_width(mut self, width: f32) -> Self {
        self.appearance.border_width = width;
        self
    }
}

impl From<ContainerStyleSheet> for iced::theme::Container {
    fn from(value: ContainerStyleSheet) -> Self {
        iced::theme::Container::Custom(Box::new(value))
    }
}
