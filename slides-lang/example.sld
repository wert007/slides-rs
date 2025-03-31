// My file!
styling blue_bg(Label):
    text_color = c"white";

styling default(Slide):
    background = rgb(120, 160, 180);

element image_with_caption(img: Image, caption: string):
    let img = img {
        valign: VAlign.Stretch,
        halign: HAlign.Stretch,
        object_fit: ObjectFit.Cover,
    };
    let caption = label(caption) {
        text_color: c"white",
        text_align: TextAlign.Right,
        valign: VAlign.Bottom,
        halign: HAlign.Right,
    };

slide intro:
    background = rgb(255, 127, 127);
    let i = image_with_caption(
        image(p"./assets/mountain.jpg"),
        "Mountain, 2024, pixabay"
    ) {
        valign: VAlign.Stretch,
        halign: HAlign.Stretch,
    };
    let lbl = l"Hello World";
    lbl.text_color = c"white";
    lbl.background = c"#616161";

slide hello:
    let a = l"""
        # This could be a title

         - With a list
         - of elements
         - to have a nice markdown support
         - which should be all that's needed for text. probably!
    """ {
        text_color: c"green",
        background: c"fuchsia",
    };

// Last line comment! Would be trivia of eof!
