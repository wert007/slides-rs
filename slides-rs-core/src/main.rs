use slides_rs_core::{
    Background, Color, Image, ImageSource, ImageStyling, Label, LabelStyling, ObjectFit,
    Positioning, Presentation, Slide, SlideStyling,
};

fn main() -> slides_rs_core::Result<()> {
    let mut presentation = Presentation::new();

    presentation.add_slide(
        Slide::new()
            .with_styling(
                SlideStyling::default()
                    .with_background(Background::Color(Color::from_rgb(255, 127, 127))),
            )
            .add_image(
                Image::new(ImageSource::path("assets/mountain.jpg"))
                    .with_positioning(Positioning::new().with_alignment_stretch())
                    .with_styling(ImageStyling::default().with_object_fit(ObjectFit::Cover)),
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
