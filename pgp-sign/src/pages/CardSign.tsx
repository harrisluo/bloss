import { useState } from "react";
import { Link, useLocation, useParams } from "react-router-dom";
import bs58 from 'bs58';
import { sha512 } from 'js-sha512';
import { CardItem } from "../components/CardItem";

export const CardSign = () => {
    const { cardInfo } = useLocation().state;
    const [signature, setSignature] = useState<string>("");
    const [message, setMessage] = useState<string>("");

    return <div className="grid grid-rows-auto gap-y-3 p-4 text-stone-100">
        <h1 className="text-3xl text-center font-semibold pb-4">Sign Message with PGP Card</h1>
        <CardItem cardInfo={cardInfo} linkOn={false} />
        <textarea
            className="w-full h-40 text-md text-gray-50 font-mono rounded-md p-2 bg-stone-700
                       focus:outline-none focus:border border-emerald-500 border-opacity-80"
            placeholder="Message"
            value={message}
            onInput={e => setMessage((e.target as HTMLTextAreaElement).value)}
        ></textarea>
        <button
            className="w-full text-lg p-2 rounded-md
                     bg-emerald-200 hover:bg-emerald-600 bg-opacity-10 hover:bg-opacity-80"
            onClick={() => {
                signMessage(cardInfo.aid, Array.from(new TextEncoder().encode(message)), (sig: string) => setSignature(sig));
            }}
        >Sign</button>
        <textarea
            className="w-full h-18 text-md text-gray-50 font-mono rounded-md p-2 bg-stone-700
                       select-text"
            placeholder="Signature"
            value={signature}
            disabled={true}
        ></textarea>
    </div>
};

interface SignDataResponse {
    SignData: Array<number>,
}

const signMessage = (aid: string, message: Array<number>, callback: (signature: string) => void): void => {
    console.log(`Signing message...`);
    chrome.runtime.sendNativeMessage(
        'com.harrisluo.bloss',
        { command: { SignData: { aid: aid, digest: message } } },
        (response) => {
            console.log(response);
            const sigBytes = (response as SignDataResponse).SignData;
            const signature = bs58.encode(sigBytes);
            callback(signature);
        }
    );
};
