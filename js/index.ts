const BLOSS_NATIVE_NAME = "com.harrisluo.bloss_native";

export type SigningAlgorithm = "ed25519";

export interface PgpCardInfo {
    manufacturer: string,
    serialNumber: string,
    aid: string,
    signingAlgo: SigningAlgorithm,
    pubkeyBytes: Array<number>,
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
                    const cards = (response.Ok as ListCardsData).ListCards;
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
    message: Array<number>,
    pin: Array<number>,
    touch_callback: () => void,
): Promise<Array<number>> => {
    console.log(`Signing message...`);
    const promise = new Promise<Array<number>>((resolve, reject) => {
        const port = chrome.runtime.connectNative(BLOSS_NATIVE_NAME);
        port.onMessage.addListener((response) => {
            console.log(response);
            if (response.Ok) {
                if (response.Ok === "AwaitTouch") {
                    touch_callback();
                } else {
                    const sigBytes = (response.Ok as SignMessageData).SignMessage;
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

interface ListCardsData {
    ListCards: PgpCardInfo[],
}

interface SignMessageData {
    SignMessage: Array<number>,
}
