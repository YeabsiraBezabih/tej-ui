//! Hardware, screens, and device icons (Lucide catalog).

/// Device and hardware icon variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceIcon {
    Laptop,
    Monitor,
    Smartphone,
    Tablet,
    Tv,
    HardDrive,
    Cpu,
    Battery,
    BatteryCharging,
    Wifi,
    WifiOff,
    Bluetooth,
    Server,
    Database,
    Terminal,
}

impl DeviceIcon {
    /// Returns the SVG vector path data in 24x24 coordinate space.
    #[must_use]
    pub const fn svg_path(self) -> &'static str {
        match self {
            Self::Laptop => {
                "M20 16V7a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v9m16 0H4m16 0 1.28 2.55A1 1 0 0 1 20.38 20H3.62a1 1 0 0 1-.9-1.45L4 16"
            }
            Self::Monitor => {
                "M20 3H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2z M8 21h8 M12 17v4"
            }
            Self::Smartphone => {
                "M17 2H7a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2z M12 18h.01"
            }
            Self::Tablet => {
                "M18 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2z M12 18h.01"
            }
            Self::Tv => {
                "M20 7H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2z M17 2l-5 5-5-5"
            }
            Self::HardDrive => {
                "M22 12H2 M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z M6 16h.01 M10 16h.01"
            }
            Self::Cpu => {
                "M4 4h16v16H4z M9 9h6v6H9z M9 1v3 M15 1v3 M9 20v3 M15 20v3 M20 9h3 M20 14h3 M1 9h3 M1 14h3"
            }
            Self::Battery => {
                "M17 7H3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2z M23 13v-2"
            }
            Self::BatteryCharging => {
                "M15 7H3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2z M23 13v-2 M11 7l-3 5h4l-3 5"
            }
            Self::Wifi => {
                "M5 12.55a11 11 0 0 1 14.08 0 M1.42 9a16 16 0 0 1 21.16 0 M8.53 16.11a6 6 0 0 1 6.95 0 M12 20h.01"
            }
            Self::WifiOff => {
                "M1 1l22 22 M16.72 11.06A10.94 10.94 0 0 1 19.08 12.55 M5 12.55a10.94 10.94 0 0 1 5.17-2.39 M10.71 5.05A16 16 0 0 1 22.58 9 M1.42 9a15.91 15.91 0 0 1 4.7-2.88 M8.53 16.11a6 6 0 0 1 6.95 0 M12 20h.01"
            }
            Self::Bluetooth => "M6.5 6.5l11 11L12 23V1l5.5 5.5-11 11",
            Self::Server => "M2 2h20v8H2z M2 14h20v8H2z M6 6h.01 M6 18h.01",
            Self::Database => {
                "M12 2C6.48 2 2 3.79 2 6v12c0 2.21 4.48 4 10 4s10-1.79 10-4V6c0-2.21-4.48-4-10-4z M2 12c0 2.21 4.48 4 10 4s10-1.79 10-4 M2 6c0 2.21 4.48 4 10 4s10-1.79 10-4"
            }
            Self::Terminal => "M4 17l6-6-6-6 M12 19h8",
        }
    }
}
