console.log("Extension Is Running");

setTimeout(() => {
    chrome.downloads.onCreated.addListener((downloadItem) => {
    fetch("http://127.0.0.1:7878", {
        method: "POST",
        headers: {
            "Content-Type": "text/plain"
        },
        body: downloadItem.url
    });
    console.log("Download detected: ", downloadItem.url);
    chrome.downloads.cancel(downloadItem.id);
    console.log("download forwarded to odm");
})
},1000);

