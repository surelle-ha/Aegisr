use figlet_rs::FIGfont;

pub struct Banner;

impl Banner {
    pub fn render(text: &str) {
        let standard_font: FIGfont = FIGfont::standard().expect("Failed to load standard font");
        let figure: Option<figlet_rs::FIGure<'_>> = standard_font.convert(text);

        match figure {
            Some(fig) => println!("{}", fig),
            None => eprintln!("Failed to render banner for text: '{}'", text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_banner() {
        Banner::render("Hello Rust");
    }
}
