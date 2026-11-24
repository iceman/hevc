use colored::CustomColor;
pub use colored::{ColoredString, Colorize};

macro_rules! define_color {
	($fn_name:ident, $r:expr, $g:expr, $b:expr) => {
		fn $fn_name(self) -> ColoredString {
			static COLOR: std::sync::OnceLock<CustomColor> =
				std::sync::OnceLock::new();
			self.custom_color(*COLOR.get_or_init(|| CustomColor::new($r, $g, $b)))
		}
	};
}

pub trait CustomColors: Colorize + Sized {
	define_color!(soft_green, 104, 157, 97);
	define_color!(gruvbox_red, 251, 73, 52);
}

impl<S: Colorize> CustomColors for S {}
