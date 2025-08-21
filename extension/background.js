console.log("Extension Is Running");

chrome.downloads.onCreated.addListener((downloadItem) => {
    console.log("Download detected: ", downloadItem.url);
})