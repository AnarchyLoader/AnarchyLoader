const HackerText = (function () {
    'use strict';

    const DEFAULT_OPTIONS = {
        iterations: 7,
        speed: 30, // Now correctly interpreted as milliseconds per "frame"
        characters: ['#', '?', '@', '+', '*', '§', '$', '£', '!', '&'],
    };

    function shuffleArray(array) {
        let currentIndex = array.length;
        while (currentIndex !== 0) {
            const randomIndex = Math.floor(Math.random() * currentIndex);
            currentIndex -= 1;
            [array[currentIndex], array[randomIndex]] = [
                array[randomIndex],
                array[currentIndex],
            ];
        }
        return array;
    }

    function getRandomCharacter(characters) {
        return characters[Math.floor(Math.random() * characters.length)];
    }

    function animateText(element, options) {
        const originalText = element.textContent;
        const splitText = originalText.split('');
        const shuffledIndices = shuffleArray([
            ...Array(splitText.length).keys(),
        ]);
        let finalText = splitText.map(() =>
            getRandomCharacter(options.characters)
        );
        let currentIteration = 0;
        let currentIndex = 0;
        let lastFrameTime = 0; // Track time for speed control

        function decodeStep(timestamp) {
            // Throttle based on speed:  Only proceed if enough time has passed
            if (timestamp - lastFrameTime < options.speed) {
                requestAnimationFrame(decodeStep);
                return; // Important: Exit early to avoid unnecessary work
            }
            lastFrameTime = timestamp; // Update the last frame time

            if (currentIteration < options.iterations) {
                if (currentIndex < shuffledIndices.length) {
                    finalText[shuffledIndices[currentIndex]] =
                        getRandomCharacter(options.characters);
                    currentIndex++;
                    element.textContent = finalText.join('');
                    requestAnimationFrame(decodeStep);
                } else {
                    currentIteration++;
                    currentIndex = 0;
                    requestAnimationFrame(decodeStep);
                }
            } else {
                if (currentIndex < shuffledIndices.length) {
                    finalText[shuffledIndices[currentIndex]] =
                        splitText[shuffledIndices[currentIndex]];
                    currentIndex++;
                    element.textContent = finalText.join('');
                    requestAnimationFrame(decodeStep);
                }
            }
        }

        requestAnimationFrame(decodeStep); // Initial call
    }

    return {
        init: function (selector, customOptions) {
            const elements = document.querySelectorAll(selector);
            if (!elements.length) {
                console.warn(`No elements found for selector: ${selector}`);
                return;
            }

            const options = { ...DEFAULT_OPTIONS, ...customOptions };

            elements.forEach((element) => {
                animateText(element, options);
            });
        },
    };
})();
