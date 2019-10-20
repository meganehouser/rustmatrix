use cursive::Cursive;
use cursive::theme::Theme;
use cursive::theme::Color;
use crate::view::GreenCodeView;

mod view;


fn main() {
    let mut siv: Cursive = Cursive::default();
    let size = siv.screen_size();
    let mut theme = Theme::default();
    theme.palette.set_color("view", Color::TerminalDefault );
    siv.set_theme(theme);

    siv.add_fullscreen_layer(GreenCodeView::new(1, size));
    siv.set_autorefresh(true);
    siv.run();
}
