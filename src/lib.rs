pub mod variants;
pub use variants::*;

pub fn add_to_fonts(fonts: &mut epaint::text::FontDefinitions, variant: Variant) {
    fonts
        .font_data
        .insert("nerdfonts".into(), variant.font_data());

    if let Some(font_keys) = fonts.families.get_mut(&epaint::FontFamily::Proportional) {
        font_keys.push("nerdfonts".into());
    }
}
