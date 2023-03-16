import { PublicKey } from "@solana/web3.js";
import { Link } from 'react-router-dom';
import { KeyIcon } from "@heroicons/react/20/solid";

export interface CardInfo {
    name: string,
    aid: string,
    pk: PublicKey,
}

export const CardItem = ({ cardInfo, linkOn }: { cardInfo: CardInfo, linkOn: boolean }) => {
    const styles = linkOn ? "flex grid-cols-2 gap-x-2 rounded-md bg-emerald-200 hover:bg-emerald-600 p-2 bg-opacity-10 hover:bg-opacity-80" :
        "flex grid-cols-2 gap-x-2 rounded-md bg-emerald-600 p-2 bg-opacity-80";
    const content = <div className={styles}>
        <div className="flex grid-cols-1 place-content-center">
            <KeyIcon className="w-16"></KeyIcon>
        </div>
        <div>
            <h2 className="text-xl font-semibold">{cardInfo.name}</h2>
            <p className="text-md text-stone-300"><span className="font-bold">OpenPGP AID:</span> {cardInfo.aid}</p>
            <p className="text-md text-stone-300"><span className="font-bold">Public key:</span> {cardInfo.pk.toBase58()}</p>
        </div>
    </div>;
    if (linkOn) {
        return <Link to="/sign" state={{cardInfo: cardInfo}}>{content}</Link>;
    } else {
        return content;
    }
};
