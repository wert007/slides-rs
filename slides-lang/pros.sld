styling default(Label):
    font = gfont("Roboto");
    text_color = c"black";
    // h1
    // h2
    // h3
    // h4
    // h5
    // h6
    // code
    // text
    text.text_color = c"#595959";

styling default(Slide):
    background = c"#f3f3f3";
    padding = {
        top: 0.2sh,
        right: 0.1sw,
        bottom: 0.2sh,
        left: 0.1sw,
    };

element image_with_caption(img: Image, caption: String):
    img = img {
        object_fit: ObjectFit.Cover,
        valign: VAlign.Stretch,
        halign: HAlign.Stretch,
    }
    let lbl =
        label(caption) {
            valign: VAlign.Bottom,
            halign: HAlign.Right,
            text_color: c"white",
            font_size: 0.8,
        }
    valign = VAlign.Stretch;
    halign = HAlign.Stretch;

template page_number(color: Color = c"black"):
    let number = slide_index - 3;
    let page_number =
        l'{number}' {
            valign: VAlign.Bottom,
            halign: HAlign.Center,
            text_align: TextAlign.Center,
            text_color: color,
            z_index: 999,
        };

slide code:
    let bg =
        image_with_caption(
            image(p"./pros-assets/code.jpg"),
            "Image by Christopher Kuszajewski from Pixabay");

slide threads:
    let bg =
        image_with_caption(
            image(p"./pros-assets/threads.jpg"),
            "Bild von Myriams-Fotos auf Pixabay");

slide frozen:
    let bg = image_with_caption(image(p"./pros-assets/frozen.jpg"), "Bild von adege auf Pixabay");

slide title:
    let title =
        l"# Einseitige Synchronisation und Deadlocks" {
            text_align: TextAlign.Center,
            font_size: 2.5,
            valign: VAlign.Bottom,
            halign: HAlign.Stretch,
            margin: {
                bottom: 0.50sh,
            },
        }
    let subtitle =
        l"## Nach „Parallele Programmierung spielend gelernt mit dem ‚Java-Hamster-Modell‘“ im Proseminar Praktische Informatik" {
            text_align: TextAlign.Center,
            font_size: 1.3,
            valign: VAlign.Top,
            halign: HAlign.Stretch,
            text_color: c"#595959",
            margin: {
                top: 0.50sh,
                left: 0.15sw,
                right: 0.15sw,
            },
        }

slide toc:
    page_number();
    let toc =
        l"""
            # Übersicht

            1. Übersicht
            2. Beispiel
            3. Einseitige Synchronisation
            4. Deadlocks
        """ {
        };

slide example:
    let bg =
        image_with_caption(
            image(p"./pros-assets/car-repair.jpg") {
                filter: brightness(0.4),
            },
            "Bild von emkanicepic auf Pixabay");

    page_number(c"white");

    let text =
        l"""
            # Beispiel

             - Arbeitsplatz: _Autowerkstatt_
             - Angestellte: _Amanda_ und _Bobbl_
             - Werkzeuge: _Licht_ und _Hebebühne_
             - Zwei Arten von Problemen
                - Zugriff der Angestellten auf ein Werkzeug --> Einseitige Synchronisation
                - Zugriff der Angestellten auf mehrere Werkzeuge --> Deadlocks
        """ {
            text_color: c"white",
        };

slide one_sided_sync:
    page_number();
    let text =
        l"""
            # Einseitige Synchronisation

            1. Grundproblem
            2. Ständiges Überprüfen
            3. Getaktetes Überprüfen
            4. Überprüfen auf Anfrage
            5. Überprüfen auf Anfrage ohne Kenntnis anderer Threads
            6. Fazit zur einseitigen Synchronisation
        """;

element two_icons(icon1: String, icon2: String):
    let img1 =
        image(p'./pros-assets/{icon1}.png') {
        margin: {right: 50%},
        halign: HAlign.Right,
            valign: VAlign.Stretch,
        };
    let img2 =
        image(p'./pros-assets/{icon2}.png') {
        halign: HAlign.Left,
        margin: {left: 50%},
            valign: VAlign.Stretch,
        };

element four_icons(left: TwoIcons, right: TwoIcons):
    left = left {
        width: 50%,
        margin: {right: 50%},
        halign: HAlign.Right,
        valign: VAlign.Stretch,
    };

    right = right {
        width: 50%,
        halign: HAlign.Left,
        margin: {left: 50%},
        valign: VAlign.Stretch,
    };

slide base_problem:
    page_number();
    let title = l"# Einseitige Synchronisation: Grundproblem";
    let icons =
        grid("*", "*,*,*,*,*,*") {
            halign: HAlign.Stretch,
            valign: VAlign.Stretch,
            margin: {left: 0.10sw, top: 0.20sh, right: 0.10sw, bottom: 0.20sh},
        };
    // title.column_span = 3;
    let row =
        four_icons(two_icons("amanda", "light"), two_icons("wait", "bobbl")) {
            width: 0.30sw,
            halign: HAlign.Center,
        };
    icons.add(title);
    icons.add(row);
