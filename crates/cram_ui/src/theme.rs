use egui::{Color32, Stroke, Visuals};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    Nord,
    Dracula,
    SolarizedDark,
    SolarizedLight,
    GruvboxDark,
}

impl Theme {
    pub const ALL: [Theme; 7] = [
        Theme::Dark,
        Theme::Light,
        Theme::Nord,
        Theme::Dracula,
        Theme::SolarizedDark,
        Theme::SolarizedLight,
        Theme::GruvboxDark,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
            Theme::Nord => "Nord",
            Theme::Dracula => "Dracula",
            Theme::SolarizedDark => "Solarized Dark",
            Theme::SolarizedLight => "Solarized Light",
            Theme::GruvboxDark => "Gruvbox Dark",
        }
    }

    pub fn is_dark(self) -> bool {
        !matches!(self, Theme::Light | Theme::SolarizedLight)
    }

    pub fn visuals(self) -> Visuals {
        match self {
            Theme::Dark => Visuals::dark(),
            Theme::Light => Visuals::light(),
            _ => {
                let palette = self.palette();
                let mut v = if self.is_dark() {
                    Visuals::dark()
                } else {
                    Visuals::light()
                };
                v.panel_fill = palette.panel;
                v.window_fill = palette.panel;
                v.faint_bg_color = palette.faint;
                v.extreme_bg_color = palette.extreme;
                v.window_stroke = Stroke::new(1.0, palette.border);
                v.widgets.noninteractive.bg_fill = palette.faint;
                v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.border);
                v
            }
        }
    }

    fn palette(self) -> Palette {
        match self {
            Theme::Nord => Palette {
                panel: Color32::from_rgb(0x2E, 0x34, 0x40),
                faint: Color32::from_rgb(0x3B, 0x42, 0x52),
                extreme: Color32::from_rgb(0x43, 0x4C, 0x5E),
                border: Color32::from_rgb(0x4C, 0x56, 0x6A),
            },
            Theme::Dracula => Palette {
                panel: Color32::from_rgb(0x28, 0x2A, 0x36),
                faint: Color32::from_rgb(0x44, 0x47, 0x5A),
                extreme: Color32::from_rgb(0x38, 0x3A, 0x59),
                border: Color32::from_rgb(0x62, 0x72, 0xA4),
            },
            Theme::SolarizedDark => Palette {
                panel: Color32::from_rgb(0x00, 0x2B, 0x36),
                faint: Color32::from_rgb(0x07, 0x36, 0x42),
                extreme: Color32::from_rgb(0x0A, 0x40, 0x50),
                border: Color32::from_rgb(0x58, 0x6E, 0x75),
            },
            Theme::SolarizedLight => Palette {
                panel: Color32::from_rgb(0xFD, 0xF6, 0xE3),
                faint: Color32::from_rgb(0xEE, 0xE8, 0xD5),
                extreme: Color32::from_rgb(0xFC, 0xF5, 0xE2),
                border: Color32::from_rgb(0x93, 0xA1, 0xA1),
            },
            Theme::GruvboxDark => Palette {
                panel: Color32::from_rgb(0x28, 0x28, 0x28),
                faint: Color32::from_rgb(0x3C, 0x38, 0x36),
                extreme: Color32::from_rgb(0x50, 0x49, 0x45),
                border: Color32::from_rgb(0x66, 0x5C, 0x54),
            },
            // Dark and Light use default egui visuals, so palette is unused.
            Theme::Dark | Theme::Light => Palette {
                panel: Color32::TRANSPARENT,
                faint: Color32::TRANSPARENT,
                extreme: Color32::TRANSPARENT,
                border: Color32::TRANSPARENT,
            },
        }
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

struct Palette {
    panel: Color32,
    faint: Color32,
    extreme: Color32,
    border: Color32,
}
