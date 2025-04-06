use slides_rs_core::{
    Background, Color, Font, Image, ImageSource, ImageStyling, Label, LabelStyling, ObjectFit,
    Presentation, Slide, SlideStyling, StyleUnit, Thickness,
};

fn main() -> slides_rs_core::Result<()> {
    let mut presentation = Presentation::new();
    Ok(())
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

    //     let blue_bg_lbl = presentation.add_styling(
    //         LabelStyling::new()
    //             .with_background(Background::Color(Color::rgb(20, 60, 180)))
    //             .with_text_color(Color::WHITE),
    //         "blue-background",
    //     );

    //     presentation.set_default_style(LabelStyling::new().with_font(Font::gfont("Quicksand")));
    //     presentation.set_default_style(
    //         SlideStyling::new().with_background(Background::Color(Color::rgb(96, 96, 96))),
    //     );

    //     presentation.add_slide(
    //         Slide::new()
    //             .with_styling(
    //                 SlideStyling::new().with_background(Background::Color(Color::rgb(255, 127, 127))),
    //             )
    //             .add_image(
    //                 Image::new(ImageSource::path("assets/mountain.jpg"))
    //                     .with_positioning(Positioning::new().with_alignment_stretch())
    //                     .with_element_styling(ImageStyling::new().with_object_fit(ObjectFit::Cover)),
    //             )
    //             .add_label(
    //                 Label::new("Hello World!")
    //                     .with_positioning(
    //                         Positioning::new()
    //                             .with_alignment_center()
    //                             .with_padding(Thickness::all(StyleUnit::Pixel(50.0))),
    //                     )
    //                     .with_element_styling(
    //                         LabelStyling::new()
    //                             .with_background(Background::Color(Color::argb(51, 51, 51, 180)))
    //                             .with_text_color(Color::WHITE)
    //                             .with_font(Font::gfont("Roboto")),
    //                     ),
    //             )
    //             .add_label(
    //                 Label::new("Hiiiiii World!")
    //                     .with_element_styling(
    //                         LabelStyling::new()
    //                             .with_background(Background::Color(Color::rgb(127, 127, 255)))
    //                             .with_font(Font::system("Arial")),
    //                     )
    //                     .with_positioning(
    //                         Positioning::new().with_padding(Thickness::all(StyleUnit::Pixel(10.0))),
    //                     ),
    //             ),
    //     );

    //     presentation.add_slide(
    //         Slide::new().add_label(
    //             Label::new(
    //                 r#"# This could be a title

    // - With a list
    // - of elements
    // - to have a nice markdown support
    // - which should be all that's needed for text. probably!"#,
    //             )
    //             .with_positioning(Positioning::new().with_alignment_center())
    //             .with_styling(blue_bg_lbl),
    //         ),
    //     );

    //     presentation.output_to_directory("out_presentation")
}
