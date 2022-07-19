use stylist::css;
use stylist::StyleSource;

pub struct Themes;

impl Themes {
    pub fn global(self) -> StyleSource {
        css!(
            r#"
            html, body {
                font-family: sans-serif;
                padding: 0;
                margin: 0;
                display: flex;
                justify-content: center;
                align-items: center;
                min-height: 100vh;
                flex-direction: column;
                background-color: white;
            }
            "#
        )
    }

}
