import * as utils from "./utils";
import { isMessage } from "./messages";

async function contentScriptRegistered() {
    const matchingScripts = await utils.scripting.getRegisteredContentScripts({
        ids: ["plugin-polyfill"],
    });
    return matchingScripts?.length > 0;
}

async function enable() {
    if (utils.declarativeNetRequest) {
        const playerPage = utils.runtime.getURL("/player.html");
        const rules = [
            {
                id: 1,
                action: {
                    type: chrome.declarativeNetRequest.RuleActionType.REDIRECT,
                    redirect: { regexSubstitution: playerPage + "#\\0" },
                },
                condition: {
                    regexFilter: ".*",
                    responseHeaders: [
                        {
                            header: "content-type",
                            values: [
                                "application/x-shockwave-flash",
                                "application/futuresplash",
                                "application/x-shockwave-flash2-preview",
                                "application/vnd.adobe.flash.movie",
                            ],
                        },
                    ],
                    resourceTypes: [
                        chrome.declarativeNetRequest.ResourceType.MAIN_FRAME,
                    ],
                },
            },
            {
                id: 2,
                action: {
                    type: chrome.declarativeNetRequest.RuleActionType.REDIRECT,
                    redirect: { regexSubstitution: playerPage + "#\\0" },
                },
                condition: {
                    regexFilter: "^.*\\.s(?:wf|pl)(\\?.*|#.*|)$",
                    responseHeaders: [
                        {
                            header: "content-type",
                            values: [
                                "application/octet-stream",
                                "application/binary-stream",
                                "",
                            ],
                        },
                    ],
                    resourceTypes: [
                        chrome.declarativeNetRequest.ResourceType.MAIN_FRAME,
                    ],
                },
            },
            {
                id: 3,
                action: {
                    type: chrome.declarativeNetRequest.RuleActionType.REDIRECT,
                    redirect: { regexSubstitution: playerPage + "#\\0" },
                },
                condition: {
                    regexFilter: "^.*\\.s(?:wf|pl)(\\?.*|#.*|)$",
                    excludedResponseHeaders: [{ header: "content-type" }],
                    resourceTypes: [
                        chrome.declarativeNetRequest.ResourceType.MAIN_FRAME,
                    ],
                },
            },
        ];
        await utils.declarativeNetRequest.updateDynamicRules({
            removeRuleIds: [1, 2, 3],
            addRules: rules,
        });
    }
    if (
        !utils.scripting ||
        (utils.scripting.ExecutionWorld && !utils.scripting.ExecutionWorld.MAIN)
    ) {
        return;
    }
    if (!(await contentScriptRegistered())) {
        await utils.scripting.registerContentScripts([
            {
                id: "plugin-polyfill",
                js: ["dist/pluginPolyfill.js"],
                persistAcrossSessions: true,
                matches: ["<all_urls>"],
                excludeMatches: [
                    "https://sso.godaddy.com/*",
                    "https://authentication.td.com/*",
                    "https://*.twitch.tv/*",
                    "https://www.tuxedocomputers.com/*",
                    "https://*.taobao.com/*",
                    "https://*.time4learning.com/*",
                    "https://*.edgenuity.com/*",
                ],
                runAt: "document_start",
                world: "MAIN",
            },
            {
                id: "4399",
                matches: ["https://www.4399.com/flash/*"],
                js: ["dist/siteContentScript4399.js"],
                world: "MAIN",
                runAt: "document_start",
            },
        ]);
    }
}

async function disable() {
    if (utils.declarativeNetRequest) {
        await utils.declarativeNetRequest.updateDynamicRules({
            removeRuleIds: [1, 2, 3],
        });
    }
    if (
        !utils.scripting ||
        (utils.scripting.ExecutionWorld && !utils.scripting.ExecutionWorld.MAIN)
    ) {
        return;
    }
    if (await contentScriptRegistered()) {
        await utils.scripting.unregisterContentScripts({
            ids: ["plugin-polyfill", "4399"],
        });
    }
}

async function onAdded(permissions: chrome.permissions.Permissions) {
    if (
        permissions.origins &&
        permissions.origins.length >= 1 &&
        permissions.origins[0] !== "<all_urls>"
    ) {
        await utils.storage.sync.set({
            ["showReloadButton"]: true,
        });
    }
}

function onMessage(
    request: unknown,
    _sender: chrome.runtime.MessageSender,
    _sendResponse: (response: unknown) => void,
): void {
    if (isMessage(request)) {
        if (request.type === "open_url_in_player") {
            chrome.tabs.create({
                url: utils.runtime.getURL(`player.html#${request.url}`),
            });
        }
    }
}

(async () => {
    const { ruffleEnable } = await utils.getOptions();
    if (ruffleEnable) {
        await enable();
    }
})();

// Listeners must be registered synchronously at the top level,
// otherwise they won't be called in time when the service worker wakes up
if (chrome?.runtime && !chrome.runtime.onMessage.hasListener(onMessage)) {
    chrome.runtime.onMessage.addListener(onMessage);
}

utils.storage.onChanged.addListener(async (changes, namespace) => {
    if (namespace === "sync" && "ruffleEnable" in changes) {
        if (changes["ruffleEnable"]!.newValue) {
            await enable();
        } else {
            await disable();
        }
    }
});

async function handleInstalled(details: chrome.runtime.InstalledDetails) {
    if (
        details.reason === chrome.runtime.OnInstalledReason.INSTALL &&
        !(await utils.hasAllUrlsPermission())
    ) {
        await utils.openOnboardPage();
    }
}

chrome.runtime.onInstalled.addListener(handleInstalled);
utils.permissions.onAdded.addListener(onAdded);
