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
    // arrows.arrow(base.amanda, base.light, options | { from_pos: {x: 0.5, y: 0},line_kind: 'orthogonal'});
    arrows.arrow(base.amanda, base.bobbl, { color: c"#595959", from_pos: {x: 0.5, y: 0.0}, to_pos: {x: 0.5, y: 0.0}, label: "I am a arrow!", line_kind: 'orthogonal' });
//     arrows.arrow(base.bobbl, base.light, options | { startSocket: "bottom" });

slide deadlock_cyclic_wait:
    background = c"white";
    let title =
        l"""
            # Definition eines Deadlocks
             4. Zyklische Wartebedingung
        """ { halign: HAlign.Stretch, valign: VAlign.Top };
    let left =
        stackh([ icon("amanda"), icon("lift") ]) {
            halign: HAlign.Stretch,
            valign: VAlign.Center,
            height: 0.30sh,
            margin: { left: 0.10sw, right: 0.60sw, top: 0.30sh, bottom: 0.10sh },
        };
    let right =
        stackh([ icon("light"), icon("bobbl") ]) {
            halign: HAlign.Stretch,
            valign: VAlign.Center,
            height: 0.30sh,
            margin: { left: 0.60sw, right: 0.10sw, top: 0.30sh, bottom: 0.10sh },
        };
    arrows.arrow(left.children[0],
        right.children[0],
        {   color: c"#595959", label: "fordert an", from_pos: {x: 0.5, y: 0}, to_pos: {x: 0.5, y: 0}, line_kind: 'orthogonal' });
    arrows.arrow(right.children[1],
        left.children[1],
        {
            color: c"#595959",
            label: "fordert an",
            from_pos: {x: 0.5, y: 1},
            to_pos: {x: 0.5, y: 1},
            line_kind: 'orthogonal',
        });

