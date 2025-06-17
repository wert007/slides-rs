class SvgPathCreator {
    constructor(x, y) {
        this.buffer = [`M ${x} ${y}`];
        this.rotation = 0;
    }

    rotate(rotation) {
        this.rotation += rotation;
    }

    setRotation(rotation) {
        this.rotation = rotation;
    }

    transpose(x, y) {
        return [Math.cos(this.rotation) * x - Math.sin(this.rotation) * y, Math.sin(this.rotation) * x + Math.cos(this.rotation) * y];
    }

    toSvgPath() {
        return this.buffer.join(' ');
    }

    move(x_raw, y_raw) {
        const [x, y] = this.transpose(x_raw, y_raw)
        this.buffer.push(`m ${x} ${y}`);
    }

    line(x_raw, y_raw) {
        const [x, y] = this.transpose(x_raw, y_raw)
        this.buffer.push(`l ${x} ${y}`);
    }

    arc(radius_x, radius_y, rotation, large_arc_raw, sweep_raw, x_raw, y_raw) {
        const [x, y] = this.transpose(x_raw, y_raw);
        const large_arc = large_arc_raw ? 1 : 0;
        const sweep = sweep_raw ? 1 : 0;
        this.buffer.push(`a ${radius_x} ${radius_y} ${rotation} ${large_arc} ${sweep} ${x} ${y}`);
    }

    lineAbsolut(x, y) {
        this.buffer.push(`L ${x} ${y}`);
    }
}

function calculate_orthogonal_connection(start, startRel, end, endRel, lastWasVertical) {
    const direction = { x: Math.sign(startRel.x - 0.5), y: Math.sign(startRel.y - 0.5) };
    const delta = { x: end.x - start.x, y: end.y - start.y };
    console.log(start, delta, end);
    if (Math.abs(delta.x) < 0.1 && Math.abs(delta.y) < 0.1) {

        return [end];
    }
    if (direction.x == 0 && direction.y == 0) {
        if (Math.abs(delta.x) > Math.abs(delta.y)) {
            if (lastWasVertical == false) {
                direction.y = Math.sign(delta.y);
            } else {
                direction.x = Math.sign(delta.x);
            }
        } else {
            if (lastWasVertical == true) {
                direction.x = Math.sign(delta.x);
            } else {
                direction.y = Math.sign(delta.y);
            }
        }
    }
    if (direction.x != 0 && direction.y != 0) {
        if (Math.abs(startRel.x) > Math.abs(startRel.y)) {
            if (lastWasVertical == false) {
                direction.x = 0;
            } else {
                direction.y = 0;
            }
        } else {
            if (lastWasVertical == true) {
                direction.y = 0;
            } else {
                direction.x = 0;
            }
        }
    }
    const deltaLength = delta.x * delta.x + delta.y * delta.y;
    const minDistance = Math.sqrt(deltaLength) * 0.10;
    delta.x = Math.max(Math.abs(delta.x), minDistance);
    delta.y = Math.max(Math.abs(delta.y), minDistance);
    let point = { x: delta.x * direction.x + start.x, y: delta.y * direction.y + start.y };
    const newDelta = { x: end.x - point.x, y: end.y - point.y };
    if (deltaLength <= newDelta.x * newDelta.x + newDelta.y * newDelta.y) {
        point = { x: minDistance * direction.x + start.x, y: minDistance * direction.y + start.y };
    }
    const isVertical = direction.y != 0 && direction.x == 0;
    return [start, ...calculate_orthogonal_connection(point, { x: 0.5, y: 0.5 }, end, endRel, isVertical)];
}

function calculate_positioning(line) {
    const posFrom = line.from.dom.getBoundingClientRect();
    const posFromRelative = line.from.pos;
    const posTo = line.to.dom.getBoundingClientRect();
    const posToRelative = line.to.pos;

    const parent = line.parent?.getBoundingClientRect() ?? { x: 0, y: 0 };

    const x1 = posFrom.left + posFrom.width * posFromRelative.x + window.scrollX - parent.x;
    const y1 = posFrom.top + posFrom.height * posFromRelative.y + window.scrollY - parent.y;
    const x2 = posTo.left + posTo.width * posToRelative.x + window.scrollX - parent.x;
    const y2 = posTo.top + posTo.height * posToRelative.y + window.scrollY - parent.y;

    const start = { x: x1, y: y1 };
    const end = { x: x2, y: y2 };
    const positioning = { start, end, rotation_start: 0, rotation_end: 0, points: [start, end] };
    switch (line.kind.toLowerCase()) {
        case 'direct': {
            const deltaX = x2 - x1;
            const deltaY = y2 - y1;
            const rotation = Math.atan2(deltaY, deltaX);
            positioning.rotation_start = rotation;
            positioning.rotation_end = rotation;
            break;
        }
        case 'orthogonal': {
            const points = calculate_orthogonal_connection(start, posFromRelative, end, posToRelative, undefined);
            positioning.points = points;
            if (points.length < 2) {
                break;
            }

            positioning.rotation_start = Math.atan2(points[1].y - points[0].y, points[1].x - points[0].x);
            positioning.rotation_end = Math.atan2(points.at(-1).y - points.at(-2).y, points.at(-1).x - points.at(-2).x);
            break;
        }
    }
    if (line.options.endtip) {
        const tip_size = line.options.width * 3;
        positioning.end.x -= Math.cos(positioning.rotation_end) * tip_size;
        positioning.end.y -= Math.sin(positioning.rotation_end) * tip_size;
        positioning.points[positioning.points.length - 1] = positioning.end;
    }
    return positioning;
}

function emit_tip(tip, rotation, size, builder) {
    if (tip == undefined) return;
    builder.rotate(rotation);
    const direction = tip.flip ? -1 : 1;

    switch (tip.kind.toLowerCase()) {
        case 'arrow':
            if (tip.flip) {
                builder.move(3 * size, 0);
            }
            builder.line(direction * 3 * size, 0);             //    |
            builder.move(direction * -3 * size, 3 * size);
            builder.line(direction * 3 * size, -3 * size);     //   \
            builder.line(direction * -3 * size, -3 * size);            //   /
            builder.move(0, 3 * size);
            break;
        case 'triangle':
            if (tip.flip) {
                builder.move(3 * size, 0);
            }
            builder.line(0, size * 1.5);
            builder.line(direction * size * 3, -size * 1.5);
            builder.line(direction * -size * 3, -size * 1.5);
            builder.line(0, size * 1.5);
            if (!tip.flip) {
                builder.move(3 * size, 0);
            }
            break;
        case 'circle':
            const radius = 1.5 * size;
            builder.move(2 * radius, 0);
            builder.arc(radius, radius, 0, 1, 0, -2 * radius, 0);
            builder.arc(radius, radius, 0, 1, 0, 2 * radius, 0);
            break;
        case 'square':
            builder.line(0, size * 1.5);
            builder.line(3 * size, 0);
            builder.line(0, -3 * size);
            builder.line(-3 * size, 0);
            builder.line(0, size * 1.5);
            builder.move(3 * size, 0);
            break;
        case 'diamond':
            builder.line(size * 1.5, size * 1.5);
            builder.line(size * 1.5, -size * 1.5);
            builder.line(-size * 1.5, -size * 1.5);
            builder.line(-size * 1.5, size * 1.5);
            builder.move(3 * size, 0);
            break;
        case 'none':
            break;
        default:
            throw new Error("")
    }
    builder.rotate(-rotation);
}

function turnLineIntoPath(line) {
    const positioning = calculate_positioning(line);
    let builder = new SvgPathCreator(positioning.start.x, positioning.start.y);
    emit_tip(line.starttip, positioning.rotation_start, line.options.width, builder);
    switch (line.kind.toLowerCase()) {
        case 'direct':
            builder.lineAbsolut(positioning.end.x, positioning.end.y);
            break;
        case 'orthogonal':
            for (let i = 1; i < positioning.points.length; i++) {
                const point = positioning.points[i];
                builder.lineAbsolut(point.x, point.y);
            }
            break;
        default: break;
    }
    emit_tip(line.endtip, positioning.rotation_end, line.options.width, builder);

    return builder.toSvgPath();
}

function get_label_placement_for_orthogonal(points) {
    let minX = Number.MAX_SAFE_INTEGER;
    let maxX = Number.MIN_SAFE_INTEGER;
    let bestDelta = { x: 0, y: 0 };
    let point = undefined;
    for (let i = 0; i < points.length - 1; i++) {
        const delta = {
            x: points[i + 1].x - points[i].x,
            y: points[i + 1].y - points[i].y,
        };
        if (delta.x == 0) continue;
        if (minX > points[i].x) minX = points[i].x;
        if (maxX < points[i].x) maxX = points[i].x;
        if (minX > points[i + 1].x) minX = points[i + 1].x;
        if (maxX < points[i + 1].x) maxX = points[i + 1].x;
        if (Math.abs(delta.x) > Math.abs(bestDelta.x)) {
            bestDelta = delta;
            point = { x: points[i].x + delta.x * 0.5, y: points[i].y + delta.y * 0.5 };
        }
    }
    return point ?? points[0];
}

function create_line(options) {
    const line = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    line.setAttribute('stroke', options.color);
    line.setAttribute('fill', 'transparent');
    line.setAttribute('stroke-width', options.width);
    return line;
}

function create_label(text) {
    if (text == undefined) return undefined;
    const dom = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    dom.textContent = text;
    dom.setAttribute('text-anchor', 'middle');
    dom.style['transform-box'] = 'border-box';
    dom.style['transform-origin'] = 'center';
    return dom;
}

function create_svg_canvas() {
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.style.width = "100%";
    svg.style.height = "100%";
    svg.style.position = "absolute";
    svg.style.left = "0";
    svg.style.top = "0";
    return svg;
}

const ARROW_TIP = { kind: 'arrow', filled: false, flip: false };
const NO_TIP = { kind: 'none', filled: false, flip: false };

function fill_options_with_default(options) {
    options.width ??= 2;
    options.color ??= 'black';
    options.kind ??= 'Direct';
    options.starttip ??= { ...NO_TIP };
    options.starttip.flip = !options.starttip.flip;
    options.endtip ??= ARROW_TIP;
    options.startposrel ??= { x: 0.5, y: 0.5 };
    options.endposrel ??= { x: 0.5, y: 0.5 };
}

class SimpleConnector {
    constructor() {
        this.lines = [];
        window.addEventListener('resize', () => this.updateAll());
        window.addEventListener('scroll', () => this.updateAll(), true);
    }

    connect(from, to, options = {}) {
        const container = create_svg_canvas();
        if (options.parent) {
            options.parent.appendChild(container);
        } else {
            document.appendChild(container);
        }
        fill_options_with_default(options);
        const line = {
            kind: options.kind,
            from: { dom: from, pos: options.startposrel },
            to: { dom: to, pos: options.endposrel },
            dom: create_line(options),
            label_dom: create_label(options.label),
            parent: options.parent,
            options,
            starttip: options.starttip,
            endtip: options.endtip,
            container,
        };
        line.dom.setAttribute('d', turnLineIntoPath(line));
        container.appendChild(line.dom);
        if (line.label_dom) {
            container.appendChild(line.label_dom);
        }
        this.lines.push(line);
        this.updateLine(line);
        return line;
    }

    updateLine(line) {
        line.dom.setAttribute('d', turnLineIntoPath(line));

        const positioning = calculate_positioning(line);
        switch (line.kind.toLowerCase()) {
            case 'direct': {
                const deltaX = positioning.end.x - positioning.start.x;
                const deltaY = positioning.end.y - positioning.start.y;
                const rotation = Math.atan2(deltaY, deltaX);
                if (line.label_dom) {
                    line.label_dom.setAttribute('x', (positioning.start.x + positioning.end.x) / 2);
                    line.label_dom.setAttribute('y', (positioning.start.y + positioning.end.y) / 2);

                    line.label_dom.style.rotate = `${rotation}rad`;
                    line.label_dom.style.translate = `0px -7px`;
                }
                break;
            }
            case 'orthogonal': {
                if (line.label_dom) {
                    const point = get_label_placement_for_orthogonal(positioning.points);
                    line.label_dom.setAttribute('x', point.x);
                    line.label_dom.setAttribute('y', point.y);
                    line.label_dom.style.translate = `0px -7px`;
                }
                break;
            }
        }
    }

    updateAll() {
        for (const l of this.lines) {
            this.updateLine(l);
        }
    }
}
