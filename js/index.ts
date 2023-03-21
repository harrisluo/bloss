const BLOSS_NATIVE_NAME = "com.harrisluo.bloss_native";

export interface PgpCardInfo {
    manufacturer: string,
    serialNumber: string,
    aid: string,
    signingAlgo: string,
    pubkeyBytes: Uint8Array,
}

export interface BlossError {
    type: string,
    details: any,
}

export const listCards = async (): Promise<PgpCardInfo[]> => {
    console.log("Getting cards...");
    const promise = new Promise<PgpCardInfo[]>((resolve, reject) => {
        chrome.runtime.sendNativeMessage(
            BLOSS_NATIVE_NAME,
            {command: "ListCards"},
            (response) => {
                console.log(response);
                if (response.Ok) {
                    const cards = response.Ok.ListCards.map(parsePgpCardInfo);
                    resolve(cards);
                } else {
                    reject(wrapError(response.Error));
                }
            }
        );
    })
    return promise;
}

export const signMessage = async (
    aid: string,
    message: Uint8Array,
    pin: Uint8Array,
    touch_callback: () => void,
): Promise<Uint8Array> => {
    console.log(`Signing message...`);
    const promise = new Promise<Uint8Array>((resolve, reject) => {
        const port = chrome.runtime.connectNative(BLOSS_NATIVE_NAME);
        port.onMessage.addListener((response) => {
            console.log(response);
            if (response.Ok) {
                if (response.Ok === "AwaitTouch") {
                    touch_callback();
                } else {
                    const sigBytes = new Uint8Array(response.Ok.SignMessage);
                    resolve(sigBytes);
                    port.disconnect();
                }
            } else {
                reject(wrapError(response.Error));
                port.disconnect();
            }
        });
        port.onDisconnect.addListener(function () {
            console.log('Disconnected');
        });
        port.postMessage({command: {SignMessage: { aid, message, pin }}});
    })
    return promise;
};

const parsePgpCardInfo = (data: any): PgpCardInfo => {
    data.pubkeyBytes = new Uint8Array(data.pubkeyBytes);
    return data;
}

const wrapError = (e: any): BlossError => {
    if (typeof e === "string") {
        return {
            type: e,
            details: null,
        };
    } else {
        const etype = Object.keys(e)[0];
        return {
            type: etype,
            details: e[etype],
        };
    }
}
