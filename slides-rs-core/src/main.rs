use slides_rs_core::{
    Background, Color, Image, ImageSource, ImageStyling, Label, LabelStyling, ObjectFit,
    Positioning, Presentation, Slide, SlideStyling,
};

fn main() -> slides_rs_core::Result<()> {
    let mut presentation = Presentation::new();

    // slide intro:
    //  slide.styling.background(rgb(255, 127, 127));
    //  // Creation of anonymous image
    //  image(p"assets/mountain") {
    //      positioning: { alignment: center, }
    //      styling: {
    //          object-fit: cover,
    //      }
    //  };
    //  let lbl = l"Hello World";
    //  lbl.position.align_center();
    //  lbl.styling.text_color = c"white";
    //  lbl.styling.background = c"#616161";
    //
    //
    //

    let blue_bg_lbl = presentation.add_styling(
        LabelStyling::new().with_background(Background::Color(Color::from_rgb(20, 60, 180))),
    );

    presentation.add_slide(
        Slide::new()
            .with_styling(
                SlideStyling::default()
                    .with_background(Background::Color(Color::from_rgb(255, 127, 127))),
            )
            .add_image(
                Image::new(ImageSource::path("assets/mountain.jpg"))
                    .with_positioning(Positioning::new().with_alignment_stretch())
                    .with_element_styling(ImageStyling::new().with_object_fit(ObjectFit::Cover)),
            )
            .add_label(
                Label::new("Hello World!")
                    .with_positioning(Positioning::new().with_alignment_center())
                    .with_element_styling(
                        LabelStyling::new()
                            .with_background(Background::Color(Color::from_rgb(51, 51, 51)))
                            .with_text_color(Color::WHITE),
                    ),
            )
            .add_label(
                Label::new("Hiiiiii World!").with_element_styling(
                    LabelStyling::new()
                        .with_background(Background::Color(Color::from_rgb(127, 127, 255))),
                ),
            ),
    );

    presentation.add_slide(
        Slide::new().add_label(
            Label::new(
                r#"# This could be a title

- With a list
- of elements
- to have a nice markdown support
- which should be all that's needed for text. probably!"#,
            )
            .with_positioning(Positioning::new().with_alignment_center()),
        ),
    );

    presentation.output_to_directory("out_presentation")
}
