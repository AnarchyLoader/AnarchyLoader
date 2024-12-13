function isMobile() {
    const toMatch = [
        /Android/i,
        /webOS/i,
        /iPhone/i,
        /iPad/i,
        /iPod/i,
        /BlackBerry/i,
        /Windows Phone/i,
    ];

    return toMatch.some((toMatchItem) => {
        return navigator.userAgent.match(toMatchItem);
    });
}

window.onload = function () {
    console.log('Is mobile: ' + isMobile());

    if (!isMobile()) {
        setRelease();
        setNightly();

        const video = document.querySelector('.background-video');
        const warn = document.querySelector('.warn');
        var showen = false;

        video.addEventListener('ended', () => {
            video.src = 'static/assets/background_loop.mp4';
            video.loop = true;
        });

        setTimeout(() => {
            warn.style.opacity = 1;
            video.style.filter = 'brightness(0.1)';
            showen = true;
        }, 2000);

        document.addEventListener('keydown', (event) => {
            if (event.key === 'Insert' && showen) {
                handleGui();
            }
        });
    } else {
        const video = document.querySelector('.background-video');
        const gui = document.querySelector('.gui');
        video.src = 'static/assets/background_loop.mp4';
        video.loop = true;

        gui.classList.add('gui-mobile');

        handleGui();
    }
};

function handleGui() {
    console.log('Showing GUI');

    const gui = document.querySelector('.gui');
    const warn = document.querySelector('.warn');
    const logo = document.querySelector('.logo');

    if (gui.style.opacity == 0) {
        warn.style.opacity = 0;
        gui.style.opacity = 1;
        gui.style.pointerEvents = 'auto';

        setTimeout(() => {
            gui.style.height = '400px';
        }, 300);

        setTimeout(() => {
            logo.style.opacity = 1;
        }, 400);
    }
}

async function fetchJSON(url) {
    try {
        const response = await fetch(url);
        if (!response.ok) {
            throw new Error('Network response was not ok');
        }
        return await response.json();
    } catch (error) {
        console.error('Error fetching JSON:', error);
        return [];
    }
}

async function setRelease() {
    const data = await fetchJSON(
        'https://api.github.com/repos/AnarchyLoader/AnarchyLoader/releases/latest'
    );
    const downloadUrl = data?.assets?.[0]?.browser_download_url ?? '';
    const downloadButton = document.querySelector('#release');
    if (downloadUrl) {
        downloadButton.href = downloadUrl;
    } else {
        downloadButton.classList.add('disabled');
    }
}

async function setNightly() {
    const data = await fetchJSON(
        'https://api.github.com/repos/AnarchyLoader/AnarchyLoader/releases'
    );
    const downloadUrl =
        data?.find((release) => release.prerelease)?.assets?.[0]
            ?.browser_download_url ?? '';
    const downloadButton = document.querySelector('#nightly');
    if (downloadUrl) {
        downloadButton.href = downloadUrl;
    } else {
        downloadButton.classList.add('disabled');
    }
}
