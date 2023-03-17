import { PublicKey } from "@solana/web3.js"
import { useEffect, useState } from "react"
import { CardItem } from "../components/CardItem";
import { listCards } from "../util/pcscHost";
import { PgpCardInfo } from "../util/schema";

export const CardList = () => {
    const [cards, setCards] = useState<PgpCardInfo[]>([]);
    useEffect(
        () => {
            listCards().then((cards: PgpCardInfo[]) => {
                setCards(cards);
            }).catch((e) => {
                alert(e)
            });
        },
        [],
    );
    const cardItems = cards.map((cardInfo: PgpCardInfo) => <CardItem cardInfo={cardInfo} linkOn={true} />)

    return <div className="p-4 text-stone-100">
        <h1 className="text-3xl text-center font-semibold pb-4">Select Card</h1>
        <div className="grid grid-rows-1 gap-y-3">{cardItems}</div>
    </div>
};
