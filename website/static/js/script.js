function setOSName() {
    const userAgent = window.navigator.userAgent;
    document.OSName = /Windows/.test(userAgent)
        ? 'Windows'
        : /Mac/.test(userAgent)
        ? 'Mac/iOS'
        : /X11/.test(userAgent)
        ? 'UNIX'
        : /Linux/.test(userAgent)
        ? 'Linux'
        : 'Unknown';
}

export async function load() {
    setOSName();

    if (location.pathname === '/') {
        const latestRelease = await fetchLatestRelease();
        if (latestRelease !== '') {
            document.getElementById('download-stable').link = latestRelease;
            document
                .getElementById('download-stable')
                .classList.remove('disabled');
        }

        const latestPrerelease = await fetchLatestPrerelease();
        if (latestPrerelease !== '') {
            document.getElementById('download-prerelease').link =
                latestPrerelease;
            document
                .getElementById('download-prerelease')
                .classList.remove('disabled');
        }
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

async function fetchLatestRelease() {
    const data = await fetchJSON(
        'https://api.github.com/repos/AnarchyLoader/AnarchyLoader/releases/latest'
    );
    return data?.assets?.[0]?.browser_download_url ?? '';
}

async function fetchLatestPrerelease() {
    const data = await fetchJSON(
        'https://api.github.com/repos/AnarchyLoader/AnarchyLoader/releases'
    );
    const latestPrerelease = data?.find((release) => release.prerelease);
    return latestPrerelease?.assets?.[0]?.browser_download_url ?? '';
}

function alertCompatibility() {
    if (document.OSName !== 'Windows') {
        alert('Loader is not supported on Unix or Mac');
    }
}

async function openDownloadPage(self) {
    alertCompatibility();
    if (self.classList.contains('disabled')) return;
    window.open(self.link, '_blank');
}

document.load = load;
document.openDownloadPage = openDownloadPage;
