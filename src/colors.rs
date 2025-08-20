use ratzilla::ratatui::style::Color;

#[derive(Default)]
pub struct ColourTheme {
    pub color_bg: Color,
    pub color_fg: Color,
    pub color_bg_alt: Color,
    pub color_fg_alt: Color,
    pub color_5: Color,
    pub color_6: Color,
    pub name: String,
    id: usize,
}

#[allow(dead_code)]
impl ColourTheme {
    pub fn new() -> ColourTheme {
        ColourTheme {
            color_bg: Color::Black,
            color_fg: Color::White,
            color_bg_alt: Color::DarkGray,
            color_fg_alt: Color::LightBlue,
            color_5: Color::Green,
            color_6: Color::Cyan,
            name: "Starter".to_string(),
            id: 0,
        }
    }

    pub fn switch_colour(&mut self) {
        // shoddy coding here change later please :3
        match self.id {
            1 => self.to_campfire(),
            2 => self.to_stag(),
            _ => self.to_yellow(),
        }
        self.id += 1;
        if self.id > 2 {
            self.id = 0;
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_yellow(&mut self) {
        self.color_bg = Color::from_u32(0x002E_281D);
        self.color_fg = Color::from_u32(0x00DC_D0B4);
        self.color_bg_alt = Color::from_u32(0x009A_927D);
        self.color_fg_alt = Color::from_u32(0x00EC_C570);
        self.color_5 = Color::from_u32(0x00C9_B077);
        self.color_6 = Color::from_u32(0x00AA_9871);
        self.name = "Smokey Yellow".to_string();
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_campfire(&mut self) {
        self.color_bg = Color::from_u32(0x0022_222E);
        self.color_fg = Color::from_u32(0x0099_BCC1);
        self.color_bg_alt = Color::from_u32(0x002A_4C66);
        self.color_fg_alt = Color::from_u32(0x00E6_945B);
        self.color_5 = Color::from_u32(0x0091_2D2B);
        self.color_6 = Color::from_u32(0x005C_4954);
        self.name = "Campfire".to_string();
    }
    #[allow(clippy::wrong_self_convention)]
    pub fn to_stag(&mut self) {
        self.color_bg = Color::from_u32(0x000D1528);
        self.color_fg = Color::from_u32(0x00DE92A8);
        self.color_bg_alt = Color::from_u32(0x002A4461);
        self.color_fg_alt = Color::from_u32(0x00CF3961);
        self.color_5 = Color::from_u32(0x008D3950);
        self.color_6 = Color::from_u32(0x007E4576);
        self.name = "Stag".to_string();
    }
}
