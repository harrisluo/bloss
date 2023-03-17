import bs58 from "bs58";
import { ListCardsData, PgpCardInfo, SignMessageData } from "./schema";

const PCSC_HOST_NAME = "com.harrisluo.bloss";

export const listCards = async (): Promise<PgpCardInfo[]> => {
    console.log("Getting cards...");
    const promise = new Promise<PgpCardInfo[]>((resolve, reject) => {
        chrome.runtime.sendNativeMessage(
            PCSC_HOST_NAME,
            {command: "ListCards"},
            (response) => {
                console.log(response);
                if (response.Ok) {
                    const cards = (response.Ok as ListCardsData).ListCards;
                    resolve(cards);
                } else {
                    reject(response.Error);
                }
            }
        );
    })
    return promise;
}

export const signMessage = async(
    aid: string,
    message: Array<number>,
    pin: Array<number>,
    touch_callback: () => void,
): Promise<Array<number>> => {
    console.log(`Signing message...`);
    const promise = new Promise<Array<number>>((resolve, reject) => {
        const port = chrome.runtime.connectNative("com.harrisluo.bloss");
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
                reject(response.Error);
            }
        });
        port.onDisconnect.addListener(function () {
            console.log('Disconnected');
        });
        port.postMessage({command: {SignMessage: { aid, message, pin }}});
    })
    return promise;
};
