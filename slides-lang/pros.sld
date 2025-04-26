import module"arrows";
// import p"./pros-assets/leader-line.head.html";
// import p"./pros-assets/leader-line-arrow.init.js";

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
    padding = { top: 0.2sh, right: 0.1sw, bottom: 0.2sh, left: 0.1sw, };

element image_with_caption(img: Image, caption: String):
    img = img { object_fit: ObjectFit.Cover, valign: VAlign.Stretch, halign: HAlign.Stretch, }
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
            margin: { bottom: 0.50sh, },
        }
    let subtitle =
        l"## Nach „Parallele Programmierung spielend gelernt mit dem ‚Java-Hamster-Modell‘“ im Proseminar Praktische Informatik" {
            text_align: TextAlign.Center,
            font_size: 1.3,
            valign: VAlign.Top,
            halign: HAlign.Stretch,
            text_color: c"#595959",
            margin: { top: 0.50sh, left: 0.15sw, right: 0.15sw, },
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
        """ { };

slide example:
    let bg =
        image_with_caption(
            image(p"./pros-assets/car-repair.jpg") { filter: brightness(0.4), },
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
        """ { text_color: c"white", };

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

element icon(name: String):
    let icon = image(p'./pros-assets/{name}.png') { width: 100%, height: 100%, };

element two_icons(icon1: String, icon2: String, size: StyleUnit = 30%):
    let body =
        stackh([ icon(icon1) { width: size, }, icon(icon2) { width: size, }, ]) {
            width: 100%,
            height: 100%,
        };

slide base_problem:
    page_number();
    let title = l"# Einseitige Synchronisation: Grundproblem";
    let amanda =
        stackv(
            [
                two_icons("amanda", "light", 50%),
                l"Amanda hat das Licht" { text_align: TextAlign.Center, },
            ]);
    let bobbl =
        stackv(
            [
                two_icons("wait", "bobbl", 50%),
                l"Bobbl wartet auf das Licht" { text_align: TextAlign.Center, },
            ]);
    let row = stackh([ amanda, bobbl ]) { height: 50%, };
    let content =
        stackv([ title, row ]) {
            halign: HAlign.Stretch,
            valign: VAlign.Stretch,
            margin: { left: 0.10sw, top: 0.20sh, right: 0.10sw, bottom: 0.20sh, },
        };

element empty():
    // Not sure how well we support empty statements
    let i = 42;

slide continous_checking:
    steps = 3;
    page_number();
    let title = l"# Ständiges Überprüfen";
    let row1 =
        stackh([ two_icons('amanda', 'light'), two_icons('question_mark', 'bobbl') ]) {
            height: 30%,
            animations: [ showAfterStep(1) ],
        };
    let row2 =
        stackh([ two_icons('amanda', 'light'), two_icons('question_mark', 'bobbl') ]) {
            height: 30%,
            animations: [ showAfterStep(2) ],
        };
    let row3 =
        stackh([ stackh([ icon('amanda'), empty() ]), two_icons('light', 'bobbl') ]) {
            height: 30%,
            animations: [ showAfterStep(3) ],
        };
    let content =
        stackv([ title, row1, row2, row3 ]) {
            halign: HAlign.Stretch,
            valign: VAlign.Stretch,
            margin: { left: 0.10sw, top: 0.20sh, right: 0.10sw, bottom: 0.20sh, },
        };

slide occasional_checking_1:
    steps = 3;
    page_number();
    let title = l"# Getaktetes Überprüfen";
    let row1 =
        stackh([ two_icons('amanda', 'light'), two_icons('question_mark', 'bobbl') ]) {
            height: 30%,
            animations: [ showAfterStep(1) ],
        };
    let row2 =
        stackh([ two_icons('amanda', 'light'), two_icons('wait', 'bobbl') ]) {
            height: 30%,
            animations: [ showAfterStep(2) ],
        };
    let row3 =
        stackh([ two_icons('amanda', 'light'), two_icons('question_mark', 'bobbl') ]) {
            height: 30%,
            animations: [ showAfterStep(3) ],
        };
    let content =
        stackv([ title, row1, row2, row3 ]) {
            halign: HAlign.Stretch,
            valign: VAlign.Stretch,
            margin: { left: 0.10sw, top: 0.20sh, right: 0.10sw, bottom: 0.20sh, },
        };

slide occasional_checking_2:
    steps = 2;
    page_number();
    let title = l"# Getaktetes Überprüfen";
    let row1 =
        stackh([ two_icons('amanda', 'light'), two_icons('question_mark', 'bobbl') ]) {
            height: 30%,
        };
    let row2 =
        stackh([ icon('amanda'), icon('light'), two_icons('wait', 'bobbl') ]) {
            height: 30%,
        };
    let row3 =
        stackh([ stackh([ icon('amanda'), empty() ]), two_icons('light', 'bobbl') ]) {
            height: 30%,
            animations: [ showAfterStep(1) ],
        };
    let content =
        stackv([ title, row1, row2, row3 ]) {
            halign: HAlign.Stretch,
            valign: VAlign.Stretch,
            margin: { left: 0.10sw, top: 0.20sh, right: 0.10sw, bottom: 0.20sh, },
        };

slide checking_when_requested:
    page_number();
    let title = l"# Überprüfen auf Anfrage";
    let row1 =
        stackh([ two_icons('amanda', 'light'), two_icons('question_mark', 'bobbl') ]) {
            height: 30%,
        };
    let row2 =
        stackh([ two_icons('amanda', 'light'), two_icons('wait', 'bobbl') ]) {
            height: 30%,
        };
    let row3 =
        stackh([ empty(),empty(), icon('amanda') { height: 100% }, icon('light') { height: 100% }, icon('bobbl') { height: 100% }, empty() ]) {
            height: 30%,
        };
    let content =
        stackv([ title, row1, row2, row3 ]) {
            halign: HAlign.Stretch,
            valign: VAlign.Stretch,
            margin: { left: 0.10sw, top: 0.20sh, right: 0.10sw, bottom: 0.20sh, },
        };

slide checking_when_requested_without_thread_knowledge:
    page_number();
    let title = l"# Überprüfen auf Anfrage ohne Kenntnis anderer Threads";
    let row1 =
        stackh([ two_icons('amanda', 'light'), two_icons('question_mark', 'bobbl') ]) {
            height: 30%,
        };
    let row2 =
        stackh([ two_icons('amanda', 'light'), two_icons('wait', 'bobbl') ]) {
            height: 30%,
        };
    let row3 =
        stackh([ icon('amanda') { height: 100% }, empty(),empty(), icon('light') { height: 100% }, icon('bobbl') { height: 100% }, empty(), ]) {
            height: 30%,
        };
    let content =
        stackv([ title, row1, row2, row3 ]) {
            halign: HAlign.Stretch,
            valign: VAlign.Stretch,
            margin: { left: 0.10sw, top: 0.20sh, right: 0.10sw, bottom: 0.20sh, },
        };

slide resume_one_sided:
    page_number();
    let title = l"# Fazit zur einseitigen Synchronisation" {
        halign: HAlign.Stretch,
        valign: VAlign.Center,
        font_size: 3.0,
        text_align: TextAlign.Center,
    };

slide deadlocks:
    page_number();
    let text = l"""
        # Deadlocks

        1. Grundproblem
        2. Definition eines Deadlocks
        3. Präventive Deadlockverhinderung
        4. Deadlockverhinderung zur Laufzeit
        5. Deadlockerkennung zur Laufzeit
        6. Livelocks
        7. Fazit zu Deadlocks
    """;

element deadlock_icons():
    let size = 20%;
    let amanda = icon('amanda') {
        width: size,
        height: size,
        halign: HAlign.Left,
        valign: VAlign.Center,
    };
    let bobbl = icon('bobbl') {
        width: size,
        height: size,
        halign: HAlign.Right,
        valign: VAlign.Center,
    };
    let light = icon('light') {
        width: size,
        height: size,
        halign: HAlign.Center,
        valign: VAlign.Bottom,
    };
    let lift = icon('lift') {
        width: size,
        height: size,
        halign: HAlign.Center,
        valign: VAlign.Top,
    };
    members = {
        amanda: amanda,
        bobbl: bobbl,
        light: light,
        lift: lift,
    };

slide deadlocks_problem:
    page_number();
    let base = deadlock_icons() {
        halign: HAlign.Stretch,
        valign: VAlign.Stretch,
        margin: {
            left: 0.10sw,
            right: 0.10sw,
            top: 0.10sh,
            bottom: 0.10sh,
        }
    };
    // let position = positionInside(a, 0.0, 0.5);
    // let amanda = base.amanda;
    let options = {
        color: c"#595959",
        path: "grid",
    };
    arrows.arrow(base.amanda, base.light, options);
    arrows.arrow(base.amanda, base.lift, options);
    arrows.arrow(base.bobbl, base.light, options);
    arrows.arrow(base.bobbl, base.lift, options);
