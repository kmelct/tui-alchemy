use std::io::{self, Write};

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
pub const README_PATH: &str = "README.md";
pub const SCREENSHOTS_PATH: &str = "docs/screenshots/";

pub fn write_version(mut writer: impl Write) -> io::Result<()> {
    writeln!(writer, "{NAME} {VERSION}")
}

pub fn write_help(mut writer: impl Write, binary_name: &str) -> io::Result<()> {
    writeln!(writer, "{NAME} {VERSION}")?;
    writeln!(writer, "{DESCRIPTION}")?;
    writeln!(writer)?;
    writeln!(writer, "Usage: {binary_name} [--help|--version]")?;
    writeln!(writer)?;
    writeln!(writer, "Play:")?;
    writeln!(writer, "  Pick two discovered elements to test a recipe.")?;
    writeln!(writer, "  Starter recipe: Water + Fire -> Steam.")?;
    writeln!(
        writer,
        "  Use arrow keys or h/j/k/l, Enter, 1-9, PageUp/PageDown, [ and ], Esc, c, or mouse drag/drop."
    )?;
    writeln!(writer, "  Press q to quit.")?;
    writeln!(writer)?;
    writeln!(writer, "Tutorial: {README_PATH}")?;
    writeln!(writer, "Screenshots: {SCREENSHOTS_PATH}")?;
    writeln!(writer, "Repository: {REPOSITORY}")
}

#[cfg(test)]
mod tests {
    #[test]
    fn help_text_points_players_to_readme_and_screenshots() {
        let mut output = Vec::new();

        super::write_help(&mut output, "tui-alchemy").expect("help writes");
        let help = String::from_utf8(output).expect("valid utf-8");

        assert!(help.contains("README.md"));
        assert!(help.contains("docs/screenshots/"));
        assert!(help.contains("Water + Fire"));
    }

    #[test]
    fn version_text_uses_package_metadata() {
        let mut output = Vec::new();

        super::write_version(&mut output).expect("version writes");
        let version = String::from_utf8(output).expect("valid utf-8");

        assert!(version.contains(env!("CARGO_PKG_NAME")));
        assert!(version.contains(env!("CARGO_PKG_VERSION")));
    }
}
