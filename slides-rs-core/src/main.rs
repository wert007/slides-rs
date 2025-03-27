use slides_rs_core::{
    Background, Color, Label, LabelStyling, Positioning, Presentation, Slide, SlideStyling,
};

fn main() -> slides_rs_core::Result<()> {
    let mut presentation = Presentation::new();

    presentation.add_slide(
        Slide::new()
            .with_styling(
                SlideStyling::default()
                    .with_background(Background::Color(Color::from_rgb(255, 127, 127))),
            )
            .add_label(
                Label::new("Hello World!")
                    .with_positioning(Positioning::new().with_alignment_center()),
            )
            .add_label(
                Label::new("Hiiiiii World!").with_styling(
                    LabelStyling::default()
                        .with_background(Background::Color(Color::from_rgb(127, 127, 255))),
                ),
            ),
    );

    presentation.output_to_directory("out_presentation")
}
