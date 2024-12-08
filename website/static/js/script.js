window.onload = function () {
    const video = document.querySelector('.background-video');
    const img = document.querySelector('.background-img');
    const warn = document.querySelector('.warn');
    var showen = false;

    video.addEventListener('ended', () => {
        video.style.display = 'none';
        img.style.display = 'block';
    });

    setTimeout(() => {
        warn.style.opacity = 1;
        video.style.filter = "brightness(0.1)";
        showen = true;
    }, 2000);

    document.addEventListener('keydown', (event) => {
        if (event.key === 'Insert' && showen) {
            handleGui();
        }
    });
}

function handleGui() {
    console.log("Showing GUI");

    const gui = document.querySelector('.gui');
    const warn = document.querySelector('.warn');
    const logo = document.querySelector('.logo');

    if (gui.style.opacity == 0) {
        warn.style.opacity = 0;
        gui.style.opacity = 1;
        gui.style.pointerEvents = 'auto';

        setTimeout(() => {
            gui.style.height = "400px";
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

async function downloadRelease() {
    console.log("Downloading release build...");
    const data = await fetchJSON("https://api.github.com/repos/AnarchyLoader/AnarchyLoader/releases/latest");
    const downloadUrl = data?.assets?.[0]?.browser_download_url ?? '';
    if (downloadUrl) {
        window.open(downloadUrl, '_blank');
    } else {
        console.error('No release build available');
    }
}

async function downloadNightly() {
    console.log("Downloading nightly build...");
    const data = await fetchJSON("https://api.github.com/repos/AnarchyLoader/AnarchyLoader/releases");
    const downloadUrl = data?.find(release => release.prerelease)?.assets?.[0]?.browser_download_url ?? '';
    if (downloadUrl) {
        window.open(downloadUrl, '_blank');
    } else {
        console.error('No nightly build available');
    }
}