styling blue_bg(Label):
    background = rgb(20, 60, 180);
    text_color = c"white";

styling default(Slide):
    background = rgb(120, 160, 180);

slide intro:
    background = rgb(255, 127, 127);
    // let dict = {
    //     alignment: .center,
    // };
    // Creation of anonymous image
    let i = image(p"./assets/mountain.jpg");
    //  {
    //    alignment: .center,
    //
    //            object-fit: cover,
    //
    //};
    let lbl = l"Hello World";
    // lbl.align_center();
    lbl.text_color = c"white";
    lbl.background = c"#616161";


slide hello:
   let a = l"""
   # This could be a title

   - With a list
   - of elements
   - to have a nice markdown support
   - which should be all that's needed for text. probably!
   """
