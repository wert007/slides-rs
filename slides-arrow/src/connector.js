function create_line(options) {
    const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    line.setAttribute('stroke', options.color || 'black');
    line.setAttribute('stroke-width', options.width || '2');
    return line;
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

class SimpleConnector {
    constructor() {
        this.lines = [];
        window.addEventListener('resize', () => this.updateAll());
        window.addEventListener('scroll', () => this.updateAll(), true);
    }

    connect(el1, el2, options = {}) {
        console.log("options", options);
        const container = create_svg_canvas();
        if (options.parent) {
            options.parent.appendChild(container);
        } else {
            document.appendChild(container);
        }
        const line = { from: el1, to: el2, line: create_line(options), parent: options.parent };
        container.appendChild(line.line);
        this.lines.push(line);
        this.updateLine(line);
        return line;
    }

    updateLine(line) {
        const pos1 = line.from.getBoundingClientRect();
        const pos2 = line.to.getBoundingClientRect();

        const parent = line.parent?.getBoundingClientRect() ?? { x: 0, y: 0 };

        const x1 = pos1.left + pos1.width / 2 + window.scrollX - parent.x;
        const y1 = pos1.top + pos1.height / 2 + window.scrollY - parent.y;
        const x2 = pos2.left + pos2.width / 2 + window.scrollX - parent.x;
        const y2 = pos2.top + pos2.height / 2 + window.scrollY - parent.y;

        line.line.setAttribute('x1', x1);
        line.line.setAttribute('y1', y1);
        line.line.setAttribute('x2', x2);
        line.line.setAttribute('y2', y2);
    }

    updateAll() {
        for (const l of this.lines) {
            this.updateLine(l);
        }
    }
}

// // Example usage:
// document.addEventListener('DOMContentLoaded', () => {
//   const svg = document.getElementById('connector-svg');
//   const connector = new SimpleConnector(svg);

//   const el1 = document.getElementById('box1');
//   const el2 = document.getElementById('box2');

//   connector.connect(el1, el2, { color: 'red', width: 3 });
// });
