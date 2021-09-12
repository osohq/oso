
// Try to get data from the cache, but fall back to fetching it live.
async function getVersions() {
    const cacheName = `oso-data`;
    const url = 'https://s3.amazonaws.com/data.oso.dev/versions.json';
    console.log(`get cached data: ${url}`)
    let cachedData = await getCachedData(cacheName, url);

    if (cachedData) {
        console.log('Retrieved cached data');
        return cachedData;
    }

    console.log('Fetching fresh data');

    const cacheStorage = await caches.open(cacheName);
    await cacheStorage.add(url);
    cachedData = await getCachedData(cacheName, url);
    return cachedData;
}

// Get data from the cache.
async function getCachedData(cacheName, url) {
    const cacheStorage = await caches.open(cacheName);
    const cachedResponse = await cacheStorage.match(url);

    if (!cachedResponse || !cachedResponse.ok) {
        return false;
    }

    return await cachedResponse.json();
}

export async function setVersionList() {
    try {
        console.log("done");
        const data = await getVersions();
        console.log({ data });
        document.getElementById('version-list').innerHTML = data.versions.reduce((html, v) => {
            return html + `
          <a href="https://docs-preview.oso.dev/v/${v}/index.html"
              class="p-2 flex items-start rounded-lg hover:bg-gray-50">
              ${v}
          </a>
          `;
        }, '');
    } catch (error) {
        console.error({ error });
    }
}