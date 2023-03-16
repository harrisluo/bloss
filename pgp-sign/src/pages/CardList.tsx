import { PublicKey } from "@solana/web3.js"
import { useEffect, useState } from "react"
import { CardItem, CardInfo } from "../components/CardItem";

export const CardList = () => {
    const [cards, setCards] = useState<CardInfo[]>([]);
    useEffect(
        () => getCards((cards: CardInfo[]) => {setCards(cards)}),
        [],
    );
    const cardItems = cards.map((cardInfo: CardInfo) => <CardItem cardInfo={cardInfo} linkOn={true} />)

    return <div className="p-4 text-stone-100">
        <h1 className="text-3xl text-center font-semibold pb-4">Select Card</h1>
        <div className="grid grid-rows-1 gap-y-3">{cardItems}</div>
    </div>
};

interface ListCardsResponse {
    ListCards: { name: string, aid: string, pk: Array<number> }[],
}

const getCards = (callback: (cards: CardInfo[]) => void): void => {
    console.log(`Getting cards...`);
    chrome.runtime.sendNativeMessage(
        'com.harrisluo.bloss',
        {command: "ListCards"},
        (response) => {
            console.log(response);
            const cards = (response as ListCardsResponse).ListCards.map(
                (value: { name: string, aid: string, pk: Array<number> }) => ({
                    name: value.name,
                    aid: value.aid,
                    pk: new PublicKey(value.pk)
                })
            );
            callback(cards);
        }
    );
};
