styling blue_bg(Label):
    it.background = rgb(20, 60, 180);
    it.text_color = c"white";

slide intro:
    slide_.styling_.background = rgb(255, 127, 127);
    let dict = {
        position: { alignment: .center, },
    };
    // Creation of anonymous image
    // image(p"assets/mountain") {
    //    position: { alignment: center, }
    //    styling: {
    //            "object-fit": cover,
    //    }
    //};
    let lbl = l"Hello World";
    lbl.position.align_center();
    lbl.styling_.text_color = c"white";
    lbl.styling_.background = c"#616161";


//slide hello:
//    l"""
//    # This could be a title
//
//    - With a list
//    - of elements
//    - to have a nice markdown support
//    - which should be all that's needed for text. probably!
//    """ {
//    position: { alignment: center },
//    styling: {
//        references: [ blue_bg ]
//    }
//}
