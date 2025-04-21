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
    currentStepCount = slides[activeSlide].dataset.stepCount;
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
}

stepReached.push({
 slideId: 'continous_checking',
 step: 1,
 trigger: () => {
    document.getElementById('continous_checking-content-flex-59').classList.remove('invisible');
},
 reverse: () => {
    document.getElementById('continous_checking-content-flex-59').classList.add('invisible');
}
});
stepReached.push({
 slideId: 'continous_checking',
 step: 2,
 trigger: () => {
    document.getElementById('continous_checking-content-flex-72').classList.remove('invisible');
},
 reverse: () => {
    document.getElementById('continous_checking-content-flex-72').classList.add('invisible');
}
});
stepReached.push({
 slideId: 'continous_checking',
 step: 3,
 trigger: () => {
    document.getElementById('continous_checking-content-flex-83').classList.remove('invisible');
},
 reverse: () => {
    document.getElementById('continous_checking-content-flex-83').classList.add('invisible');
}
});
stepReached.push({
 slideId: 'occasional_checking_1',
 step: 1,
 trigger: () => {
    document.getElementById('occasional_checking_1-content-flex-100').classList.remove('invisible');
},
 reverse: () => {
    document.getElementById('occasional_checking_1-content-flex-100').classList.add('invisible');
}
});
stepReached.push({
 slideId: 'occasional_checking_1',
 step: 2,
 trigger: () => {
    document.getElementById('occasional_checking_1-content-flex-113').classList.remove('invisible');
},
 reverse: () => {
    document.getElementById('occasional_checking_1-content-flex-113').classList.add('invisible');
}
});
stepReached.push({
 slideId: 'occasional_checking_1',
 step: 3,
 trigger: () => {
    document.getElementById('occasional_checking_1-content-flex-126').classList.remove('invisible');
},
 reverse: () => {
    document.getElementById('occasional_checking_1-content-flex-126').classList.add('invisible');
}
});
stepReached.push({
 slideId: 'occasional_checking_2',
 step: 1,
 trigger: () => {
    document.getElementById('occasional_checking_2-content-flex-165').classList.remove('invisible');
},
 reverse: () => {
    document.getElementById('occasional_checking_2-content-flex-165').classList.add('invisible');
}
});
