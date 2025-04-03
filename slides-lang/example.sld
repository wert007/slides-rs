// My file!
styling blue_bg(Label):
    text_color = c"white";

styling default(Slide):
    background = rgb(120, 160, 180);

element image_with_caption(img: Image, caption: String):
    img = img {
        object_fit: ObjectFit.Cover,
        valign: VAlign.Stretch,
        halign: HAlign.Stretch,
    };
    let captionLabel = label(caption) {
        text_color: c"white",
        text_align: TextAlign.Right,
        valign: VAlign.Bottom,
        halign: HAlign.Right,
    };

slide intro:
    background = rgb(255, 127, 127);
    let i = image_with_caption(image(p"./assets/mountain.jpg"), "Mountain, 2024, pixabay") {
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
        valign: VAlign.Center,
        halign: HAlign.Center,
    };

slide code:
    let a = l"""
        ```rust
        fn main() {
            println!("Hello World!");
        }
        ```
    """ {
        valign: VAlign.Center,
        halign: HAlign.Center,
    };

// Last line comment! Would be trivia of eof!
