import module"arrows";

element icon(name: String):
    let icon = image(p'./pros-assets/{name}.png') { width: 100%, height: 100% };

element two_icons(icon1: String, icon2: String):
    let body = stackh([ icon(icon1), icon(icon2) ]) { width: 100%, height: 100% };

element empty():
    // Not sure how well we support empty statements
    let i = 42;

element deadlock_icons():
    let size = 20%;
    let amanda =
        icon('amanda') {
            width: size * 0.5,
            height: size,
            halign: HAlign.Left,
            valign: VAlign.Center,
        };
    let bobbl =
        icon('bobbl') {
            width: size * 0.5,
            height: size,
            halign: HAlign.Right,
            valign: VAlign.Center,
        };
    let light =
        icon('light') {
            width: size * 0.5,
            height: size,
            halign: HAlign.Center,
            valign: VAlign.Bottom,
        };
    let lift =
        icon('lift') { width: size * 0.5, height: size, halign: HAlign.Center, valign: VAlign.Top };

// members = {
//     amanda: amanda,
//     bobbl: bobbl,
//     light: light,
//     lift: lift,
// };
slide deadlocks_problem:
background = c"white";
    let title =
        l"# Deadlocks: Grundproblem" {
            halign: HAlign.Stretch,
            margin: { left: 0.10sw, right: 0.10sw, top: 0.10sh, bottom: 0.10sh },
        };
    let base =
        deadlock_icons() {
            halign: HAlign.Stretch,
            valign: VAlign.Stretch,
            margin: { left: 0.10sw, right: 0.10sw, top: 0.40sh, bottom: 0.10sh },
        };
    // let position = positionInside(a, 0.0, 0.5);
    // let amanda = base.amanda;
    let options = { color: c"#595959" };
    arrows.arrow(base.amanda, base.light, options | { from_pos: {x: 0.5, y: 0},line_kind: 'orthogonal'});
    arrows.arrow(base.amanda, base.lift, options | { from_pos: {x: 0.7, y: 0.2}, to_pos: {x: 0, y: 0.5} });
//     arrows.arrow(base.bobbl, base.light, options | { startSocket: "bottom" });
//     arrows.arrow(base.bobbl, base.lift, options | { startSocket: "top" });
//     let wait_amanda =
//         icon('wait') {
//             width: 5%,
//             height: 10%,
//             valign: VAlign.Bottom,
//             halign: HAlign.Left,
//             margin: { bottom: 0.10sh, left: 0.25sw },
//             background: c"#f3f3f3",
//         };
//     let wait_bobbl =
//         icon('wait') {
//             width: 5%,
//             height: 10%,
//             halign: HAlign.Right,
//             valign: VAlign.Top,
//             margin: { top: 0.40sh, right: 0.25sw },
//             background: c"#f3f3f3",
//         };

// slide deadlock_definition:
//     page_number();
//     let content =
//         l"""
//             # Definition eines Deadlocks

//             Benötigte Bedingungen:

//             1. Bedingung des wechselseitigen Ausschlusses
//             2. Belegungs- und Wartebedingung
//             3. Ununterbrechbarkeitsbedingung
//             4. Zyklische Wartebedingung
//         """;

// element deadlock_unique_access_yes():
//     let content =
//         stackv(
//             [
//                 l"Möglich" { text_align: TextAlign.Center },
//                 stackh([ icon("amanda"), empty(), icon("light"), icon("bobbl") ]) { height: 25% },
//                 l"oder" { text_align: TextAlign.Center },
//                 stackh([ icon("amanda"), icon("light"), empty(), icon("bobbl") ]) { height: 25% },
//             ]) { height: 100%, width: 100% };

// element crossed(inner: Element):
//     inner = inner { halign: HAlign.Stretch, valign: VAlign.Stretch };
//     let line1 =
//         empty() {
//             width: 0.005sw,
//             halign: HAlign.Center,
//             valign: VAlign.Stretch,
//             // margin: {top: 0.3sh, bottom: 0.1sh},
//             rotate: 45,
//             background: c"red",
//         };
//     let line2 =
//         empty() {
//             width: 0.005sw,
//             halign: HAlign.Center,
//             valign: VAlign.Stretch,
//             // margin: {top: 0.3sh, bottom: 0.1sh},
//             rotate: 0 - 45,
//             background: c"red",
//         };

// element deadlock_unique_access_no():
//     let content =
//         stackv(
//             [
//                 l"Nicht möglich" { text_align: TextAlign.Center },
//                 crossed(stackh([ icon("amanda"), icon("light"), icon("bobbl") ])),
//             ]) { height: 100%, width: 100% };

// slide deadlock_unique_access:
//     page_number();
//     let title =
//         l"""
//             # Definition eines Deadlocks
//              1. Bedingung des wechselseitigen Ausschlusses
//         """ { halign: HAlign.Stretch, valign: VAlign.Top };
//     let seperator =
//         empty() {
//             width: 0.005sw,
//             halign: HAlign.Center,
//             valign: VAlign.Stretch,
//             margin: { top: 0.3sh, bottom: 0.1sh },
//             background: c"#78909c",
//         };
//     let left =
//         deadlock_unique_access_yes() {
//             halign: HAlign.Stretch,
//             valign: VAlign.Stretch,
//             margin: { left: 0.10sw, right: 0.60sw, top: 0.30sh, bottom: 0.10sh },
//         };
//     let right =
//         deadlock_unique_access_no() {
//             halign: HAlign.Stretch,
//             valign: VAlign.Stretch,
//             margin: { left: 0.60sw, right: 0.10sw, top: 0.30sh, bottom: 0.10sh },
//         };

// slide deadlock_additional_resources:
//     page_number();
//     let size = 0.3sh;
//     let title =
//         l"""
//             # Definition eines Deadlocks
//              2. Belegungs- und Wartebedingung
//         """ { halign: HAlign.Stretch, valign: VAlign.Top };
//     let left =
//         two_icons("lift", "amanda") {
//             halign: HAlign.Left,
//             valign: VAlign.Center,
//             width: size * 2.0,
//             height: size,
//         };
//     let right =
//         icon("light") { halign: HAlign.Right, valign: VAlign.Center, width: size, height: size };
//     arrows.arrow(left, right, { color: c"#595959", middle_label: "fordert zusätzlich an" });

// slide deadlock_no_release:
//     page_number();
//     let title =
//         l"""
//             # Definition eines Deadlocks
//             3. Ununterbrechbarkeitsbedingung
//         """ { halign: HAlign.Stretch, valign: VAlign.Top };
//     let seperator =
//         empty() {
//             width: 0.005sw,
//             halign: HAlign.Center,
//             valign: VAlign.Stretch,
//             margin: { top: 0.3sh, bottom: 0.1sh },
//             background: c"#78909c",
//         };
//     let left_amanda = icon("amanda");
//     let right_amanda = icon("amanda");
//     let left_lift = icon("lift");
//     let right_lift = icon("lift");
//     let left =
//         stackh([ left_amanda, empty(), empty(), left_lift ]) {
//             halign: HAlign.Stretch,
//             valign: VAlign.Stretch,
//             margin: { left: 0.10sw, right: 0.60sw, top: 0.30sh, bottom: 0.10sh },
//         };
//     let right =
//         crossed(stackh([ right_amanda, empty(), empty(), right_lift ])) {
//             halign: HAlign.Stretch,
//             valign: VAlign.Stretch,
//             margin: { left: 0.60sw, right: 0.10sw, top: 0.30sh, bottom: 0.10sh },
//         };
//     arrows.arrow(left_amanda, left_lift, { color: c"#595959", middle_label: "gibt zurück" });
//     arrows.arrow(right_amanda, right_lift, { color: c"#595959", middle_label: "wird entzogen" });

// slide deadlock_cyclic_wait:
//     page_number();
//     let title =
//         l"""
//             # Definition eines Deadlocks
//              4. Zyklische Wartebedingung
//         """ { halign: HAlign.Stretch, valign: VAlign.Top };
//     let left =
//         stackh([ icon("amanda"), icon("lift") ]) {
//             halign: HAlign.Stretch,
//             valign: VAlign.Center,
//             height: 0.30sh,
//             margin: { left: 0.10sw, right: 0.60sw, top: 0.30sh, bottom: 0.10sh },
//         };
//     let right =
//         stackh([ icon("light"), icon("bobbl") ]) {
//             halign: HAlign.Stretch,
//             valign: VAlign.Center,
//             height: 0.30sh,
//             margin: { left: 0.60sw, right: 0.10sw, top: 0.30sh, bottom: 0.10sh },
//         };
//     arrows.arrow(left.children[0],
//         right.children[0],
//         {   color: c"#595959", middle_label: "fordert an", start_socket: "top", end_socket: "top" });
//     arrows.arrow(right.children[1],
//         left.children[1],
//         {
//             color: c"#595959",
//             middle_label: "fordert an",
//             start_socket: "bottom",
//             end_socket: "bottom",
//         });

