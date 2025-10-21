console.log("Extension Is Running");
let allHeaders = {};


browser.downloads.onCreated.addListener((downloadItem) => {
    console.log("making post request");
    let header = allHeaders[downloadItem.url];
    fetch("http://127.0.0.1:7878", {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify({
            "url": downloadItem.url,
            "headers": header
        })
    });
    setTimeout(() => {
        browser.downloads.cancel(downloadItem.id);
    }, 100);
    console.log("url", downloadItem.url);
    console.log("headers: ", JSON.stringify(header, null, 2));
})

handler = function (details) {
    let header = {};
    for (const h of details.requestHeaders) {
        header[h.name] = h.value;
    }
    allHeaders[details.url] = header;
    console.log("adding to allHeaders: ", details.url);
    console.log("header: ", JSON.stringify(header, null, 2));
}

chrome.webRequest.onBeforeSendHeaders.addListener(
    (details) => {
        handler(details);
    },
    { urls: ["*://*/*"] },
    ["requestHeaders"]
);

// *://*/* --> this says that look for any requests.. with any protocol http or https, with any path, and any body