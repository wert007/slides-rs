/**
 * @type Element[]
 */
let slides;
/**
 * @type number
 */
let activeSlide;

function init() {
    slides = document.getElementsByClassName("slide");
    var slide_id = window.location.hash.slice(1);
    activeSlide = 0;
    for (let i = 0; i < slides.length; i++) {
        if (slides[i].id == slide_id) {
            activeSlide = i;
            break;
        }
    }
    slides[activeSlide].classList.add("active");
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
    window.location.hash = slides[activeSlide].id;
}
