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
    padding = {top: 10%, right: 10%, bottom: 10%, left: 10%};

element image_with_caption(img: Image, caption: String):
    img = img {
        object_fit: ObjectFit.Cover,
        valign: VAlign.Stretch,
        halign: HAlign.Stretch,
    }
    let lbl = label(caption) {
        valign: VAlign.Bottom,
        halign: HAlign.Right,
        text_color: c"white",
    }
    valign = VAlign.Stretch;
    halign = HAlign.Stretch;

styling page_number(Label):
    valign = VAlign.Bottom;
    halign = HAlign.Center;
    text_align = TextAlign.Center;

slide code:
    let bg = image_with_caption(image(p"./pros-assets/code.jpg"), "Image by Christopher Kuszajewski from Pixabay");

slide threads:
    let bg = image_with_caption(image(p"./pros-assets/threads.jpg"), "Bild von Myriams-Fotos auf Pixabay");

slide frozen:
    let bg = image_with_caption(image(p"./pros-assets/frozen.jpg"), "Bild von adege auf Pixabay");

slide title:
    let title = l"# Einseitige Synchronisation und Deadlocks" {
        text_align: TextAlign.Center,
        font_size: 52pt,
        valign: VAlign.Bottom,
        halign: HAlign.Stretch,
        margin: {
            bottom: 50%,
        }
    }
    let subtitle = l"## Nach „Parallele Programmierung spielend gelernt mit dem ‚Java-Hamster-Modell‘“ im Proseminar Praktische Informatik" {
        text_align: TextAlign.Center,
        font_size: 28pt,
        valign: VAlign.Top,
        halign: HAlign.Stretch,
        margin: {
            top: 50%,
            left: 15%,
            right: 15%,
        }
    }


slide toc:
    let toc = l"""
        # Übersicht

        1. Übersicht
        2. Beispiel
        3. Einseitige Synchronisation
        4. Deadlocks
    """ {
    };

    let page = l"1" { styles: [page_number]};

slide example:

    let text = l"""
        # Beispiel

         - Arbeitsplatz: _Autowerkstatt_
         - Angestellte: _Amanda_ und _Bobbl_
         - Werkzeuge: _Licht_ und _Hebebühne_
         - Zwei Arten von Problemen
            - Zugriff der Angestellten auf ein Werkzeug --> Einseitige Synchronisation
            - Zugriff der Angestellten auf mehrere Werkzeuge --> Deadlocks
    """ { text_color: c"white"};

    let bg = image_with_caption(image(p"./pros-assets/car-repair.jpg") {
        filter: brightness(0.4),
    }, "Bild von emkanicepic auf Pixabay");

    let page = l"2" { text_color: c"white", styles: [page_number]};

slide one_sided_sync:
    let text = l"""
        # Einseitige Synchronisation

        1. Grundproblem
        2. Ständiges Überprüfen
        3. Getaktetes Überprüfen
        4. Überprüfen auf Anfrage
        5. Überprüfen auf Anfrage ohne Kenntnis anderer Threads
        6. Fazit zur einseitigen Synchronisation
    """;

    let page = l"3" { text_color: c"white", styles: [page_number]};
