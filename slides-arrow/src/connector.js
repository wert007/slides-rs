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
    switch (line.kind) {
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
    if (line.options.end_tip) {
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

    switch (tip.kind) {
        case 'arrow':
            builder.line(3 * size, 0);             //    |
            builder.move(-3 * size, 3 * size);
            builder.line(3 * size, -3 * size);     //   \
            builder.line(-3 * size, -3 * size);            //   /
            break;
        case 'triangle':
            builder.line(0, size * 1.5);
            builder.line(size * 3, -size * 1.5);
            builder.line(-size * 3, -size * 1.5);
            builder.line(0, size * 1.5);
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
            builder.move(0, 3 * size);
            break;
        case 'diamond':
            // builder.line(size * 1.5, 0);
            // builder.line(0, 3 * size);
            // builder.line(-3 * size, 0);
            // builder.line(0, -3 * size);
            // builder.line(size * 1.5, 0);
            builder.move(3 * size, 3 * size);
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
    emit_tip(line.start_tip, positioning.rotation_start, line.options.width, builder);
    switch (line.kind) {
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
    emit_tip(line.end_tip, positioning.rotation_end, line.options.width, builder);

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
        if (delta.x > bestDelta.x) {
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

const ARROW_TIP = { kind: 'circle', is_filled: false };

function fill_options_with_default(options) {
    options.width ??= 2;
    options.color ??= 'black';
    options.line_kind ??= 'direct';
    options.end_tip ??= ARROW_TIP;
    options.from_pos ??= { x: 0.5, y: 0.5 };
    options.to_pos ??= { x: 0.5, y: 0.5 };
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
            kind: options.line_kind,
            from: { dom: from, pos: options.from_pos },
            to: { dom: to, pos: options.to_pos },
            dom: create_line(options),
            label_dom: create_label(options.label),
            parent: options.parent,
            options,
            start_tip: options.start_tip,
            end_tip: options.end_tip,
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

        // console.log("line", line);
        // const posFrom = line.from.dom.getBoundingClientRect();
        // const posFromRelative = line.from.pos;
        // const posTo = line.to.dom.getBoundingClientRect();
        // const posToRelative = line.to.pos;

        // const parent = line.parent?.getBoundingClientRect() ?? { x: 0, y: 0 };

        // const x1 = posFrom.left + posFrom.width * posFromRelative.x + window.scrollX - parent.x;
        // const y1 = posFrom.top + posFrom.height * posFromRelative.y + window.scrollY - parent.y;
        // const x2 = posTo.left + posTo.width * posToRelative.x + window.scrollX - parent.x;
        // const y2 = posTo.top + posTo.height * posToRelative.y + window.scrollY - parent.y;

        // const start = { x: x1, y: y1 };
        // const end = { x: x2, y: y2 };
        // let rotation_start, rotation_end;
        // switch (line.kind) {
        //     case 'direct': {
        //         const deltaX = x2 - x1;
        //         const deltaY = y2 - y1;
        //         const rotation = Math.atan2(deltaY, deltaX);
        //         rotation_start = rotation;
        //         rotation_end = rotation;
        //         if (line.end_tip) {
        //             end.x -= line.end_tip.size * Math.cos(rotation);
        //             end.y -= line.end_tip.size * Math.sin(rotation);
        //         }
        //         if (line.start_tip) {
        //             start.x -= line.start_tip.size * Math.cos(rotation);
        //             start.y -= line.start_tip.size * Math.sin(rotation);
        //         }
        //         line.dom[0].setAttribute('x1', start.x);
        //         line.dom[0].setAttribute('y1', start.y);
        //         line.dom[0].setAttribute('x2', end.x);
        //         line.dom[0].setAttribute('y2', end.y);
        //         if (line.label_dom) {
        //             line.label_dom.setAttribute('x', (start.x + end.x) / 2);
        //             line.label_dom.setAttribute('y', (start.y + end.y) / 2);

        //             line.label_dom.style.rotate = `${rotation}rad`;
        //             line.label_dom.style.translate = `0px -7px`;
        //         }
        //         break;
        //     }
        //     case 'orthogonal': {
        //         const points = calculate_orthogonal_connection(start, posFromRelative, end, posToRelative, undefined);
        //         if (points.length < 2) {
        //             break;
        //         }

        //         rotation_start = Math.atan2(points[1].y - points[0].y, points[1].x - points[0].x);
        //         rotation_end = Math.atan2(points.at(-1).y - points.at(-2).y, points.at(-1).x - points.at(-2).x);
        //         if (line.end_tip) {
        //             end.x -= line.end_tip.size * Math.cos(rotation_end);
        //             end.y -= line.end_tip.size * Math.sin(rotation_end);
        //             points[points.length - 1] = end;
        //         }
        //         if (line.start_tip) {
        //             start.x -= line.start_tip.size * Math.cos(rotation_start);
        //             start.y -= line.start_tip.size * Math.sin(rotation_start);
        //             points[0] = start;
        //         }
        //         if (points.length == line.dom.length + 1) {
        //             for (let i = 0; i < points.length - 1; i++) {
        //                 line.dom[i].setAttribute('x1', points[i].x);
        //                 line.dom[i].setAttribute('y1', points[i].y);
        //                 line.dom[i].setAttribute('x2', points[i + 1].x);
        //                 line.dom[i].setAttribute('y2', points[i + 1].y);
        //             }
        //         } else {
        //             for (const dom of line.dom) {
        //                 dom.remove();
        //             }
        //             line.dom = create_line(line.options, points);
        //             for (const dom of line.dom) {
        //                 line.container.appendChild(dom);
        //             }
        //         }
        //         if (line.label_dom) {
        //             const point = get_label_placement_for_orthogonal(points);
        //             line.label_dom.setAttribute('x', point.x);
        //             line.label_dom.setAttribute('y', point.y);
        //             line.label_dom.style.translate = `0px -7px`;
        //         }
        //         break;
        //     }
        // }
        // if (line.start_tip) {
        //     set_tip_position(line.start_tip, start, rotation_start, line.options);
        // }
        // if (line.end_tip) {
        //     set_tip_position(line.end_tip, end, rotation_end, line.options);
        // }
    }

    updateAll() {
        for (const l of this.lines) {
            this.updateLine(l);
        }
    }
}
