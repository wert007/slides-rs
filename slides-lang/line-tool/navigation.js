/**
 * @type Element[]
 */
let slides;
/**
 * @type number
 */
let activeSlide;
let currentStep = 0;
let currentStepCount = 0;
let stepReached = [];
var globals = {};

function init_navigation() {
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
    currentStepCount = slides[activeSlide].dataset.stepCount;
}

function getElementById(id) {
    return document.querySelector(`[data-element-id="${id}"]`);
}

/**
 *
 * @param {KeyboardEvent} event
 */
function keydown(event) {
    switch (event.key) {
        case "ArrowUp":
        case "ArrowLeft":
            change_step_relative(-1);
            break;
        case "ArrowDown":
        case "ArrowRight":
            change_step_relative(1);
            break;
    }
}

function change_step_relative(offset) {
    let target = currentStep + offset;
    if (target < 0) {
        move_to_slide_relative(-1);
    } else if (target > currentStepCount) {
        move_to_slide_relative(1);
    } else {
        const currentSlideId = slides[activeSlide].id;
        for (let animation of stepReached) {
            if (animation.slideId != currentSlideId) continue;
            if (animation.step == target) {
                animation.trigger();
            } else if (animation.step - 1 == target) {
                animation.reverse();
            }
        }
        currentStep = target;
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
    currentStepCount = slides[activeSlide].dataset.stepCount;
    currentStep = 0;
    onSlideChange();
}

