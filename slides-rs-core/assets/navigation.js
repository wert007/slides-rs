let slides;
let activeSlide;

function init() {
    slides = document.getElementsByClassName("slide");
    slides[0].classList.add("active");
    activeSlide = 0;
}

/**
 *
 * @param {KeyboardEvent} event
 */
function keydown(event) {
    switch (event.key) {
        case "ArrowUp":
        case "ArrowLeft":
            move_to_slide_relative(-1);
            break;
        case "ArrowDown":
        case "ArrowRight":
            move_to_slide_relative(1);
            break;
    }
}

function move_to_slide_relative(offset) {
    let target = activeSlide + offset;
    move_to_slide(target);
}

function move_to_slide(target) {
    if (target < 0 || target >= slides.length)
        return;
    slides[activeSlide].classList.remove("active");
    activeSlide = target;
    slides[activeSlide].classList.add("active");
}
